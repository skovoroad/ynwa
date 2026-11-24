# Tasks

## 1. Checkpoint resumption primitives in ynwa-core and ynwa-football

- [x] 1.1 Add `PhysicsSystem::from_timestamp(last_update: f32)` in `ynwa-core/src/systems/physics.rs` and `ReplayDriver::from_index(journal: Vec<JournalEntry>, next_index: usize)` in `ynwa-core/src/replay.rs`; add unit tests in `physics_tests.rs` and `replay_tests.rs` verifying resumed stepping matches a continuous run from step `0` (`cargo test -p ynwa-core`).
- [x] 1.2 Extract `create_football_replay_world_from_state(config: GameConfig, state: GameState, journal: Vec<JournalEntry>, next_index: usize) -> Result<World, String>` in `ynwa-football/src/replay.rs` (sharing implementation with `create_football_replay_world`); add a test in `ynwa-football/src/tests/replay_tests.rs` confirming a world resumed mid-match produces the same final state as a full replay (`cargo test -p ynwa-football replay`).

## 2. Position control and seek input in ynwa-player

- [x] 2.1 Add `steps_done()` and `reposition(step)` to `SimulationControl` in `ynwa-player/src/simulation.rs` (clamping to `[0, max_steps]` when set and resetting `accumulator` to `0.0`); extend `simulation_tests.rs` to cover clamping, accumulator reset, and `max_steps == None` (`cargo test -p ynwa-player simulation`).
- [x] 2.2 Add `SeekDirection` (`Backward`, `Forward`) and `poll_replay_seek() -> Option<SeekDirection>` in `ynwa-player/src/input.rs` bound to `Left`/`Right` key presses; verify `cargo build -p ynwa-player` succeeds.

## 3. Checkpoint-backed replay seek and zero-copy events in run_replay

- [x] 3.1 Keep `record` alive in `run_replay` (`ynwa-player/src/main.rs`), replace the cloned `events` and `visible_events` vectors with `event_cursor` + `&record.journal[..event_cursor]`, and update `event_cursor` via `partition_point` on seek with scroll reset to `0`; verify `cargo build -p ynwa-player` succeeds.
- [x] 3.2 Maintain `checkpoints: Vec<(u64, GameState)>` every `seek_steps = round(5.0 / timestep)` steps in `run_replay`, implement forward seek (step in place, recording crossed checkpoints) and backward seek (resume from nearest prior checkpoint via `create_football_replay_world_from_state`, fast-forward remaining `<= seek_steps` steps, and `reposition`); verify manual `Right`/`Left` seeking advances/rewinds ~5 match-seconds and reproduces the same state.

## 4. Integration verification

- [x] 4.1 Run `cargo fmt --check`, `cargo clippy --workspace`, and `cargo test --workspace`; verify all pass with no warnings.

## Workflow follow-up

- Archive the chosen change (`replay-seek` or `replay-seek-fast`) after comparing implementations and satisfying review requirements.

