# Proposal

## Why

During the `Setup` stage (kick-off, throw-in, corner, goal kick) the player advances the simulation in real time while players merely run to their set-piece positions. This wall-clock wait is wasted time for anyone watching, recording, or replaying a match.

## What Changes

- Add a fixed `4x` time-scale that applies only while the game is in `GameStage::Setup`, so the setup stage completes in about a quarter of the wall-clock time.
- Keep the simulation `timestep` unchanged: only the wall-clock pacing changes. The simulation, the recorded journal, and replay determinism are byte-for-byte identical.
- Apply the acceleration in all three player modes (`Play`, `Record`, `Replay`).
- Hardcode the `4x` factor (no new CLI flag) and show no on-screen indicator; players are not teleported.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player-client`: the simulation pacing behavior gains a Setup-stage `4x` time-scale that changes wall-clock speed without changing the simulation step.

## Impact

- `ynwa-player/src/simulation.rs`: `SimulationControl` gains a wall-clock time-scale (speed multiplier).
- `ynwa-player/src/main.rs`: the `Play`/`Record`/`Replay` loops and `advance` select the speed from `world.game().state().stage`.
- No changes to `ynwa-core`, `ynwa-football`, the recording format, or replay semantics.
