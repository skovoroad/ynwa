# Tasks

## 1. Direct tests for unexercised stdlib functions

- [ ] 1.1 Add a test for `chase_ball()` asserting `Run(Ball)` with reason `"chase_ball"`; verify `cargo test -p ynwa-script-tests` passes
- [ ] 1.2 Add a test for `stop(reason)` asserting `Stop` with the given reason; verify `cargo test -p ynwa-script-tests` passes
- [ ] 1.3 Extend the region helper (or add one) so `my_regions()` provides `"attack"` and `"defence"`, and add tests for `run_to_attack_position()` and `run_to_defence_position()` asserting `Run(Point)` to the region center with the matching reason; verify `cargo test -p ynwa-script-tests` passes
- [ ] 1.4 Add a test for `run_to_opponent_penalty_area()` on the football field asserting `Run(Point)` to the penalty-area center with reason `"run_to_opponent_penalty_area"`; verify `cargo test -p ynwa-script-tests` passes

## 2. Coverage completeness

- [ ] 2.1 Run `cargo test -p ynwa-script-tests` and confirm every public `stdlib.lua` function has a direct test and no function is left unexercised
