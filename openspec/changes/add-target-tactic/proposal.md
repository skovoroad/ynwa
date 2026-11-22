# Proposal

## Why

`teams/team_a` and `teams/team_b` are minimal demonstration teams. To validate the updated `stdlib.lua` API in full-match simulations and establish the first-stage target tactic described in `context/vision/tactics.md`, we need a dedicated target team implemented strictly through the public `stdlib.lua` interface.

## What Changes

- Create a new team directory (`teams/team_c`) with `meta.toml`, 11 players (`static.toml`, `tactical.toml`, and `script.lua`), and `preamble.lua`.
- Implement the target roles from `context/vision/tactics.md` in `teams/team_c`:
  - **Team fallback**: attack position when team has ball; press ball if among the 3 closest teammates (`count_teammates_around(ball_position(), distance(my_position(), ball_position())) < 3`) or defence position when opponent has ball / ball is free.
  - **Goalkeeper**: `default_goalkeeper_with_ball()` and `default_goalkeeper_cover_position()` for all non-ball-owning states (`team_has_ball`, `opponent_has_ball`, `ball_is_free`).
  - **Fullbacks**: dribble to opponent penalty area region and pass to a free teammate from a designated list, or to the nearest non-free teammate from the same list if none is free.
  - **Center backs**: pass to the nearest free teammate ahead from a designated list using manual `if` comparisons (`is_teammate_free`, `is_teammate_ahead`, `distance`, `kick_to_position(teammate_position(n), ...)`), or fallback to `run_to_attack_position()`.
  - **Wide and central midfielders**: pass to a free teammate ahead from prioritized lists, or advance to opponent penalty area and shoot.
  - **Forwards**: advance to opponent penalty area and shoot (`kick_to_opponent_goal()`).

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
<!-- None (adds team content conforming to `TeamRepository` and `scripting-and-protocols`; `skip_specs: true`) -->

## Impact

- `teams/team_c/` (new team configuration and Lua scripts)
- Depends on `update-lua-stdlib`.

