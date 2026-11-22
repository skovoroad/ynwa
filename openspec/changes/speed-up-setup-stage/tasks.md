# Tasks

## 1. Time-scale in SimulationControl

- [ ] 1.1 Add a `speed` field (default `1.0`) and a `set_speed` method to `SimulationControl` in `ynwa-player/src/simulation.rs`, and make `accumulate` add `delta_time * speed` instead of `delta_time`. Verify `cargo build -p ynwa-player` succeeds.
- [ ] 1.2 Add `simulation_tests.rs` next to `simulation.rs` covering that `accumulate` scales the accumulated frame time by `speed` (e.g. after `set_speed(4.0)`, `accumulate(0.5)` leaves the accumulator at `2.0`). Verify `cargo test -p ynwa-player` passes.

## 2. Stage-driven speed selection

- [ ] 2.1 Add `const SETUP_SPEEDUP: f32 = 4.0;` and a pure `fn setup_speed(stage: &GameStage) -> f32` in `main.rs` that returns `SETUP_SPEEDUP` for `GameStage::Setup(_)` and `1.0` for `GameStage::Play`, with unit tests for both branches in `main_tests.rs`. Verify `cargo test -p ynwa-player` passes.
- [ ] 2.2 In `advance`, call `simulation.set_speed(setup_speed(&world.game().state().stage))` before `simulation.accumulate(get_frame_time())`. Verify `cargo build -p ynwa-player` succeeds and `cargo clippy -p ynwa-player` reports no warnings.

## 3. Integration check

- [ ] 3.1 Run `cargo test`, `cargo clippy`, and `cargo fmt --check` from the repository root and confirm they all pass with no warnings.
