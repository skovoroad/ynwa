# Tasks

## 1. Core Preamble (Layer 1)

- [ ] 1.1 Add `get_opponents()` returning `context.opponents` in `ynwa-scripts/preambles/core.lua`
- [ ] 1.2 Mark non-user-facing Layer 1 functions (`ball_owner`, `my_regions`, `get_teammates`, `get_opponents`, `my_index`, `get_opponent_goal`, `get_own_goal`, `get_opponent_penalty_area`) as internal by comment; document that `get_teammates()` includes me

## 2. Standard Library (Layer 2)

- [ ] 2.1 Wrap `ynwa-scripts/preambles/stdlib.lua` in a `do ... end` block with an internal section at the top (separated by a dedicated comment) and a public API section below
- [ ] 2.2 Move `parse_col`, `parse_notation`, `is_in_region_obj`, `run_to_region_obj`, `get_teammate_by_number`, and `am_i_ball_owner` into the internal section as `local` functions; add constants `FREE_PLAYER_RADIUS = 5.0` and `DIRECTION_TOLERANCE = 0.5`
- [ ] 2.3 Remove `pass_to_teammate`, `pass_to_players_by_numbers`, `run_to_start_position`, `kick_to_cell`, and `kick_to_region`
- [ ] 2.4 Add `count_teammates_around(pos, radius)` and `count_opponents_around(pos, radius)` (strict `<`, including me for teammates)
- [ ] 2.5 Add `teammate_position(number)` and `opponent_position(number)` with explicit `error(...)` on unknown squad number; add a code comment that squad numbers are a provisional player identifier pending the DSL "player" type
- [ ] 2.6 Add `ball_owner_number()` (returns `nil` when nobody possesses the ball, including an in-flight pass; code comment on the missing "nothing" type in the DSL)
- [ ] 2.7 Add `is_teammate_free(number)`, `is_opponent_free(number)`, and `is_teammate_ahead/behind/left/right(number)`, `is_opponent_ahead/behind/left/right(number)` per design decision 8
- [ ] 2.8 Add `kick_to_position(pos, reason?)` and temporary `default_goalkeeper_with_ball()`

## 3. Team Migration

- [ ] 3.1 `teams/team_a/preamble.lua` and `teams/team_b/preamble.lua`: replace `am_i_top3_closest_to_ball()` with the loop-free formula `count_teammates_around(ball_position(), distance(my_position(), ball_position())) < 3` (no behavior change)
- [ ] 3.2 Both teams: replace `pass_to_nearest_teammate` in `goalkeeper_play.i_have_ball` with `default_goalkeeper_with_ball` and delete the local helper (no behavior change)
- [ ] 3.3 `team_a`: replace `pass_to_numbers({6, 7, 8})` / `({9, 10, 11})` with sequential `if` distance comparisons choosing the nearest listed teammate and `kick_to_position(teammate_position(n), "pass_to_#" .. n)` (no behavior change in practice)
- [ ] 3.4 `team_b` `nonforward_with_ball`: replace raw `.z` comparisons with `is_teammate_ahead(10)` / `is_teammate_ahead(11)` and `kick_to_position(teammate_position(n), "pass_to_#" .. n)` (behavior change: receiver must be more than 0.5 m ahead)
- [ ] 3.5 `team_b` `forward_with_ball`: drop the `my_regions()["attack"]` / `is_in_region_obj` branch; shoot in the opponent penalty area, otherwise `run_to_opponent_penalty_area()` (behavior change: no shot from the attack region)
- [ ] 3.6 Verify `team_a` / `team_b` call only user-facing API functions and contain no player loops, coordinate reads, or `{action=...}` tables

## 4. Tests and Documentation

- [ ] 4.1 Update `ynwa-script-tests/fixtures/team_a.lua`, `fixtures/team_b.lua`, and `scenarios/goal_kick_team*` to stop calling removed functions (`run_to_start_position`, `kick_to_cell`, `kick_to_region`)
- [ ] 4.2 Update `ynwa-integration-testing/scenarios/pass_*/teams/*/players/01/script.lua` to `kick_to_position(teammate_position(2), "pass_to_#2")`
- [ ] 4.3 Update `ynwa-script-tests/tests/stdlib_functions.rs`: remove direct tests of internal/removed helpers, keep their behavior covered via public functions, add tests for all new public functions (including boundary cases: strict radius, 0.5 m direction tolerance, unknown numbers, in-flight `ball_owner_number`), and assert internal helpers are `nil` in the team-preamble scope
- [ ] 4.4 Update `context/user-docs/scripting_player.md` to document exactly the user-facing API whitelist
- [ ] 4.5 Run `cargo test --workspace`, `cargo clippy`, `cargo fmt --check`, and a short `ynwa-simulator` match `team_a` vs `team_b` without Lua runtime errors
