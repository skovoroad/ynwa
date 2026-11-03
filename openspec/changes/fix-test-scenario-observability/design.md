# Design

## Context

The player replays a recording in real time: `SimulationControl::for_replay` sets
`rate = 1/fixed_dt`, so one model step is emitted per `fixed_dt` wall-clock seconds. The
wall-clock duration of a replay therefore equals the model time until the stop event, and
`fixed_dt` is the scenario's `run.dt` baked into the recording header.

`fixed_dt` does **not** set the duration — it only sets how many frames that duration is
split into. Duration is set entirely by how long the ball takes to reach the line. See
proposal.md for the motivation (the current ~0.3 s is unobservable).

Physics facts that constrain the value choices (all in code):

- ball friction `2.0 m/s²` (`BALL_FRICTION_DECELERATION` in `ynwa-core/src/systems/physics.rs`),
  so a free ball at speed `v0` runs at most `v0² / 4` m;
- field width `68.0` m, length `104.615` m; goal mouth `x ∈ [30.34, 37.66]`;
- scenario RNG is deterministic (`temperature = 0.0`, fixed seed), so a kick is straight at
  exactly `shot_power / 5` m/s.

## Goals / Non-Goals

**Goals:**

- Stretch the pre-event phase of every scenario to roughly 3–3.5 s of model time so the
  initial positioning, ball motion, and the stop event are visible.

**Non-Goals:**

- Do not change `dt` (stays `0.1`).
- Do not change stop criteria, `[expect]`, or `final_state.toml`.
- Do not add a post-event observation trigger (`after_event`) — that is a separate task.
- No code, engine, runner, or player changes.

## Decisions

### D1 — Lengthen the phase via positions/speeds only, keep `dt = 0.1`

Duration = model time until the event, independent of `dt`. Lowering `dt` to `0.05` would
only smooth motion (10 → 20 world frames/s), not lengthen it, and would touch every
`scenario.toml`. Keeping `dt = 0.1` is the minimal change and still yields ~35 frames over
~3.5 s, which is enough to observe the event.

Alternatives considered: `dt = 0.05` for smoother motion — rejected as unnecessary for
minimal observability (YAGNI).

### D2 — Per-scenario values chosen to stay within physics limits

The ball must still reach the line (or goal) and still be moving when it crosses, so a
`Goal` is detected inside the goal window rather than stopping short.

| Scenario | `initial_state.toml` change | Event | ~Time to event |
|---|---|---|---|
| `corner` | ball `z: 5 → 40`, `vz: -20 → -15` | `GoalLine` (A) | ~3.5 s |
| `goal_kick` | ball `z: 5 → 40`, `vz: -20 → -15` | `GoalLine` (B) | ~3.5 s |
| `throw_in` | ball `x: 34 → 10`, `vx` stays `20` | `Touchline` (A) | ~3.5 s |
| `goal_into_b` | ball `z: 99 → 70`, `vz: 20 → 15` | `Goal` (B) | ~2.9 s |
| `goal_into_a` | ball `z: 5 → 35`, `vz: -20 → -15` | `Goal` (A) | ~2.9 s |
| `stdlib_kick_to_goal` | ball and A#1 `z: 102 → 82` | `kick_outcome` → `Goal` (B) | ~3.5 s |

Rationale per scenario:

- `corner` / `goal_kick`: `x = 10` stays outside the goal mouth, so crossing `z < -0.11`
  produces `GoalLine`, not `Goal`; the slower `vz = -15` and `z = 40` give ~3.5 s.
- `throw_in`: moving `x` from `34` to `10` keeps `vx = +20`, so the ball travels farther to
  exit the right touchline at `x > 68.11` (~3.5 s).
- `goal_into_b` / `goal_into_a`: `x = 34` stays in the goal mouth; the ball still crosses
  the goal line moving (~9 m/s) and is detected as a goal, not an overshoot.
- `stdlib_kick_to_goal`: kick speed is fixed at `shot_power / 5 = 10` m/s, so the maximum
  run is `10² / 4 = 25` m. `z = 82` leaves ~22.7 m to the goal line — within range, and the
  ball is still moving (~3 m/s) when it crosses.

### D3 — Keep the `event` stop trigger

The final frame remains the moment of the event (ball behind the line), not the resulting
set-piece positioning. Observing the positioning needs the `after_event` trigger and is out
of scope here.

## Risks / Trade-offs

- [Kick scenario is near the physics limit (~22.7 m of a 25 m max run)] → The chosen `z = 82`
  leaves margin; the ball is still moving when it reaches the goal, so the `Goal` fires.
- [At the locked replay tempo (~1:1, no speed-up) ~3 s is brief for careful inspection] →
  Acceptable for minimal observability; the player keeps pause available, and post-event
  inspection is deferred to the `after_event` task.
- [The `throw_in` prose in `context/tasks/.../solution.md` says "toward the left" while the
  values move the ball right] → No code impact; the planning docs here use the correct
  direction. The task source document is not part of this change.
