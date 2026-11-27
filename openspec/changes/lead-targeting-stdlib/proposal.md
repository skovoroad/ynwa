# Proposal

## Why

Players make decisions discretely at `0.5–3.0` s intervals, while `chase_ball` and teammate passes currently target static snapshot positions. As a result, players chase behind a rolling ball and passes land behind moving teammates. Computing simple linear lead targets in `stdlib.lua` from observed velocities and averaged player/pass speeds fixes both issues while keeping user tactics compact and hiding player stats from scripts.

## What Changes

- Expose observed `velocity` (`{x, z}` in the caller's team orientation) on `context.ball`, `context.teammates[i]`, `context.opponents[i]`, and `context.me`, and expose engine physics constants (`max_player_speed`, `max_kick_speed`, `ball_friction`) in `GAME_DATA.physics` in `ScriptedDecisionMaker`.
- Add `INTERNAL` Layer 1 velocity wrappers (`ball_velocity()`, `teammate_velocity(number)`, `opponent_velocity(number)`, `my_velocity()`) to `ynwa-scripts/preambles/core.lua`.
- Update `chase_ball(reason?)` in `ynwa-scripts/preambles/stdlib.lua` to compute a linear lead point using an averaged player speed, `ball_velocity()`, and ball friction deceleration, clamped to `GAME_DATA.field` bounds, returning `{action = "run", target_type = "point", ...}`.
- Add `pass_to(number, reason?)` to `ynwa-scripts/preambles/stdlib.lua` to compute a linear lead point using an averaged pass speed and `teammate_velocity(number)`, clamped to `GAME_DATA.field` bounds, and delegate to `kick_to_position` with reason `"pass_to_#<number>"` (or `"<custom> | pass_to_#<number>"`).
- Update `default_goalkeeper_with_ball(reason?)` in `stdlib.lua` to use `pass_to(best_number, reason)`.
- Replace `kick_to_position(teammate_position(n), ...)` pass calls in `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`, `teams/team_c/preamble.lua`, and `teams/team_d/preamble.lua` with `pass_to(n, ...)` (preserving custom reasons).
- Add unit tests in `ynwa-core` and `ynwa-script-tests`, and add symmetric `Team::A`/`Team::B` end-to-end scenarios (`lead_pass_moving_teammate_team_a`/`_b` and `chase_moving_ball_team_a`/`_b`) in `ynwa-integration-testing`.

## Capabilities

### New Capabilities

### Modified Capabilities
- `scripting-and-protocols`: Add `velocity` to `context` schema, internal velocity wrappers to `core.lua`, lead-targeted `chase_ball(reason?)` and new `pass_to(number, reason?)` to `stdlib.lua`.

## Impact

- `ynwa-core/src/systems/decision/scripted_decision_maker.rs`
- `ynwa-scripts/preambles/core.lua`
- `ynwa-scripts/preambles/stdlib.lua`
- `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`, `teams/team_c/preamble.lua`, `teams/team_d/preamble.lua`
- `ynwa-script-tests/tests/stdlib_functions.rs`, `ynwa-script-tests/tests/team_orientation.rs`
- `ynwa-integration-testing/scenarios/` (new `lead_pass_moving_teammate_team_a`/`_b` and `chase_moving_ball_team_a`/`_b` scenarios + regression check)
- `openspec/specs/scripting-and-protocols/spec.md`

