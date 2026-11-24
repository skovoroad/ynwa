# Tasks

## 1. Wire-format change

- [x] 1.1 Add `team` + `number` to `JournalEvent::DecisionAssigned` and `JournalEvent::KickOutcome`, and `possessed_by_team`/`possessed_by_number` to `JournalEvent::PossessionChange` in `ynwa-core/src/journal.rs`; verify `cargo build -p ynwa-core` surfaces the compile errors at the recording/test sites that still need updating
- [x] 1.2 Update the `ReplayDriver::apply_event` match arms in `ynwa-core/src/replay.rs` to ignore the new fields via `..`; verify `cargo test -p ynwa-core replay` passes with no behavior change

## 2. Populate the new fields at recording sites

- [x] 2.1 Populate `team`/`number` in the three `DecisionAssigned` recording sites in `ynwa-core/src/systems/decision/decision_system.rs` from `config.players[player_index]`; verify `decision_system_tests` assertions include the new fields
- [x] 2.2 Populate `team`/`number` in the `KickOutcome` recording site in `ynwa-core/src/systems/action.rs`; verify `action_tests::test_kick_outcome_is_recorded` asserts them
- [x] 2.3 Populate `possessed_by_team`/`possessed_by_number` in the `PossessionChange` recording site in `ynwa-core/src/systems/ball_possession.rs`, including the `None` ball-free case; verify `ball_possession_tests::test_records_possession_change` asserts both branches

## 3. Update serialization and unit tests

- [x] 3.1 Update the event constructors in `ynwa-core/src/tests/codec_tests.rs`, `journal_tests.rs`, and `replay_tests.rs` to include the new fields; verify `cargo test -p ynwa-core` passes
- [x] 3.2 Extend `journal_tests::journal_entries_round_trip_through_json` to cover a `PossessionChange` with `possessed_by = None` and assert the new fields round-trip; verify the test passes

## 4. Render team + tactical number in the player UI

- [x] 4.1 Change `event_text` in `ynwa-player/src/ui.rs` to render `DecisionAssigned`, `KickOutcome`, and `PossessionChange` with `team` + `number` (e.g. `A#7`) instead of `#<index>`; update the three affected `event_text` tests in `ynwa-player/src/ui_tests.rs` (`decision_assigned_renders_player_and_decision`, `possession_change_renders_possessor_and_last_team`, `kick_outcome_renders_kicker`) to include the new fields and expected labels; verify `cargo test -p ynwa-player` passes
- [x] 4.2 Run `cargo clippy --workspace` and `cargo fmt --check --all`; verify no warnings
