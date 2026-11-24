# Tasks

## 1. Position control in SimulationControl

- [ ] 1.1 Add a `steps_done()` accessor and a `reposition(step)` method to `SimulationControl` that clamps `step` to `[0, max_steps]` (when `max_steps` is set), assigns `steps_done`, and resets `accumulator` to `0.0`; verify `cargo build -p ynwa-player` succeeds.
- [ ] 1.2 Extend `simulation_tests.rs` to cover `reposition` clamping at the upper bound, accumulator reset after reposition, and the `max_steps == None` (no clamp) case; verify `cargo test -p ynwa-player simulation` passes.

## 2. Replay seek orchestration

- [ ] 2.1 In `run_replay`, keep the `Record` alive and pass `record.clone()` to `create_football_replay_world` so it can be rebuilt for backward seek; verify replay still loads with `cargo build -p ynwa-player`.
- [ ] 2.2 Compute `seek_steps = round(5.0 / timestep)` once and implement forward seek (tight-loop `world.step` to the clamped target, then `reposition`) on the `Right` key; verify a manual replay advances ~5 match-seconds per press.
- [ ] 2.3 Implement backward seek (rebuild the world from `record.clone()`, fast-forward to the clamped target, then `reposition`) on the `Left` key; verify a manual replay returns to an earlier position and reproduces the same on-screen state.
- [ ] 2.4 Re-derive the `Events:` panel state (`event_cursor`, `visible_events`, scroll offset) for the new `elapsed_time` after either seek direction; verify the panel reflects the seeked-to position and auto-follows.

## 3. Integration verification

- [ ] 3.1 Run `cargo fmt --check`, `cargo clippy --workspace`, and `cargo test -p ynwa-player`; verify all pass with no warnings.

## Workflow follow-up

- Archive the change after the project's review requirements are satisfied.
