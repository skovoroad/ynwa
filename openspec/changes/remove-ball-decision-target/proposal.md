# Proposal

## Why

Once `chase_ball(reason?)` in `stdlib.lua` computes its own lead point and returns `target_type = "point"`, `DecisionTarget::Ball` and `target_type = "ball"` become unused dead code and remain the only sport-specific variant in `DecisionTarget`. Removing them keeps `ynwa-core` sport-agnostic and simplifies decision parsing, movement resolution, and arrival checks.

Blocked by: `lead-targeting-stdlib`.

## What Changes

- **BREAKING** (internal decision schema): Remove `DecisionTarget::Ball` from `ynwa-core` (`game.rs`, `systems/movement.rs`, `systems/decision/mod.rs`, `systems/decision/decision_parser.rs`, `systems/decision/util.rs`).
- Remove `target_type = "ball"` handling from `parse_decision` in `ynwa-core/src/systems/decision/decision_parser.rs`.
- Remove the `&BallState` parameter from `resolve_target_point` in `ynwa-core/src/systems/decision/util.rs`.
- Remove the `DecisionTarget::Ball` match arm from `ynwa-player/src/ui.rs`.
- Update unit and integration test fixtures in `ynwa-core`, `ynwa-football`, and `ynwa-script-tests` that previously constructed `Decision::Run(DecisionTarget::Ball)` or tested `target_type = "ball"`.

## Capabilities

### New Capabilities

### Modified Capabilities
- `core-simulation`: Remove `DecisionTarget::Ball` from `DecisionSystem` arrival check and `ActionSystem` run target resolution.
- `scripting-and-protocols`: Remove `target_type = "ball"` from the `LuaDecision` return schema.

## Impact

- `ynwa-core/src/game.rs`, `ynwa-core/src/systems/movement.rs`, `ynwa-core/src/systems/decision/*`
- `ynwa-player/src/ui.rs`
- `ynwa-core/src/tests/*`, `ynwa-football/src/*`, `ynwa-script-tests/tests/basic_scripts.rs`
- `openspec/specs/core-simulation/spec.md`, `openspec/specs/scripting-and-protocols/spec.md`

