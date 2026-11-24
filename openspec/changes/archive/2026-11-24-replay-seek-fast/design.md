# Design

## Context

See `proposal.md` for motivation and `specs/player-client/spec.md` for behavioral requirements. Relevant current state in the codebase:

- `run_replay` in `ynwa-player/src/main.rs` loads a `Record`, clones `record.journal` into `events`, incrementally clones entries into `visible_events`, and passes ownership of `record` into `create_football_replay_world(record)`.
- `World::step(timestep)` advances `elapsed_time` by `timestep`, calls each `System::update(&mut game, new_timestamp)`, and advances `Game`.
- Two replay systems hold internal cursor/time state outside `GameState`:
  - `ReplayDriver` (`ynwa-core/src/replay.rs`) holds `journal: Vec<JournalEntry>` and `next_index: usize`, applying entries whose `timestamp <= timestamp`.
  - `PhysicsSystem` (`ynwa-core/src/systems/physics.rs`) holds `last_update: f32` (initialized to `0.0`) and integrates positions over `delta_time = timestamp - self.last_update`.
- `Game` has no other replay-relevant mutable state (`RngManager` is unused in replay and `journal_sink` is a no-op). `GameState` and `GameConfig` are `Clone`.

## Goals / Non-Goals

**Goals:**
- Fixed 5-second forward/backward seek in `Replay` mode with `O(1)` (`<= seek_steps` re-simulation) backward seek cost regardless of recording length.
- Bit-identical state reproduction when resuming from a checkpoint compared to stepping from step `0`.
- Zero-copy event display (`&record.journal[..event_cursor]`) and centralized keyboard handling in `input.rs`.
- Minimal, non-intrusive additions to `ynwa-core` and `ynwa-football`.

**Non-Goals:**
- No timeline scrubber, drag-to-seek, or configurable seek step.
- No `System` trait changes (no `reset`/`as_any` on `dyn System`).
- No seek in `Play` or `Record` modes, and no recording file format changes.

## Decisions

### Periodic in-memory `GameState` checkpoints

`run_replay` maintains `checkpoints: Vec<(u64, GameState)>`, initialized with `(0, record.header.initial_state.clone())` and appended whenever `steps_done` crosses a multiple of `seek_steps` (`round(5.0 / timestep)`) during normal playback or forward seek.

- *Why `seek_steps` interval*: Every backward seek lands within `[0, seek_steps]` steps of a stored checkpoint, bounding re-simulation work to at most 5 match-seconds (300 physics-only steps at 60 Hz) even for 90-minute matches.
- *Alternative considered*: Re-simulating from step `0` on every backward seek. Rejected here to avoid `O(position)` scaling on longer recordings and repeated full `Record` cloning.

### Minimal resumption constructors in `ynwa-core` and `ynwa-football`

Resuming from a `GameState` checkpoint at step `cp_step` (`elapsed_time = T`) requires restoring both `PhysicsSystem::last_update` and `ReplayDriver::next_index`:
- `PhysicsSystem::from_timestamp(last_update: f32)` initializes `last_update` to `T`, so the next step computes `delta_time = (T + timestep) - T == timestep` instead of `T + timestep`.
- `ReplayDriver::from_index(journal: Vec<JournalEntry>, next_index: usize)` starts at `next_index = journal.partition_point(|e| e.timestamp <= T)`, skipping events already reflected in `GameState`.
- `create_football_replay_world_from_state(config: GameConfig, state: GameState, journal: Vec<JournalEntry>, next_index: usize) -> Result<World, String>` in `ynwa-football/src/replay.rs` builds the replay world from a checkpoint; `create_football_replay_world(record)` delegates to it with `next_index = 0`.

- *Why not `System::reset` on `World`*: `World` stores `Vec<Box<dyn System>>`; adding downcast/reset methods to `System` would touch the core trait across all systems. Passing `from_timestamp` / `from_index` into a shared world builder keeps `System` untouched.
- *Journal cloning*: Passing `record.journal[next_index..].to_vec()` with `next_index = 0` to `from_index` (or keeping `record.journal.clone()` with `next_index`) is simple; slicing `record.journal[next_index..].to_vec()` with `ReplayDriver::new(...)` would even avoid `from_index`, though `from_index` makes the intent explicit and keeps indices aligned with `record.journal`.

### Forward seek steps in place; backward seek resumes from checkpoint

- **Forward seek (`Right`)**: `target = (steps_done + seek_steps).min(total_steps)`. Step `world.step(timestep)` in a loop for `target - steps_done` steps, recording any crossed checkpoint boundaries, then call `simulation.reposition(target)`.
- **Backward seek (`Left`)**: `target = steps_done.saturating_sub(seek_steps)`. Pick the last `(cp_step, cp_state)` in `checkpoints` with `cp_step <= target`, rebuild `world` via `create_football_replay_world_from_state(record.header.config.clone(), cp_state.clone(), record.journal.clone(), next_index)`, step `target - cp_step` times, and call `simulation.reposition(target)`.

### Zero-copy events panel via `partition_point`

`run_replay` keeps `record` alive and drops both `events` and `visible_events`. Instead, it tracks `event_cursor: usize` and passes `&record.journal[..event_cursor]` to `render_scene`:
- During normal playback, `event_cursor` advances monotonically while `timestamp <= elapsed`.
- After any seek, `event_cursor = record.journal.partition_point(|e| e.timestamp <= elapsed)` and `event_scroll` resets to `0` (auto-follow).

### Centralized input and `SimulationControl` position accounting

- `ynwa-player/src/input.rs` exposes a `SeekDirection` (`Backward`, `Forward`) helper (e.g. `poll_replay_seek() -> Option<SeekDirection>`) so all `KeyCode` polling stays in `input.rs`.
- `ynwa-player/src/simulation.rs` adds `steps_done(&self) -> u64` and `reposition(&mut self, step: u64)` (clamping to `max_steps` when `Some`, setting `steps_done`, and resetting `accumulator = 0.0`).

## Risks / Trade-offs

- **Memory per checkpoint** → Each checkpoint clones `GameState` (~22 players + ball + team stats, a few KB). At 1 checkpoint per 5 match-seconds, a 90-minute match stores ~1080 states (a few MB). Acceptable in desktop memory; can be thinned or capped later if needed.
- **Floating-point `elapsed_time` drift** → Because checkpoints capture the exact `GameState` produced by sequential `world.step(timestep)` calls, `cp_state.elapsed_time` and `PhysicsSystem::last_update` match a continuous run bitwise, preserving exact determinism.
- **Interrupted recordings (`footer.total_steps == 0`)** → `total_steps` is already derived from the last journal timestamp, so seek clamping and checkpointing work unchanged.

