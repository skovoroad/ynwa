# Design

## Context

The player's `SimulationControl` (`ynwa-player/src/simulation.rs`) couples two quantities: `rate` (steps per wall-clock second, default `60.0`) and `timestep = 1.0 / rate`. `pace() = 1.0 / rate` is the frame-time that must accumulate before one step is due. In `advance` (`ynwa-player/src/main.rs`), each frame accumulates `get_frame_time()`, then steps the world by `timestep` once per accumulated `pace`. Because `timestep == pace`, the match clock always advances in lockstep with the wall clock, independent of `rate`; the `+`/`-` keys therefore change simulation resolution, not playback speed. There is currently no fast-forward.

`World::game().state().stage` is `GameStage::Setup(_)` or `GameStage::Play` (`ynwa-core/src/game.rs`). `advance` is the single function shared by all three modes (`Play`, `Record`, `Replay`). Replay re-simulates physics with the one fixed `RecordHeader::timestep` (`ynwa-core/src/replay.rs`, `ynwa-football/src/replay.rs`), so the simulation step must not vary.

## Goals / Non-Goals

**Goals:**

- Make the `Setup` stage run at `4x` wall-clock speed without changing the simulation step, journal, or replay determinism.
- Apply the acceleration uniformly across `Play`, `Record`, and `Replay`.

**Non-Goals:**

- No CLI flag (the `4x` factor is hardcoded), no on-screen speed indicator, no teleporting players.
- No change to the `+`/`-` key semantics, `ynwa-core`, `ynwa-football`, the recording format, or the replay world.

## Decisions

### 1. Speed multiplier at a fixed timestep (not a larger timestep)

Add a wall-clock time-scale to the player's pacing, leaving `timestep` fixed.

- **Why**: the simulation content — and replay fidelity — depends on a single fixed `RecordHeader::timestep`. A larger `timestep` during `Setup` would desync replay and coarsen movement/arrival.
- **Alternative considered**: enlarge `timestep` during `Setup`. Rejected: breaks replay determinism and changes the simulated motion.

### 2. Speed lives on `SimulationControl` and scales the accumulated frame time

Add `speed: f32` (default `1.0`) with `set_speed`, and make `accumulate` add `delta_time * speed`.

- **Why**: a single, minimal change point; frame-rate independent; directly unit-testable on `SimulationControl` without macroquad.
- **Alternative considered**: run a fixed number of extra steps per rendered frame. Rejected: ties pacing to the render frame rate and is less smooth.

### 3. Select speed from the game stage inside the shared `advance` function

Before accumulating, `advance` sets `simulation.set_speed(...)` to `SETUP_SPEEDUP` when `world.game().state().stage` is `Setup`, else `1.0`.

- **Why**: `advance` is the one place all three modes already share, so the stage→speed mapping lives in exactly one spot. The player is football-specific, so referencing `GameStage` there is appropriate.
- **Alternative considered**: duplicate the stage→speed logic in each of `run_play`/`run_record`/`run_replay`. Rejected: three copies of the same rule.

### 4. Hardcode the factor as a constant

Add `const SETUP_SPEEDUP: f32 = 4.0;` in `main.rs` and use `1.0` for `Play`.

- **Why**: the user explicitly chose a hardcoded `4x` with no flag and no indicator (YAGNI).

## Risks / Trade-offs

- **[Frame-boundary lag]** The speed changes on the next `accumulate` after a stage flip, so one rendered frame may run at the previous speed. → Negligible; `Setup` stages are short and the transition is visually seamless.
- **[Faster clock display]** `elapsed_time` ticks `4x` during `Setup`. → Accepted; no indicator is required by the decision.
- **[Future stages]** If another stage is added, the stage→speed mapping must be revisited. → The mapping is localized in `advance`, so it is a one-line extension.
- **[Recording appearance]** A recording still contains the full `Setup` at correct timestamps; replaying it also fast-forwards `Setup`. → Intended and consistent with "change only viewing speed".

## Migration Plan

None. No data format, persisted state, or public API changes; the recording format is untouched. Rollback is reverting the change (or setting the constant to `1.0`).

## Open Questions

None.
