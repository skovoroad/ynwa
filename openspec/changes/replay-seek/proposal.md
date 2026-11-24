# Proposal

## Why

In `Replay` mode the player can already pause, speed up, and slow down playback, but cannot jump to a different point in the recording. Re-watching a specific moment (a goal, a contested ball, a tactical sequence) requires replaying from the start or waiting at high speed. A fixed seek step lets the user scrub the recording forward and backward without leaving the current replay.

## What Changes

- Add a fixed forward/backward seek in `Replay` mode of `ynwa-player`.
- A seek moves the replay position by a fixed 5 seconds of match time, clamped to `[0, total_steps]`.
- Forward seek advances the already-running replay world by the target number of steps (no rebuild).
- Backward seek rebuilds the replay world from the recorded header and re-simulates physics-only up to the target step (no snapshots or checkpoints).
- Bind seek to the `Left`/`Right` arrow keys (currently unused for the events panel; `Up`/`Down`/`PgUp`/`PgDown` remain for event scrolling).
- After a seek, the `Events:` panel re-derives the visible journal entries for the new `elapsed_time`.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `player-client`: extend the replay keyboard controls and pacing requirements to define fixed-step seeking and the resulting events-panel behavior.

## Impact

- `ynwa-player/src/main.rs` — keep the `Record` available for rebuild, implement seek, recompute the visible events after seeking.
- `ynwa-player/src/simulation.rs` — expose current step count and a way to set it/reset pacing after a seek.
- `ynwa-player/src/input.rs` — bind `Left`/`Right` to seek.
- No changes to `ynwa-core` or `ynwa-football`: seek reuses the existing `create_football_replay_world` re-simulation path and its determinism guarantee.
