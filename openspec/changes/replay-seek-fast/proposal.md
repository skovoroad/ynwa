# Proposal

## Why

In `Replay` mode the player can pause and change playback speed, but cannot scrub forward or backward through the recording. Re-watching a moment requires restarting from the beginning or waiting at high speed. Adding a fixed 5-second seek backed by periodic in-memory state checkpoints keeps backward seeks `O(1)` regardless of recording length, while avoiding full-record cloning and duplicate event-list allocations.

## What Changes

- Add a fixed 5-second forward/backward seek in `Replay` mode of `ynwa-player`, bound to `Right`/`Left` arrow keys and clamped to `[0, total_steps]`.
- Forward seek advances the active replay world in place; backward seek restores from the nearest prior in-memory `GameState` checkpoint and fast-forwards only the remaining steps (`<= seek_steps`).
- Add minimal state-resumption constructors in `ynwa-core` (`PhysicsSystem::from_timestamp`, `ReplayDriver::from_index`) and `ynwa-football` (`create_football_replay_world_from_state`) so a replay world can resume cleanly from a checkpoint without re-simulating from step `0`.
- Replace the cloned `events` / `visible_events` buffers in `run_replay` with a zero-copy slice `&record.journal[..event_cursor]` updated via `partition_point` on seek, and centralize seek key polling in `ynwa-player/src/input.rs`.

## Capabilities

### New Capabilities

<!-- none -->

### Modified Capabilities

- `player-client`: extend the replay keyboard controls and pacing requirements to define fixed-step seeking and the resulting events-panel behavior.

## Impact

- `ynwa-core/src/systems/physics.rs` — add `PhysicsSystem::from_timestamp` so resumed worlds preserve `last_update`.
- `ynwa-core/src/replay.rs` — add `ReplayDriver::from_index` so resumed worlds start at the matching journal cursor.
- `ynwa-football/src/replay.rs` — extract `create_football_replay_world_from_state` shared with `create_football_replay_world`.
- `ynwa-player/src/simulation.rs` — expose `steps_done()` and `reposition(step)`.
- `ynwa-player/src/input.rs` — poll `Left`/`Right` seek actions alongside existing controls.
- `ynwa-player/src/main.rs` — maintain periodic `GameState` checkpoints, execute forward/backward seeks, and render events directly from `&record.journal[..event_cursor]`.

