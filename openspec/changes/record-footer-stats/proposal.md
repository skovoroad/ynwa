# Proposal

## Why

As we run batches of matches to evaluate and compare team tactics, we need a reliable way to collect match scores and statistics. Some future tactical and physical statistics (such as continuous possession time, distance covered, or zone occupancy) will be accumulated in `GameState` (`team_stats` and `player_stats`) during simulation without emitting per-tick journal events, so they cannot be reconstructed solely from the event stream. Writing the final match statistics into the recording's `Footer` at the end of `.jsonl` recordings makes every recording self-contained and allows external scripts to extract match results and full statistics in `O(1)` from the final line.

## What Changes

- Introduce `RecordFooter` in `ynwa-core::record` (symmetric to `RecordHeader`) carrying `total_steps: u64`, `elapsed_time: f32`, `team_stats: HashMap<Team, StatSet>`, and `player_stats: Vec<StatSet>`.
- **BREAKING**: Replace `total_steps: u64` on `Record` with `footer: RecordFooter` (`Record { header: RecordHeader, footer: RecordFooter, journal: Vec<JournalEntry> }`) and keep abstraction boundaries clean across layers:
  - `Game::finish_journal` passes domain `&GameState` to `JournalSink::finish(self: Box<Self>, final_state: &GameState)` (`Game` and `JournalSink` do not depend on `RecordFooter`).
  - `FileJournalRecorder::finish` combines its step counter (`self.steps`) with `final_state` to build `RecordFooter` and passes `&RecordFooter` to `RecordWriter::finish(&mut self, footer: &RecordFooter)`.
- Update the JSON Lines `RecordLine::Footer(RecordFooter)` format to serialize `total_steps`, `elapsed_time`, `team_stats` (deterministically sorted), and `player_stats` directly without legacy format backward-compatibility shims.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `recording-and-replay`: `Record` and the JSON Lines `footer` line carry final match statistics (`elapsed_time`, `team_stats`, `player_stats`) in addition to `total_steps`, populated automatically when `Game::finish_journal()` finalizes a recording.

## Impact

- `ynwa-core`: `record.rs` (`RecordFooter`, `Record`), `journal.rs` (`JournalSink::finish`, `EventsCollection`, `CollectJournalRecorder`), `codec.rs` (`RecordWriter::finish`, `RecordLine::Footer`, `JsonRecordWriter`, `JsonRecordReader`, `FileJournalRecorder`), `record_io.rs` (`write_json_record`), and `game.rs` (`Game::finish_journal`).
- `ynwa-player` and `ynwa-integration-testing`: updated to use `RecordFooter` / `record.footer.total_steps` (or equivalent accessor) with no behavioral regression.

