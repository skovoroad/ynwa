# Spec Delta

## MODIFIED Requirements

### Requirement: Event-Sourced Record Model and JournalSink Lifecycle

Match recording SHALL store `Record { header: RecordHeader { config, initial_state, timestep }, total_steps: u64, journal: Vec<JournalEntry> }` instead of per-frame dumps. Pluggable `JournalSink` recording MUST default to zero overhead when disabled, count steps once per `World::step`, and defer streaming I/O errors until journal finalization without aborting simulation steps.

#### Scenario: Zero-overhead default, pre-step header snapshot, and deferred I/O errors
- **WHEN** `RecordHeader::from_game(game, timestep)` is captured before the first `World::step` and `N` steps execute
- **THEN** unrecorded matches incur zero allocation overhead, in-memory collection assembles a `Record` with `total_steps == N` on finalization, and streaming file recording flushes the header immediately and surfaces any write failure on `finish_journal()` rather than panicking mid-step

### Requirement: Deterministic Serialization, Codec Traits, and Crash-Resilient JSON Lines

`GameConfig` and `GameState` SHALL serialize unordered maps, sets, and `Field` zones in deterministic sorted key order. `RecordWriter` and `RecordReader` MUST decouple `Record` from byte transport (supporting streaming files, in-memory buffers, and whole-`Record` dumps). `JsonRecordWriter` flushes newline-terminated JSON Lines (`header`, `event`..., `footer`); `JsonRecordReader` rejects missing/duplicate headers and ignores trailing lines without `'\n'`.

#### Scenario: Recovering valid prefix from crashed recording
- **WHEN** a `.jsonl` recording contains 1 `"header"` line (`config`, `initial_state`, `timestep`), `K` newline-terminated `"event"` lines, and a truncated final line without `'\n'` (no `"footer"`)
- **THEN** `JsonRecordReader::read()` returns `Ok(Record)` containing the deterministically serialized header, all `K` complete events, and `total_steps == 0`

### Requirement: Replay Pipeline and Observable State Equivalence Contract

Replay world construction SHALL initialize `Game` from `record.header.config`, overlay `record.header.initial_state`, and run physics-only re-simulation driven by due journal entries (`entry.timestamp <= current_timestamp`, plus `Setup` ball pinning) without Lua, decision systems, sport manager, or RNG. Stepping `total_steps` times at `timestep` on the same platform MUST reproduce all observable `GameState` fields within `1e-4`.

#### Scenario: Observable state equivalence vs excluded internal coordination flags
- **WHEN** a match recorded on a platform with `timestep` is replayed step-by-step with `timestep` on the same platform
- **THEN** at every step, `elapsed_time`, `stage`, `restart_position`, `restart_team`, `team_stats`, `player_stats`, all `BallState` fields, and `player_states[i].{position, velocity, current_decision, decision_reason}` match the recorded run within `1e-4`, and `check_events` evaluated before each `Play` step matches `decode_football_events(&record.journal)`
- **AND** internal live-pipeline coordination flags `player_states[i].{needs_decision, is_ready, last_error, decision_processed, last_decision_time}` are excluded from replay equivalence guarantees
