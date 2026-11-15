# Tasks

## 1. Remove `GameStage::GameOver` from `ynwa-core`

- [x] 1.1 Remove the `GameOver` variant from `GameStage` in `ynwa-core/src/game.rs`, simplify `Game::with_stage` player positioning (`Play | GameOver` → `Play`), drop the `GameOver` arm from `ScriptedDecisionMaker::make_decision` in `ynwa-core/src/systems/decision/scripted_decision_maker.rs`, and update `ynwa-core/src/tests/serde_tests.rs` (remove `GameStage::GameOver` from the round-trip cases); verify `cargo test -p ynwa-core` passes.

## 2. Remove `GameOver` from `ynwa-football` and `ynwa-player`

- [x] 2.1 Remove the `GameStage::GameOver => {}` arm from `FootballGameManager::update` in `ynwa-football/src/game_manager.rs` and the `GameStage::GameOver` test case in `ynwa-player/src/ui_tests.rs`; verify `cargo test -p ynwa-football -p ynwa-player` passes.

## 3. Remove `GameOver` from `ynwa-integration-testing`

- [x] 3.1 Remove `StageNameDto::GameOver` in `ynwa-integration-testing/src/dto.rs`, `StageMatcher::GameOver` and its arms in `ynwa-integration-testing/src/criterion.rs`, the `GameOver` arm in `loader::resolve_stage`, and the `GameOver` arms in `compare::{stage_matches, stop_stage_matches}`; update `loader_tests.rs`, `dto_tests.rs`, `compare_tests.rs`, `criterion_tests.rs`, and `tests/fixtures/failing/final_state.toml` to drop `"GameOver"`; verify `cargo test -p ynwa-integration-testing` passes.

## 4. Workspace verification

- [x] 4.1 Run `cargo test`, `cargo clippy --all-targets`, and `cargo fmt --check` and confirm all pass with no warnings and no remaining `GameOver` references in source.
