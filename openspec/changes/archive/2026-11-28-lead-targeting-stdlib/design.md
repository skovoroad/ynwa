# Design

## Context

Players make tactical decisions discretely every `0.5–3.0` s (`PlayerReactionSystem`). Currently, `chase_ball` aims at the ball's snapshot position and teammate passes use `kick_to_position(teammate_position(n))`, causing players to chase behind a rolling ball and passes to fall behind moving teammates.

To keep user tactics and the future DSL simple and stat-agnostic, lead-target computation belongs in `stdlib.lua` using only **observable kinematics** (positions and current velocities) and **averaged** physical constants, rather than exposing player `speed_rate` or `shot_power` to scripts.

## Goals / Non-Goals

- **Goals**:
  - Expose `velocity: {x, z}` in team orientation for `ball`, `me`, `teammates`, and `opponents` in `context`.
  - Provide closed-form $O(1)$ linear lead targeting in `stdlib.lua` for both `chase_ball(reason?)` and `pass_to(number, reason?)`.
  - Clamp computed lead targets to `GAME_DATA.field` boundaries.
  - Switch existing `teams/team_a..d/preamble.lua` teammate passes to `pass_to`.
  - Verify end-to-end lead kinematics in `ynwa-integration-testing`.
  
- **Non-Goals**:
  - Exposing `speed_rate`, `shot_power`, or iterative root-finding in Lua.
  - Modifying `Decision::Run` or `Decision::Kick` variants in `ynwa-core` (handled separately in `remove-ball-decision-target`).

## Decisions

### 0. `LuaExecutor` builder for pre-preamble globals (`ynwa-decisions`)
Currently `LuaExecutor::new(preamble, timeout)` runs `preamble` immediately during construction, while `DecisionEngine::new` calls `executor.set_global("GAME_DATA", data)` only afterwards. Top-level preamble code (such as `stdlib.lua` deriving `AVG_*` constants from `GAME_DATA.physics`) therefore fails with `attempt to index a nil value (global 'GAME_DATA')`.

To keep `LuaExecutor` domain-agnostic (without hardcoding `"GAME_DATA"`) and leave `LuaExecutor::new` untouched for existing tests:
- Add a generic `LuaExecutorBuilder` (e.g., `LuaExecutor::builder()` / `LuaExecutor::with_globals(...)` -> `.with_preamble(...)` / `.with_timeout(...)` -> `.build()`) that creates the `Lua` VM, applies the sandbox, installs all configured globals, and **then** executes the preamble.
- Update `DecisionEngine::new` to construct each player's `LuaExecutor` via this builder, passing `("GAME_DATA", data)` before `combined_preamble`.

### 1. Velocity in `context` (2D XZ in team orientation)
`ScriptedDecisionMaker::build_context` serializes `velocity: {x, z}` alongside `position` for `me`, `teammates`, `opponents`, and `ball`. Just like `position` (which flips `x -> width - x`, `z -> length - z` for `Team::B`), velocity vectors for `Team::B` negate both components (`vx -> -vx`, `vz -> -vz`).

### 2. Engine physics in `GAME_DATA.physics` + dimensionless heuristics in `stdlib.lua`
To keep `stdlib.lua` synchronized with `ynwa-core` if physics tuning changes, `ScriptedDecisionMaker` serializes `GAME_DATA.physics` once at VM initialization directly from the Rust constants (`MAX_SPEED_METERS_PER_SECOND`, `kick_speed(100)`, `BALL_FRICTION_DECELERATION`):
```json
"physics": {
  "max_player_speed": 10.0,
  "max_kick_speed": 20.0,
  "ball_friction": 2.0
}
```
`stdlib.lua` derives its internal `local` averaged speeds from `GAME_DATA.physics` using dimensionless heuristic ratios rather than hardcoded physical numbers:
- `AVG_PLAYER_SPEED = GAME_DATA.physics.max_player_speed * 0.7`
- `AVG_PASS_SPEED = GAME_DATA.physics.max_kick_speed * 0.6`
- `BALL_FRICTION = GAME_DATA.physics.ball_friction`
- `MAX_LEAD_TIME = 1.5` s (prevents extrapolating linear motion too far ahead)
- `PASS_FIELD_MARGIN = 1.0` m (keeps pass targets inside the touchline/goal line)

This preserves the gameplay value of `speed_rate` and `shot_power` (faster players arrive sooner; stronger passers kick sharper), avoids coupling per-tick `context` to hidden player attributes, and prevents physics drift between Rust and Lua.

### 3. Closed-form lead formulas
- **`chase_ball(reason?)`**:
  - `ball_speed = sqrt(vx^2 + vz^2)`
  - If `ball_speed == 0`, target is `ball_position()`.
  - Otherwise `t_stop_half = 0.5 * ball_speed / BALL_FRICTION` (exact midpoint time giving the ball's stopping distance under constant deceleration).
  - `t_est = min(distance(my_pos, ball_pos) / AVG_PLAYER_SPEED, t_stop_half, MAX_LEAD_TIME)`
  - `target = clamp_to_field(ball_pos + ball_vel * t_est, 0.0)`
  - Returns `{action = "run", target_type = "point", target = {x = target.x, z = target.z, y = 0}, reason = compose_reason(reason, "chase_ball")}`.
- **`pass_to(number, reason?)`**:
  - `t_est = min(distance(my_pos, tm_pos) / AVG_PASS_SPEED, MAX_LEAD_TIME)`
  - `target = clamp_to_field(tm_pos + tm_vel * t_est, PASS_FIELD_MARGIN)`
  - Delegates to `kick_to_position(target, compose_reason(reason, "pass_to_#" .. number))`.

### 4. Integration test isolation
Symmetric (`Team::A` and `Team::B`) deterministic (`temperature = 0.0`) scenarios in `ynwa-integration-testing` verify each capability independently and separate lead-math bugs from orientation-flip bugs:
- `lead_pass_moving_teammate_team_a` / `lead_pass_moving_teammate_team_b`: the receiver runs across the field to a distant `attack` target under `team_has_ball` without calling `chase_ball`, while the ball owner calls `pass_to`. Because `last_possessing_team` stays with the passing team during flight and `Kick` does not trigger `needs_decision` for teammates, reception (`PossessionChange`) succeeds only if the lead pass intersects the receiver's running path within `POSSESSION_RADIUS` (`1.0` m) after `POSSESSION_COOLDOWN` (`1.0` s).
- `chase_moving_ball_team_a` / `chase_moving_ball_team_b`: a free rolling ball moves across the pitch while a player with `reaction_rate = 10` (`3.0` s interval) receives a single `chase_ball()` decision and intercepts the ball (`PossessionChange`) before the next decision tick.

## Risks / Trade-offs

- **Loss of live arrival check for `chase_ball`**: Because `chase_ball` emits `target_type = "point"` instead of `"ball"`, `DecisionSystem` checks arrival against the predicted point `T` rather than the live ball position. In practice, `BallPossessionSystem` still claims the ball within `1.0` m and triggers `needs_decision = true` on possession change.
- **Regression in existing scenarios**: Integration scenarios or script tests asserting `DecisionTarget::Ball` or exact ball-chase trajectories must be checked; `basic_scripts.rs` tests for raw `target_type = "ball"` still pass until `remove-ball-decision-target` is applied.

