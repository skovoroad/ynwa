# Tasks

## 1. `GameState::step_count`, `RecordFooter`, and `JournalSink` Finalization in `ynwa-core`

- [ ] 1.1 Add `pub step_count: u64` (initialized to `0`, incremented by `1` in `Game::step`) to `GameState` in `ynwa-core/src/game.rs`, call `self.game.step(timestep)` in `World::step` (`ynwa-core/src/world.rs`), and remove `Game::finish_step` and `JournalSink::finish_step`.
- [ ] 1.2 Make `sorted_collections` `pub(crate)` in `ynwa-core/src/game.rs`, define `RecordFooter` (`total_steps`, `elapsed_time`, `team_stats`, `player_stats`, with `RecordFooter::from_state(state: &GameState)`) in `ynwa-core/src/record.rs`, and replace `Record::total_steps` with `pub footer: RecordFooter`.
- [ ] 1.3 Update `JournalSink::finish(self: Box<Self>, final_state: &GameState)` in `ynwa-core/src/journal.rs` and `Game::finish_journal` in `ynwa-core/src/game.rs` so `Game` passes `&self.state` to the sink without depending on `RecordFooter`, remove duplicate `steps` counters from `CollectJournalRecorder` and `EventsCollection`, and verify unit tests in `ynwa-core/src/tests/world_tests.rs`, `game_tests.rs`, and `journal_tests.rs` pass with `cargo test -p ynwa-core`.

## 2. JSON Lines Footer Codec and Call-Site Updates

- [ ] 2.1 Update `RecordWriter::finish(&mut self, footer: &RecordFooter)`, `RecordLine::Footer(RecordFooter)`, `JsonRecordWriter`, `JsonRecordReader`, and `FileJournalRecorder` (removing its private `steps` field) in `ynwa-core/src/codec.rs` (plus `write_json_record` in `ynwa-core/src/record_io.rs`) so `FileJournalRecorder::finish` builds `RecordFooter::from_state(final_state)` and writes it via `RecordWriter::finish`.
- [ ] 2.2 Update `record.footer.total_steps` accesses in `ynwa-player`, `ynwa-football` tests, and `ynwa-integration-testing`, and add unit and integration tests in `ynwa-core/src/tests/codec_tests.rs` and `ynwa-football/src/tests/journal_integration_tests.rs` verifying `RecordFooter` serialization and crash recovery (`RecordFooter::default()`), verified with `cargo test` and `cargo clippy`.
