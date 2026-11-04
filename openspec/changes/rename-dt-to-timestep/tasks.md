# Tasks

## 1. Integration-testing DTO, runner, and scenario schema

- [ ] 1.1 Rename `RunPlanDto.dt` → `RunPlanDto.timestep` in `ynwa-integration-testing/src/dto.rs` (field, `validate()`, and the `run.dt` error message); verify with `cargo build -p ynwa-integration-testing`
- [ ] 1.2 Rename the `dt` parameter → `timestep` in `runner.rs` (`run` body and doc comments) and `compare.rs` (`compare_outcome`, `compare_journal`, `entry_matches`, `entry_timestamp_matches`, `timestamps_match`, plus the module doc comment); verify with `cargo build -p ynwa-integration-testing`
- [ ] 1.3 Update `[run] dt` → `[run] timestep` in the six `scenarios/*/scenario.toml` files and the `tests/fixtures/{valid,failing}/scenario.toml` fixtures; verify with `cargo test -p ynwa-integration-testing`

## 2. Core recording and step API

- [ ] 2.1 Rename `RecordHeader.fixed_dt` → `RecordHeader.timestep` and `RecordHeader::from_game(game, fixed_dt)` → `from_game(game, timestep)` in `ynwa-core/src/record.rs`; update assertions on `header.fixed_dt` in `ynwa-core` tests; verify with `cargo test -p ynwa-core`
- [ ] 2.2 Rename `World::step(&mut self, delta_time)` → `World::step(&mut self, timestep)` in `ynwa-core/src/world.rs` and `Game::step(&mut self, delta_time)` → `Game::step(&mut self, timestep)` in `ynwa-core/src/game.rs`; leave `PhysicsSystem`'s internal `delta_time` unchanged; verify with `cargo test -p ynwa-core`

## 3. Football test helpers

- [ ] 3.1 Rename `const FIXED_DT` → `const TIMESTEP` in `ynwa-football/src/test_utils.rs` and update all references in `journal_integration_tests.rs`, `lockstep_tests.rs`, `replay_tests.rs`, and `serialization_tests.rs`; also update the `record.header.fixed_dt` prose in `ynwa-football/src/replay.rs` → `record.header.timestep`; verify with `cargo test -p ynwa-football`

## 4. Player simulation control

- [ ] 4.1 Rename `SimulationControl.step_delta` field and `step_delta()` accessor → `timestep`/`timestep()`, and `for_replay(fixed_dt, ..)` → `for_replay(timestep, ..)` in `ynwa-player/src/simulation.rs`; keep `rate`, `pace()`, and `accumulate(delta_time)` unchanged; verify with `cargo build -p ynwa-player`
- [ ] 4.2 Rename local `fixed_dt` → `timestep` and `step_delta()` → `timestep()` in `ynwa-player/src/main.rs` (`run_record`, `run_replay`, `replay_steps`, `advance`); verify with `cargo build -p ynwa-player`

## 5. Verification and residual sweep

- [ ] 5.1 Grep the workspace for residual `fixed_dt`, `step_delta`, `FIXED_DT`, and step-quantum `dt`/`delta_time`, and update the `recording-and-replay/spec.md` Purpose prose that names `RecordHeader { config, initial_state, fixed_dt }`; verify the grep returns no remaining step-quantum references
- [ ] 5.2 Run `cargo fmt`, `cargo clippy --all-targets --all-features` (no warnings), and the full `cargo test`; verify all three pass
