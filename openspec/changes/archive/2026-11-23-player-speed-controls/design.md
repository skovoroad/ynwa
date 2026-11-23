# Design

## Context

`SimulationControl` (`ynwa-player/src/simulation.rs`) stores `rate` and `timestep = 1/rate`, and `+`/`-` adjust `rate` via `increase_rate`/`decrease_rate`/`set_rate`. Because `timestep` and `pace()` move together, this changes simulation resolution, not playback speed. `handle_input` gates the keys behind `rate_locked` (disabled in `Record`/`Replay` to keep `RecordHeader::timestep` fixed). The `speed-up-setup-stage` change introduces the `speed` concept and Setup auto-`4x`; this change turns `speed` into the user-facing manual control and removes the resolution machinery.

## Goals / Non-Goals

**Goals:**

- Make `+`/`-` control a wall-clock playback `speed`, never the `timestep`.
- Make the control work in all three modes (`Play`, `Record`, `Replay`).
- Keep manual `speed` scoped to the `Play` stage; `Setup` auto-`4x` wins during setup.

**Non-Goals:**

- No CLI flag, no on-screen indicator, no teleporting.
- No change to `ynwa-core`, `ynwa-football`, the recording format, or replay semantics.

## Decisions

### 1. `speed` is the user-adjustable multiplier; rate methods are removed

Add `speed: f32` (default `1.0`) with `increase_speed`/`decrease_speed` that double/halve it, clamped to `0.25x`–`8x`. Remove `increase_rate`, `decrease_rate`, and `set_rate`.

- **Why**: matches the intent (speed, not resolution); one knob instead of two.
- **Alternative considered**: keep `rate` and add a separate speed knob. Rejected: extra surface and confusion.

### 2. Store the fixed `timestep` directly; drop `rate`

`SimulationControl` stores `timestep` (from `1/60` or `RecordHeader::timestep`) and `pace()` returns `timestep`.

- **Why**: with a fixed timestep, `rate` and the `rate`/`pace` split are redundant (`pace == timestep`).
- **Alternative considered**: keep `rate` as a derived field. Rejected: redundant state.

### 3. Apply the effective speed in `advance`, not by mutating `speed` each frame

In `advance`, compute `effective = if Setup { SETUP_SPEEDUP } else { simulation.speed() }` and call `simulation.accumulate(get_frame_time() * effective)`. `accumulate` stays a pure adder.

- **Why**: the `Setup` override must not overwrite the user's manual `speed`; the override becomes a read-time decision and `speed` remains the stored `Play` value.
- **Alternative considered**: `advance` calls `set_speed(...)` every frame (the shape from `speed-up-setup-stage`). Rejected: it would clobber the manual value.
- **Note**: this refines `speed-up-setup-stage` — its `set_speed` + `accumulate`-scaling detail is replaced here, with `accumulate` returned to a pure adder.

### 4. Remove the `rate_locked` guard; `+`/`-` work in all modes

- **Why**: the original lock existed solely to prevent `timestep` drift. With `speed`, `timestep` is fixed, so the lock is unnecessary (option (б)).
- **Alternative considered**: keep locking. Rejected: an artificial limitation with no correctness benefit.

### 5. Hardcode the speed bounds as constants

Use `MIN_SPEED = 0.25` and `MAX_SPEED = 8.0` (doubling/halving, powers of two).

- **Why**: slow-motion below `1x` is useful; powers of two match the existing `*2`/`/2` behavior.

## Risks / Trade-offs

- **[Setup override vs manual speed]** Pressing `+`/`-` during `Setup` changes the stored `speed` but has no visible effect until `Play` resumes. → Documented behavior; the override is a read-time decision.
- **[Arbitrary bounds]** `0.25x`/`8x` are chosen constants. → Constants make them easy to widen later.
- **[Reconciliation with `speed-up-setup-stage`]** This change replaces that change's `set_speed`/`accumulate`-scaling detail. → Apply `speed-up-setup-stage` first, then this change; its tasks rework the `speed` application.

## Migration Plan

None. No persisted data, format, or public API changes.

## Open Questions

None.
