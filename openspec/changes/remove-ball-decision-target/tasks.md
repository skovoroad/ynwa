# Tasks

## 1. Remove `DecisionTarget::Ball` from `ynwa-core` and `ynwa-player`

- [x] 1.1 Remove `DecisionTarget::Ball` from `ynwa-core/src/game.rs`, `ynwa-core/src/systems/movement.rs`, and `ynwa-core/src/systems/decision/mod.rs`.
- [x] 1.2 Remove `target_type == "ball"` handling from `parse_decision` in `ynwa-core/src/systems/decision/decision_parser.rs` so `"ball"` returns an unknown `target_type` error.
- [x] 1.3 Remove the `&BallState` parameter and `DecisionTarget::Ball` arm from `resolve_target_point` in `ynwa-core/src/systems/decision/util.rs` and update its call site in `ynwa-core/src/systems/decision/decision_system.rs`.
- [x] 1.4 Remove the `DecisionTarget::Ball` match arm in `ynwa-player/src/ui.rs`.

## 2. Update Tests and Verify

- [x] 2.1 Update unit and integration test fixtures using `DecisionTarget::Ball` in `ynwa-core/src/tests/` (`movement_tests.rs`, `decision_util_tests.rs`, `serde_tests.rs`) and `ynwa-football/src/` (`test_utils.rs`, `tests/replay_tests.rs`, `tests/serialization_tests.rs`, `tests/lockstep_tests.rs`).
- [x] 2.2 Update `ynwa-script-tests/tests/basic_scripts.rs` to assert that `target_type = "ball"` is rejected as an error.
- [x] 2.3 Run `cargo test` and `cargo clippy` across the workspace to confirm all tests pass with no warnings.

