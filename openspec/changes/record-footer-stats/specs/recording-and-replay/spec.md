# Spec Delta

## MODIFIED Requirements

### Requirement: Event-Sourced Record Model and JournalSink Lifecycle

Match recording SHALL store `Record { header: RecordHeader { config, initial_state, timestep }, footer: RecordFooter { total_steps, elapsed_time, team_stats, player_stats }, journal: Vec<JournalEntry> }` instead of per-frame dumps. Pluggable `JournalSink` recording MUST default to zero overhead when disabled, receive final `&GameState` upon `Game::finish_journal()` to populate `RecordFooter` (`total_steps = final_state.step_count`, `elapsed_time`, `team_stats`, `player_stats`), and defer streaming I/O errors until journal finalization without aborting simulation steps.

#### Scenario: Zero-overhead default, pre-step header snapshot, and deferred I/O errors
- **WHEN** `RecordHeader::from_game(game, timestep)` is captured before the first `World::step` and `N` steps execute
- **THEN** unrecorded matches incur zero allocation overhead, in-memory collection assembles a `Record` with `footer.total_steps == N` and final `elapsed_time`, `team_stats`, and `player_stats` on finalization, and streaming file recording flushes the header immediately and surfaces any write failure on `finish_journal()` rather than panicking mid-step

#### Scenario: Final match statistics captured in footer without journal event spam
- **WHEN** simulation systems mutate `game.state.team_stats` and `game.state.player_stats` across steps and `game.finish_journal()` is called
- **THEN** the finalized `Record.footer` and trailing `footer` line contain the exact final `step_count` (`total_steps`), `elapsed_time`, `team_stats`, and `player_stats` from `game.state` even for statistics that do not emit `JournalEvent::StatUpdate`

### Requirement: Deterministic Serialization, Codec Traits, and Crash-Resilient JSON Lines

`GameConfig`, `GameState`, and `RecordFooter` SHALL serialize unordered maps, sets, and `Field` zones in deterministic sorted key order. `RecordWriter` and `RecordReader` MUST decouple `Record` from byte transport (supporting streaming files, in-memory buffers, and whole-`Record` dumps). `JsonRecordWriter` flushes newline-terminated JSON Lines (`header`, `event`..., `footer`); `JsonRecordReader` rejects missing/duplicate headers and ignores trailing lines without `'\n'`.

#### Scenario: Recovering valid prefix from crashed recording
- **WHEN** a `.jsonl` recording contains 1 `"header"` line (`config`, `initial_state`, `timestep`), `K` newline-terminated `"event"` lines, and a truncated final line without `'\n'` (no `"footer"`)
- **THEN** `JsonRecordReader::read()` returns `Ok(Record)` containing the deterministically serialized header, all `K` complete events, and `footer == RecordFooter::default()` (`total_steps == 0`, `elapsed_time == 0.0`, empty `team_stats` and `player_stats`)

#### Scenario: Deterministic footer serialization round-trip
- **WHEN** a completed `Record` with populated `team_stats` and `player_stats` is written via `JsonRecordWriter` and read back via `JsonRecordReader`
- **THEN** the final `"type": "footer"` line deterministically serializes `total_steps`, `elapsed_time`, `team_stats`, and `player_stats`, and `JsonRecordReader::read()` reconstructs all footer fields identically
