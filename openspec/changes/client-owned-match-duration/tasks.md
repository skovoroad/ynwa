# Tasks

## 1. Remove `GAME_DURATION` and `FootballEvent::GameEnd` from `ynwa-football`

- [x] 1.1 Remove `GAME_DURATION`, `check_game_end`, and `FootballEvent::GameEnd` from `ynwa-football/src/events.rs` and the `FootballEvent::GameEnd` arm from `FootballGameManager::handle_event` in `ynwa-football/src/game_manager.rs`, update `events_tests.rs`, `journal_integration_tests.rs`, and `replay_tests.rs`, and verify `cargo test -p ynwa-football` passes.

## 2. Remove `GameEnd` from `ynwa-integration-testing`

- [x] 2.1 Remove `EventKindDto::GameEnd` in `ynwa-integration-testing/src/dto.rs`, and `EventMatcher::GameEnd` (including its special-case `team` rejection in `from_def` and its `matches_football` arm) in `ynwa-integration-testing/src/criterion.rs`, update tests in `ynwa-integration-testing/src/tests/*` (`compare_tests.rs`, `criterion_tests.rs`, `dto_tests.rs`, `runner_tests.rs`), and verify `cargo test -p ynwa-integration-testing` passes.

## 3. Client-Owned Match Duration in `ynwa-player`

- [x] 3.1 Add `DEFAULT_MATCH_DURATION = 120.0` and `--duration <seconds>` (`> 0.0` and finite, rejected with `--replay`) to `ynwa-player/src/cli.rs`, stop stepping once `elapsed_time >= cli.duration` in `Play` and `Record` modes (finalizing the recording and keeping the window open until `Escape` in `Record` mode) in `ynwa-player/src/main.rs`, remove `FootballEvent::GameEnd` formatting from `ynwa-player/src/ui.rs` and its `GameEnd` case in `ynwa-player/src/ui_tests.rs`, and verify `cargo test` and `cargo clippy` pass.

