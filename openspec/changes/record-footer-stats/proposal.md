# Proposal

## Why

As we run batches of matches to evaluate and compare team tactics, we need a reliable way to collect match scores and statistics. Some future tactical and physical statistics (such as continuous possession time, distance covered, or zone occupancy) will be accumulated in `GameState` (`team_stats` and `player_stats`) during simulation without emitting per-tick journal events, so they cannot be reconstructed solely from the event stream. Writing the final match statistics into the recording's `Footer` at the end of `.jsonl` recordings makes every recording self-contained and allows external scripts to extract match results and full statistics in `O(1)` from the final line.

Additionally, step counting (`total_steps`) was previously tracked outside `GameState` via `JournalSink::finish_step`, forcing every recorder (`FileJournalRecorder`, `CollectJournalRecorder`) to maintain its own duplicate step counter solely because `JournalSink::finish` had no access to `GameState`. Moving `step_count: u64` into `GameState` alongside `elapsed_time: f32` unifies simulation progress in `GameState` and allows `RecordFooter` to be constructed purely from `&GameState`.

## What Changes

- **BREAKING**: Add `pub step_count: u64` (initialized to `0`, incremented by `1` in `Game::step`) to `GameState`, and remove `Game::finish_step` and `JournalSink::finish_step`.
- Introduce `RecordFooter` in `ynwa-core::record` (symmetric to `RecordHeader`) carrying `total_steps: u64`, `elapsed_time: f32`, `team_stats: HashMap<Team, StatSet>`, and `player_stats: Vec<StatSet>`, constructed directly via `RecordFooter::from_state(state: &GameState)`.
- **BREAKING**: Replace `total_steps: u64` on `Record` with `footer: RecordFooter` (`Record { header: RecordHeader, footer: RecordFooter, journal: Vec<JournalEntry> }`) and keep abstraction boundaries clean across layers:
  - `Game::finish_journal` passes domain `&GameState` to `JournalSink::finish(self: Box<Self>, final_state: &GameState)` (`Game` and `JournalSink` do not depend on `RecordFooter`).
  - `FileJournalRecorder::finish` builds `RecordFooter::from_state(final_state)` and passes `&RecordFooter` to `RecordWriter::finish(&mut self, footer: &RecordFooter)`.
  - `CollectJournalRecorder` only buffers `JournalEntry`s into a shared `Vec`; the caller assembles `Record` via `Record::from_final_state(header, &final_state, journal)`. This removes `EventsCollection` entirely.
- Update the JSON Lines `RecordLine::Footer(RecordFooter)` format to serialize `total_steps`, `elapsed_time`, `team_stats` (deterministically sorted), and `player_stats` directly without legacy format backward-compatibility shims.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `core-simulation`: `GameState` tracks `step_count: u64` alongside `elapsed_time: f32` in `Game::step`, removing `Game::finish_step`.
- `recording-and-replay`: Remove `JournalSink::finish_step`; `JournalSink::finish` receives `&GameState`, and `Record` / JSON Lines `footer` carry `RecordFooter { total_steps, elapsed_time, team_stats, player_stats }`.

## Impact

- **ynwa-core**: `sorted_collections.rs` (extracted from `game.rs`), `game.rs` (`GameState::step_count`, `Game::step`, `Game::finish_journal`), `world.rs` (`World::step`), `record.rs` (`RecordFooter`, `Record`, `Record::from_final_state`), `journal.rs` (`JournalSink`, `CollectJournalRecorder`), `codec.rs` (`RecordWriter::finish`, `RecordLine::Footer`, `JsonRecordWriter`, `JsonRecordReader`, `FileJournalRecorder`), and `record_io.rs` (`write_json_record`).
- `ynwa-player`, `ynwa-football`, and `ynwa-integration-testing`: updated to read `record.footer.total_steps`.
