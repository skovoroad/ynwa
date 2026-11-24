# Tasks

## 1. Add the `compose_reason` helper and reason parameters

- [ ] 1.1 Add a `local compose_reason(custom, builtin)` helper in `ynwa-scripts/preambles/stdlib.lua` that returns `builtin` for a nil/empty `custom`, otherwise `custom .. " | " .. builtin`; verify the helper is in the internal section (not visible to team preambles) and `cargo test -p ynwa-script-tests` still passes
- [ ] 1.2 Add an optional `reason` parameter to `chase_ball`, `run_to_region`, `kick_to_opponent_goal`, `run_to_start_position`, `run_to_attack_position`, `run_to_defence_position`, `run_to_opponent_penalty_area`, `default_goalkeeper_cover_position`, and `default_goalkeeper_with_ball`, routing the final reason through `compose_reason`; verify existing `stdlib_functions` tests still pass (default strings unchanged)

## 2. Cover custom and composed reasons in script tests

- [ ] 2.1 Add tests in `ynwa-script-tests/tests/stdlib_functions.rs` asserting `chase_ball("recover")` yields `"recover | chase_ball"` and `chase_ball()` yields `"chase_ball"`; verify `cargo test -p ynwa-script-tests` passes
- [ ] 2.2 Add a test asserting `default_goalkeeper_with_ball("build_up")` yields a reason of the form `"build_up | pass_to_#<n>"`; verify the test passes
- [ ] 2.3 Add a test asserting `run_to_attack_position("press")` yields `"press | run_to_attack_position"` and the missing-region fallback still yields `"no_attack_position"`; verify the test passes

## 3. Verify the workspace

- [ ] 3.1 Run `cargo test -p ynwa-script-tests` and `cargo clippy --workspace` and `cargo fmt --check --all`; verify no warnings and all tests pass
