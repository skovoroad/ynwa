# Tasks

## 1. Backfill ynwa-core coverage before removing duplicates

- [ ] 1.1 Add `decision_parser.rs` unit tests for `target_type = "ball"` (with and without the optional `target` field) so the parser branch stays covered after the `basic_scripts.rs` removal. Verify: `cargo test -p ynwa-core decision_parser`
- [ ] 1.2 Add a `scripted_decision_maker.rs` unit test asserting the full `context` JSON field set (`me`, `teammates`, `opponents`, `ball`, `game`), replacing the direct-`context`-access Lua test being removed. Verify: `cargo test -p ynwa-core scripted_decision_maker`

## 2. Narrow ynwa-script-tests

- [ ] 2.1 Delete `ynwa-script-tests/scenarios/` and confirm no test references it (grep for `scenarios` in `ynwa-script-tests/**/*.rs` returns nothing). Verify: `cargo test -p ynwa-script-tests` still compiles
- [ ] 2.2 Delete `tests/team_orientation.rs`. Verify: `cargo test -p ynwa-script-tests` still compiles
- [ ] 2.3 In `tests/basic_scripts.rs`, remove the JSON -> `Decision` parsing tests and the context-shape tests; move `test_kick_if_ball_owner` and `test_get_opponent_goal_returns_valid_zone` into `tests/stdlib_functions.rs`; delete `tests/basic_scripts.rs`. Verify: `cargo test -p ynwa-script-tests` passes
- [ ] 2.4 Prune now-unused helpers from `src/lib.rs` (`create_test_game_with_script`, `create_test_game_with_script_and_stage`). Verify: `cargo clippy -p ynwa-script-tests --all-targets` reports no dead-code warnings

## 3. Integration checks

- [ ] 3.1 Run `cargo test -p ynwa-core -p ynwa-script-tests` and confirm the affected crates pass
- [ ] 3.2 Run `cargo clippy --workspace --all-targets` and `cargo fmt --all -- --check` and confirm they are clean

## Workflow follow-up

- Archive the change after review to apply the spec deltas to `scripting-and-protocols` and `integration-testing`.
