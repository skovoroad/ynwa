# Design

## Context

See proposal.md for motivation. Relevant current state:

- `run_replay` in `ynwa-player/src/main.rs` reads a `Record`, clones its journal into `events`, computes `total_steps` from the footer (or the last event timestamp for an interrupted recording), and builds the replay world via `create_football_replay_world(record)`. That call consumes the `Record`.
- The replay world is a deterministic physics-only re-simulation: `ReplaySetupBallPlacer` + `ReplayDriver` (feeds journaled events back by timestamp) + `PhysicsSystem`.
- `SimulationControl` (`simulation.rs`) owns `steps_done`, `max_steps`, `accumulator`, `paused`, and `speed`; `finished()` is true once `steps_done >= max_steps`.
- The `Events:` panel is filled incrementally from `events` using an `event_cursor` that tracks how many entries have `timestamp <= elapsed_time`.
- `Record` and `RecordHeader` are `Clone`; `GameState` is `Clone`.

## Goals / Non-Goals

**Goals:**
- Add fixed 5-second forward/backward seek in `Replay` mode only.
- Reuse the existing replay re-simulation path; no changes to `ynwa-core` or `ynwa-football`.
- Keep seek deterministic: a seeked-to position matches a fresh replay stepped to that position.

**Non-Goals:**
- No timeline scrubber, drag-to-seek, or configurable seek step.
- No snapshots/checkpoints (unnecessary at current recording lengths; see Risks).
- No seek in `Play` or `Record` modes.
- No changes to the recording file format.

## Decisions

### Forward seek advances in place; backward seek rebuilds

Forward seek just steps the current world `seek_steps` times in a tight loop. Backward seek cannot run physics in reverse, so it rebuilds the world from `record.clone()` and fast-forwards to the target step. This mirrors the existing determinism guarantee ("stepping `total_steps` times reproduces state within `1e-4`").

- *Alternative considered*: rebuild for both directions for a single code path. Rejected — forward is already the normal stepping direction and needs no rebuild, so rebuilding would be wasted work.
- *Alternative considered*: store `GameState` snapshots. Rejected — a snapshot alone is insufficient (the `ReplayDriver` journal cursor must also be restored), which would require a new core "resume from position" primitive. See Risks.

### Keep the `Record` available for rebuild

`run_replay` currently moves `record` into `create_football_replay_world`. It will instead hold `record` and pass `record.clone()` when building/rebuilding the world. `Record` is already `Clone`.

### Seek step arithmetic

`seek_steps = round(5.0 / timestep)` (300 at `timestep = 1/60`), computed once. Target steps are clamped to `[0, total_steps]`.

### Position accounting lives in `SimulationControl`

Add `steps_done()` accessor and `reposition(step)`, which clamps to `[0, max_steps]`, sets `steps_done`, and resets `accumulator` so wall-clock pacing resumes cleanly after the seek. The seek orchestration (world stepping/rebuild and event sync) lives in `main.rs`, because it needs both the `World` and the `Record`.

### Events panel is re-derived on seek

After a seek the incremental `event_cursor`/`visible_events` state is rebuilt from `events` for the new `elapsed_time` (events are already sorted by timestamp), and the scroll offset resets to auto-follow. This is a one-time cost per seek, not per frame.

### Input binding

`Left`/`Right` trigger backward/forward seek. They are currently unused (`Up`/`Down`/`PgUp`/`PgDown` scroll the events panel). Seek is polled in `run_replay` after the existing `handle_input` call so the common controls stay centralized and seek can reach the `World`/`Record`.

## Risks / Trade-offs

- **Backward seek cost is O(position)** → At current recording lengths (2 minutes ≈ 7200 physics-only steps) a rebuild is milliseconds and imperceptible. If recordings grow to tens of minutes, introduce checkpoints with a core "resume from position" primitive later.
- **Determinism dependency** → The correctness of backward seek relies entirely on the existing replay equivalence contract. The seek path reuses `create_football_replay_world` unchanged, so it inherits that guarantee rather than re-implementing it.
- **Events panel scroll reset** → After seek the panel jumps to the newest visible entry. This is intended (auto-follow) and matches the existing behavior when `elapsed_time` advances.
- **Interrupted recordings (`footer.total_steps == 0`)** → `total_steps` is already derived from the last journal timestamp, so seek clamping works unchanged.

## Migration Plan

None — no data format, config, or API changes. The seek is additive UI behavior in `ynwa-player`.

## Open Questions

None.
