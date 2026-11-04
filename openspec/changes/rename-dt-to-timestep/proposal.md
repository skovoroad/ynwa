# Proposal

## Why

The fixed model-time quantum per `World::step` is named three different ways across the system —
`dt` in `scenario.toml` `[run]` and the integration-test DTOs, `fixed_dt` in `RecordHeader` and the
recording header, `step_delta` in the player, and `delta_time` in `World::step`/`Game::step`. Those
names collide with two *other* quantities — the physics-internal computed delta and the player's
real-time tempo — which is the source of the `dt`/`fixed_dt`/`step_delta` confusion.

## What Changes

Rename the first quantity (the fixed per-step quantum) to the standard discrete-simulation term
`timestep` everywhere it appears. The other two quantities keep their names.

Serialization (both **BREAKING** on-disk format changes):

- `scenario.toml` `[run]` field `dt: f32` becomes `timestep: f32`.
- Recording JSONL header field `"fixed_dt"` becomes `"timestep"`.

Code identifiers:

- `RunPlanDto.dt` → `RunPlanDto.timestep` (including its validation message).
- Runner/comparison `dt` parameters → `timestep`.
- `RecordHeader.fixed_dt` and `RecordHeader::from_game(game, fixed_dt)` → `timestep`.
- `World::step(&mut self, delta_time)` and `Game::step(&mut self, delta_time)` → `timestep`.
- Player `SimulationControl::step_delta` field/accessor and `for_replay(fixed_dt, ..)` →
  `timestep` / `for_replay(timestep, ..)`.
- `ynwa-football` test helper `const FIXED_DT` → `const TIMESTEP`.

Out of scope, explicitly left unchanged:

- `delta_time` inside `PhysicsSystem` (the computed `timestamp - last_update` delta).
- The player's real-time tempo terms `rate`, `pace()`, and `accumulate(delta_time)`.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `integration-testing`: `scenario.toml` `[run]` `dt` field and its validation/tolerance references become `timestep`.
- `core-simulation`: `World::step(dt)` signature becomes `World::step(timestep)`.
- `player-client`: `fixed_dt` / `step_delta` (simulation step) references become `timestep`.
- `architecture`: `World::step(..., delta_time)` signature becomes `World::step(..., timestep)`.
- `recording-and-replay`: `RecordHeader.fixed_dt` / `from_game(game, fixed_dt)` references become `timestep`.

## Impact

- **BREAKING** on-disk scenario format: existing `scenario.toml` files with `dt =` will no longer
  parse because the runner rejects unknown fields (`deny_unknown_fields`).
- **BREAKING** on-disk recording format: existing `.jsonl` recordings with a `"fixed_dt"` header
  field will no longer deserialize.
- Source: `ynwa-integration-testing`, `ynwa-core`, `ynwa-player`, `ynwa-football` (test utils only).
- No runtime behavior, physics, or replay semantics change; this is a pure rename.
