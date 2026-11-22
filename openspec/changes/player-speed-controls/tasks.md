# Tasks

## 1. SimulationControl speed model

- [ ] 1.1 Store the fixed `timestep` directly in `SimulationControl` (drop the `rate` field) and make `pace()` return `timestep`; update `new`/`for_replay` to set `timestep` rather than `rate`. Verify `cargo build -p ynwa-player` succeeds.
- [ ] 1.2 Make `speed` the user-adjustable value: keep `speed` (default `1.0`), add `speed()`, `increase_speed()`, and `decrease_speed()` (double/halve, clamped to `0.25`–`8.0`); remove `increase_rate`, `decrease_rate`, and `set_rate`; make `accumulate` a pure adder (no internal speed scaling). Verify `cargo build -p ynwa-player` succeeds.
- [ ] 1.3 Add `simulation_tests.rs` covering `increase_speed`/`decrease_speed` doubling/halving and clamping at both bounds, and that `accumulate` adds its argument unchanged. Verify `cargo test -p ynwa-player` passes.

## 2. Keyboard bindings

- [ ] 2.1 In `handle_input`, bind `+`/`=`/`KP_ADD` and `-`/`KP_SUBTRACT` to `increase_speed`/`decrease_speed` and remove the `rate_locked` parameter (and its call-site arguments). Verify `cargo build -p ynwa-player` succeeds.

## 3. Effective speed in `advance`

- [ ] 3.1 In `advance`, replace the `setup_speed`/`set_speed` call with an inline effective speed (`SETUP_SPEEDUP` for `GameStage::Setup`, otherwise `simulation.speed()`) and call `simulation.accumulate(get_frame_time() * effective)`. Verify `cargo build -p ynwa-player` succeeds and `cargo clippy -p ynwa-player` reports no warnings.

## 4. Integration check

- [ ] 4.1 Run `cargo test`, `cargo clippy`, and `cargo fmt --check` from the repository root and confirm they all pass with no warnings.
