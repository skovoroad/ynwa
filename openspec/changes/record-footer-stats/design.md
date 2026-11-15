# Design

## Context

See `proposal.md` for motivation. Currently in `ynwa-core`:
- `GameState` stores `elapsed_time: f32`, but discrete step count is tracked outside `GameState` by calling `game.finish_step(new_timestamp)` in `World::step`, which forwards to `JournalSink::finish_step`.
- `FileJournalRecorder` and `CollectJournalRecorder` each maintain their own duplicate `steps: u64` counter.
- `Game::finish_journal(&mut self)` detaches `self.journal_sink` and calls `sink.finish()`, which takes no arguments.
- `RecordLine::Footer` in `ynwa-core/src/codec.rs` only serializes `{ total_steps: u64 }`.

## Goals / Non-Goals

**Goals:**
- Track `step_count: u64` directly in `GameState` alongside `elapsed_time: f32` and remove `Game::finish_step` / `JournalSink::finish_step` and duplicate recorder step counters.
- Capture final `step_count` (`total_steps`), `elapsed_time`, `team_stats`, and `player_stats` from `&GameState` when `Game::finish_journal()` is called and write them into the `Footer` of the recording.
- Respect layer boundaries: `Game` and `JournalSink` operate solely on domain types (`&GameState`), while `RecordFooter` lives in the recording/codec layer (`record.rs` / `codec.rs`) as the symmetric counterpart to `RecordHeader`.
- Group all trailer fields in `Record { header: RecordHeader, footer: RecordFooter, journal: Vec<JournalEntry> }` cleanly without legacy backward-compatibility shims.
- Ensure deterministic JSON key ordering for `team_stats` in the `footer` line.

**Non-Goals:**
- Backward compatibility with older `.jsonl` footer formats or keeping a deprecated top-level `Record::total_steps` field.
- Implementing specific new football metrics (e.g., xG, possession percentage, distance run) in this change; this change builds the recording footer transport so any stat added to `GameState::team_stats` or `player_stats` is automatically persisted.

## Decisions

1. **Unify `step_count: u64` in `GameState` and remove `finish_step`**:
   - Add `pub step_count: u64` to `GameState` (initialized to `0`).
   - In `Game::step(&mut self, timestep: f32)`, increment both `self.state.elapsed_time += timestep` and `self.state.step_count += 1`.
   - In `World::step(&mut self, timestep: f32)`, call `self.game.step(timestep)` after running all systems, and remove `Game::finish_step` and `JournalSink::finish_step`.

2. **Strict layer separation between `JournalSink::finish` (`&GameState`) and `RecordWriter::finish` (`&RecordFooter`)**:
   - **Domain / Journal layer (`game.rs` <-> `journal.rs`)**: `Game` does not know about `Record`, `RecordHeader`, or `RecordFooter`. `Game::finish_journal(&mut self)` passes `&self.state` to `JournalSink::finish(self: Box<Self>, final_state: &GameState)`.
   - **Sink implementation layer (`FileJournalRecorder` / `CollectJournalRecorder`)**: Neither recorder stores a private `steps` counter. On `finish(final_state)`, `FileJournalRecorder` constructs `RecordFooter::from_state(final_state)` and passes `&footer` to `self.writer.finish(&footer)`. `CollectJournalRecorder` passes `final_state` to `EventsCollection`.
   - **Recording / Codec layer (`record.rs` <-> `codec.rs`)**: Defines `RecordFooter { pub total_steps: u64, pub elapsed_time: f32, pub team_stats: HashMap<Team, StatSet>, pub player_stats: Vec<StatSet> }` with `RecordFooter::from_state(state: &GameState)` (mapping `state.step_count` to `total_steps`), symmetric to `RecordHeader::from_game(game, timestep)`.

3. **Symmetric `Record` structure (`header`, `footer`, `journal`) and shared `sorted_collections` module**:
   - Replace `pub total_steps: u64` on `Record` with `pub footer: RecordFooter` so `Record` mirrors the 3-part stream structure (`header`, `journal`, `footer`). All call sites (`ynwa-player`, `ynwa-integration-testing`, tests) are updated to `record.footer.total_steps`.
   - Extract `mod sorted_collections` out of `ynwa-core/src/game.rs` into its own crate-internal module `ynwa-core/src/sorted_collections.rs` (`pub(crate) mod sorted_collections;` in `lib.rs`), shared cleanly by `game.rs` and `record.rs`.
   - In `RecordFooter`, serialize `team_stats` via `#[serde(serialize_with = "crate::sorted_collections::map")]` for deterministic output. When `JsonRecordReader::read` recovers a crashed recording that has no `"footer"` line at all, `footer` defaults to `RecordFooter::default()`.

## Risks / Trade-offs

- **[Risk] Interrupted/crashed recordings have no footer line** → Mitigation: By design, crashed recordings recover up to the last flushed event with `RecordFooter::default()` (`total_steps = 0`, `elapsed_time = 0.0`, empty stats); normal completions (including early `Escape` exit in `ynwa-player --record` and completion in `ynwa-simulator`) call `finish_journal()` and write the full footer.
