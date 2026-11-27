# Tasks

## 1. Add Full-Team Number List Helpers to `stdlib.lua`

- [ ] 1.1 Add `teammate_numbers()` (excluding `me`) and `opponent_numbers()` to `ynwa-scripts/preambles/stdlib.lua` and add unit tests in `ynwa-script-tests/tests/stdlib_functions.rs`.

## 2. Rewrite Team Preambles with DSL-Aligned Loops

- [ ] 2.1 Rewrite `nearest_teammate_number` in `teams/team_a/preamble.lua` using a distance-ordered loop over explicit candidate numbers with an inline comment noting it intentionally matches the future DSL loop design; verify `cargo test -p ynwa-script-tests` passes.
- [ ] 2.2 Rewrite `nonforward_with_ball` in `teams/team_b/preamble.lua` using a priority-ordered loop over `{10, 11}` with an inline DSL-design comment; verify `cargo test -p ynwa-script-tests` passes.
- [ ] 2.3 Rewrite `nearest_teammate_number`, `pass_to_free_or_nearest`, `center_back_with_ball`, and `pass_to_first_free_ahead` in `teams/team_c/preamble.lua` using DSL-aligned loops (priority-ordered `ipairs` and stable `table.sort` by distance) with inline DSL-design comments; verify `cargo test -p ynwa-script-tests` passes.
- [ ] 2.4 Rewrite `nearest_teammate_number`, `nearest_teammate_ahead_number`, and `forward_with_ball` in `teams/team_d/preamble.lua` using DSL-aligned loops (preserving `my_number ~= n` and pass reason strings) with inline DSL-design comments; verify `cargo test -p ynwa-script-tests` passes.

## 3. Final Verification

- [ ] 3.1 Verify no rewritten team preamble calls `get_teammates()` or `get_opponents()` or uses nested loops, and run `cargo test` across the workspace to confirm all tests pass.

