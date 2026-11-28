# Tasks

## 1. Add iterator helpers to `stdlib.lua`

- [ ] 1.1 Write failing tests for `each(numbers)` (yields each number in list order as a single value; empty list yields no iterations) in `ynwa-script-tests/tests/stdlib_functions.rs`, and confirm they are red because `each` is nil (`cargo test -p ynwa-script-tests --test stdlib_functions`).
- [ ] 1.2 Implement `each(numbers)` as a public closure iterator in `ynwa-scripts/preambles/stdlib.lua` and confirm the `each` tests are green.
- [ ] 1.3 Write failing tests for `order_by_distance_asc(numbers)` (ascending `distance` to me; stable list order on equal distances; empty list yields no iterations; composes with `teammate_numbers()` / `opponent_numbers()`) in `ynwa-script-tests/tests/stdlib_functions.rs`, and confirm they are red because `order_by_distance_asc` is nil.
- [ ] 1.4 Implement `order_by_distance_asc(numbers)` as a public closure iterator in `ynwa-scripts/preambles/stdlib.lua` and confirm the `order_by_distance_asc` tests are green.

## 2. Rewrite team preambles with iterator loops

- [ ] 2.1 Rewrite `nearest_teammate_number` call sites in `teams/team_a/preamble.lua` to inline `for player in order_by_distance_asc({...}) do return pass_to(player) end` (removing the helper); verify `luac -p teams/team_a/preamble.lua` succeeds and `cargo test -p ynwa-script-tests` passes.
- [ ] 2.2 Rewrite `nonforward_with_ball` in `teams/team_b/preamble.lua` to use `for player in each({10, 11}) do ... end`; verify `luac -p teams/team_b/preamble.lua` succeeds and `cargo test -p ynwa-script-tests` passes.
- [ ] 2.3 Rewrite `nearest_teammate_number`, `pass_to_free_or_nearest`, `center_back_with_ball`, and `pass_to_first_free_ahead` in `teams/team_c/preamble.lua` to use `each(...)` (priority order) and `order_by_distance_asc(...)` (nearest) loops; verify `luac -p teams/team_c/preamble.lua` succeeds and `cargo test -p ynwa-script-tests` passes.
- [ ] 2.4 Rewrite `nearest_teammate_number`, `nearest_teammate_ahead_number`, and `forward_with_ball` in `teams/team_d/preamble.lua` to use `each(...)` / `order_by_distance_asc(...)` loops, preserving `my_number ~= player` and the `"pass to " .. player` reason strings; verify `luac -p teams/team_d/preamble.lua` succeeds and `cargo test -p ynwa-script-tests` passes.

## 3. Final verification

- [ ] 3.1 Verify no rewritten team preamble calls `get_teammates()` / `get_opponents()` or uses nested loops, and run `cargo test` across the workspace to confirm all tests pass.
- [ ] 3.2 Run a differential regression: for the six team pairs, replay matches with `--temperature 0` and a fixed seed on the pre-change and post-change preambles, and confirm the journals and final stats are identical (no tactical behavior change).

## Workflow follow-up

- Archive the change after review and verification.
