# Solution for task_9: edits to existing scenarios for observation

> Scope: only existing scenarios and existing stop criteria.
> Edits depending on the new `after_event` trigger (task_10) are **not** considered here.

## 1. Context and Boundaries

The `task_9` problem: replay in the player runs in real time, so "observation time = model
time until stop" (`SimulationControl::for_replay` sets `rate = 1/fixed_dt`). In the current
scenarios the ball is right next to the line and flies at high speed, so the event occurs
within 2–4 steps (~0.4 s), which visually looks like instant completion.

The solution within this task — **lengthen the phase before the event** (move the ball back and/or reduce
speed) and **reduce the simulation step** for smoother motion. The stop trigger remains
the same (`event`), so we observe the moment of the event itself (the ball going out over the line / a goal), not the
subsequent positioning — positioning is covered by `task_10` with a separate `after_event` trigger.

Physics constraints considered when choosing values:

- field: width `68.0 m`, length `104.615 m`;
- goal mouth: `x ∈ [30.34, 37.66]` (goal width `7.32 m`, center `x = 34`);
- ball radius `0.11 m`;
- ball friction `2.0 m/s²` (maximum free-ball run `v0² / 4`);
- the scenario RNG is deterministic (`temperature = 0.0`), so the kick is always straight at speed
  `shot_power / 5`.

## 2. General Edits

In all scenarios:

1. `scenario.toml`: `dt = 0.1` → `dt = 0.05`.
2. Leave the safety `steps = 1000` unchanged: at `dt = 0.05` this is 50 s of model
   time, while events occur within ~3–4 s.
3. `expect.stop`, `[[expect.journal]]`, and `final_state.toml` do not change: they are not tied to the exact
   step count and continue to match.

## 3. Edits by Scenario

### 3.1 `corner` — `initial_state.toml`

The ball is moved back from Team A's goal line (`z = 0`) and slowed; `x = 10` is kept outside the goal mouth,
so `GoalLine` fires, not a goal.

```toml
stage = "Play"

[ball]
position = { x = 10.0, y = 0.0, z = 40.0 }
velocity = { x = 0.0, y = 0.0, z = -15.0 }
last_possessing_team = "A"
```

Result: the `GoalLine` event ≈ 3.5 s (~70 steps at `dt = 0.05`).

### 3.2 `goal_kick` — `initial_state.toml`

The geometry is the same as for `corner`, but `last_possessing_team = "B"` — a goal kick is checked, not a corner.

```toml
stage = "Play"

[ball]
position = { x = 10.0, y = 0.0, z = 40.0 }
velocity = { x = 0.0, y = 0.0, z = -15.0 }
last_possessing_team = "B"
```

Result: the `GoalLine` event ≈ 3.5 s (~70 steps).

### 3.3 `throw_in` — `initial_state.toml`

The ball is moved from the right touchline (`x = 68`) toward the left; motion is along `x` only.

```toml
stage = "Play"

[ball]
position = { x = 10.0, y = 0.0, z = 52.3 }
velocity = { x = 20.0, y = 0.0, z = 0.0 }
last_possessing_team = "A"
```

Result: the `Touchline` event ≈ 3.5 s (~70 steps).

### 3.4 `goal_into_b` — `initial_state.toml`

The ball is moved back from Team B's goal (`z = 104.615`), `x = 34` stays in the goal mouth.

```toml
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 70.0 }
velocity = { x = 0.0, y = 0.0, z = 15.0 }
last_possessing_team = "A"
```

Result: the `Goal` event (into B's goal) ≈ 2.9 s (~58 steps).

### 3.5 `goal_into_a` — `initial_state.toml`

The ball is moved back from Team A's goal (`z = 0`), `x = 34` stays in the goal mouth.

```toml
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 35.0 }
velocity = { x = 0.0, y = 0.0, z = -15.0 }
last_possessing_team = "B"
```

Result: the `Goal` event (into A's goal) ≈ 2.9 s (~58 steps).

### 3.6 `stdlib_kick_to_goal` — `initial_state.toml`

Here the ball is not given an initial velocity; it gets it from the kick (`kick_outcome`). The kick speed
is fixed: `shot_power = 50` → `10 m/s`, maximum run `10² / 4 = 25 m`. Therefore the kicker
cannot be moved farther than ~25 m from the goal: otherwise the ball will not roll there and the goal will not happen.

The current position of A#1 is `z = 102` (≈2.6 m to the goal). We move it and the ball to `z = 82` (≈22.7 m to
the goal) — this gives ~3.5 s of flight with sufficient distance margin.

```toml
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 82.0 }
velocity = { x = 0.0, y = 0.0, z = 0.0 }
possessed_by = { team = "A", number = 1 }
last_possessing_team = "A"

[[players]]
team = "A"
number = 1
position = { x = 34.0, y = 0.0, z = 82.0 }

[[players]]
team = "B"
number = 1
position = { x = 34.0, y = 0.0, z = 10.0 }
```

Result: `kick_outcome` on the first step, then `Goal` into B's goal ≈ 3.5 s (~70 steps).

## 4. Summary

| Scenario | `dt` | What changes in `initial_state.toml` | Event | ~Time to event | ~Steps (`dt=0.05`) |
|---|---|---|---|---|---|
| `corner` | 0.05 | ball `z: 5 → 40`, `vz: -20 → -15` | `GoalLine` (A) | 3.5 s | 70 |
| `goal_kick` | 0.05 | ball `z: 5 → 40`, `vz: -20 → -15` | `GoalLine` (B) | 3.5 s | 70 |
| `throw_in` | 0.05 | ball `x: 34 → 10`, `vx` stays 20 | `Touchline` (A) | 3.5 s | 70 |
| `goal_into_b` | 0.05 | ball `z: 99 → 70`, `vz: 20 → 15` | `Goal` (B) | 2.9 s | 58 |
| `goal_into_a` | 0.05 | ball `z: 5 → 35`, `vz: -20 → -15` | `Goal` (A) | 2.9 s | 58 |
| `stdlib_kick_to_goal` | 0.05 | ball and A#1 `z: 102 → 82` | `kick_outcome` → `Goal` (B) | 3.5 s | 70 |

## 5. Verification

After the edits, run:

```sh
cargo test -p ynwa-integration-testing
```

All scenarios must remain green (expectations are not tied to the exact step count). For visual
verification — record the scenario after the test run and replay:

```sh
./replay.sh ~/.ynwa/scenarios/<scenario>.jsonl
```

## 6. What Remains for `task_10`

The current `event` trigger stops the run at the moment of the event, so the final frame shows
the ball behind the line, not the positioning after the set piece (ball at the corner/goal/throw-in spot).
To observe the positioning as well, after `task_10` is implemented these same scenarios switch to
`when = "after_event"` with `delay_steps = 1..=2`, and a ball-position/`restart_*` check is added to `final_state.toml`. This is not done within the current task.
