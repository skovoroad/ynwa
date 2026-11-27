# Tasks

## 1. Expose Observed Velocities and Physics Constants

- [ ] 1.1 Update `ScriptedDecisionMaker::build_context` (and `build_team_positions`) in `ynwa-core/src/systems/decision/scripted_decision_maker.rs` to serialize `velocity: {x, z}` in the caller's team orientation (`x` and `z` negated for `Team::B`) on `me`, `teammates[*]`, `opponents[*]`, and `ball`.
- [ ] 1.2 Expose `GAME_DATA.physics` (`max_player_speed`, `max_kick_speed`, `ball_friction`) in `ScriptedDecisionMaker` directly from `ynwa-core` constants (`MAX_SPEED_METERS_PER_SECOND`, `kick_speed(100)`, `BALL_FRICTION_DECELERATION`).
- [ ] 1.3 Add unit tests in `ynwa-core/src/systems/decision/scripted_decision_maker.rs` verifying `context.*.velocity` presence, `Team::A`/`Team::B` sign flipping, and `GAME_DATA.physics` values.

## 2. Layer 1 & Layer 2 Lua Lead Targeting

- [ ] 2.1 Add `INTERNAL` velocity wrappers `my_velocity()` and `ball_velocity()` to `ynwa-scripts/preambles/core.lua` (keeping the user-facing whitelist unchanged).
- [ ] 2.2 Add internal `local` constants derived from `GAME_DATA.physics` (`AVG_PLAYER_SPEED`, `AVG_PASS_SPEED`, `BALL_FRICTION`, `MAX_LEAD_TIME`, `PASS_FIELD_MARGIN`) and helpers (`clamp_to_field`, `teammate_velocity`, `opponent_velocity`) inside the `do ... end` block of `ynwa-scripts/preambles/stdlib.lua`.
- [ ] 2.3 Update `chase_ball(reason?)` in `ynwa-scripts/preambles/stdlib.lua` to compute a clamped linear lead target and return `{action = "run", target_type = "point", ...}`.
- [ ] 2.4 Add `pass_to(number, reason?)` to `ynwa-scripts/preambles/stdlib.lua` and update `default_goalkeeper_with_ball(reason?)` to call `pass_to(best_number, reason)`.

## 3. Switch Team Preambles to `pass_to`

- [ ] 3.1 Replace `kick_to_position(teammate_position(number), ...)` pass calls in `teams/team_a/preamble.lua` with `pass_to(number, ...)`.
- [ ] 3.2 Replace `kick_to_position(teammate_position(10/11), ...)` pass calls in `teams/team_b/preamble.lua` with `pass_to(10/11, ...)`.
- [ ] 3.3 Replace `kick_to_position(teammate_position(...), ...)` pass calls in `teams/team_c/preamble.lua` with `pass_to(..., ...)`.
- [ ] 3.4 Replace `kick_to_position(teammate_position(...), ...)` pass calls in `teams/team_d/preamble.lua` with `pass_to(..., ...)` (preserving custom reasons such as `"pass to 9"`).

## 4. Verification

- [ ] 4.1 Add tests in `ynwa-script-tests/tests/stdlib_functions.rs` and `ynwa-script-tests/tests/team_orientation.rs` covering stationary/moving ball `chase_ball`, stationary/moving teammate `pass_to`, field clamping, custom reasons, unknown teammate error, and `Team::A`/`Team::B` symmetry.
- [ ] 4.2 Add symmetric `Team::A`/`Team::B` scenarios in `ynwa-integration-testing/scenarios/`: `lead_pass_moving_teammate_team_a`/`_b` (receiver running without `chase_ball`) and `chase_moving_ball_team_a`/`_b` (`reaction_rate = 10`) and verify all four pass.
- [ ] 4.3 Run `cargo test` across the workspace (including `ynwa-script-tests` and `ynwa-integration-testing`) and `cargo clippy` to confirm all checks pass without warnings.

