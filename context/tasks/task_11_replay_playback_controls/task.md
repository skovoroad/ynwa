# Task

Extend the record-playback mode (`--replay`) in `ynwa-player` so that the developer can
fully analyze recordings of short integration scenarios and matches visually: control
the playback speed (including slowing down), examine the initial state and the first steps,
restart viewing from the beginning, and see in the panel the current game stage and the fact that the recording is finished.

Extracted from the `task_9_fix_test_scenarios` discussion as a complement to increasing the detail of the
scenarios themselves.

---

## 1. Problem

In `task_9_fix_test_scenarios` it emerged that short test scenarios launched via
`./replay.sh ~/.ynwa/scenarios/<scenario>.jsonl` cannot be fully analyzed by eye.
Even with reduced `dt` and moderately lengthened scenarios, the player itself in `--replay` mode has a number of
limitations:

1. **Playback cannot be slowed down (or sped up).**
   In [`run_replay`](ynwa-player/src/main.rs:151), `rate_locked = true` is passed in `handle_input`,
   blocking the `+`/`-` keys. The reason for the block is that
   [`SimulationControl::set_rate`](ynwa-player/src/simulation.rs:59) recomputes
   `self.step_delta = 1.0 / rate`. In replay mode, changing `step_delta` is not allowed: physics must
   step strictly with the recorded `record.header.fixed_dt`, otherwise the trajectory will diverge due to discrete friction
   and the replay equivalence contract will be violated. However, the real-time pace of feeding steps
   (`rate` / `pace`) does not depend on `step_delta` and can change safely.
   Without slowing down, any inherently short scenario (a tackle at close range, a contest within a
   `1 m` radius, a tap-in from close range) will again flash by in fractions of a second.

2. **The initial positioning and the first ticks' events are lost; there is no restart.**
   The player opens the window (toggling `fullscreen: true` → `set_fullscreen(false)` in
   [`main.rs`](ynwa-player/src/main.rs:30)) and immediately starts the simulation without a pause, while
   [`advance`](ynwa-player/src/main.rs:188) accumulates `get_frame_time()` with no upper bound.
   During initialization and window appearance the first steps have already executed — for example, in
   `stdlib_kick_to_goal` the kick (`KickOutcome` on step 1) happens before the person sees
   the field. The recording cannot be restarted from frame zero inside the open window — one has to close
   the player and call `./replay.sh` again.

3. **The UI shows neither the end of the recording nor the assigned stage (`GameStage`).**
   - When all recording steps have been played (`steps_done >= max_steps`),
     [`draw_control_panel`](ynwa-player/src/ui.rs:29) keeps displaying the green
     `Status: Running`, although the world no longer steps.
   - The current stage [`GameState::stage`](ynwa-core/src/game.rs:267) (`Play`, `Setup("corner")`,
     `Setup("goal kick")`, `GameOver`) is not shown in the panel at all, and goal-line events in
     [`event_text`](ynwa-player/src/ui.rs:198) are all printed identically as `"Goal line; last touch: …"`.
     As a result, from the final frame before the first positioning tick it is impossible to visually confirm that the engine
     assigned a corner rather than a goal kick.

---

## 2. Requirements

### 2.1. Playback speed control in `--replay`

- Allow in `--replay` mode decreasing and increasing the playback speed with the `-` / `+` keys
  (e.g., speed multipliers `0.125x`, `0.25x`, `0.5x`, `1.0x`, `2.0x`, `4.0x` relative to
  real time `1.0 / fixed_dt`).
- When the speed changes in `--replay`, **only** the step-feeding frequency (`rate` / `pace`) changes, and
  `step_delta` remains strictly equal to `fixed_dt` from the record header. Trajectories and events at any
  speed remain bitwise/numerically the same as at `1.0x`.
- In `--record` mode, changing the pace must still be forbidden, since `fixed_dt`
  is fixed in the record header at start.

### 2.2. Control of viewing start, restart, and single step

- Ensure the ability to calmly examine the initial state (`initial_state`, step `0`) and
  the scenario's first ticks:
  - clamp the maximum `delta_time` passed to `simulation.accumulate(...)` per
    render frame, so the window open/resize lag does not "fast-forward" the recording by a batch of steps;
  - support starting `--replay` paused (by default or via a CLI flag — see the questions below).
- Add restarting the recording from the beginning via a hotkey (e.g., `R`) in `--replay` mode:
  recreating the replay world from the already read-in-memory `Record` (`Record` implements `Clone`), resetting
  the step counter in `SimulationControl`, and clearing the shown-events list (`visible_events` /
  `event_cursor`) without re-reading the file from disk.
- Support a single step forward while paused (e.g., with the `.` or `Right` key), until the recording
  reaches `total_steps` — convenient for frame-by-frame analysis of short episodes (the moment of a kick, line
  crossing, stage change).

### 2.3. Control-panel informativeness (`ui.rs`)

- Show the `FINISHED` status (in a separate color, distinct from `Running` and `PAUSED`) when the replay
  has reached the end of the recording (`simulation.finished()`).
- Show the current match stage (`GameStage`): `Play`, `Setup (<reason>)`, `GameOver`.
- Show the current playback speed/multiplier (in `--replay` mode, e.g. `Speed: 0.25x`)
  and up-to-date hints for the available control keys (`Space`, `+/-`, `R`, step).

---

## 3. What Is Touched (approximately)

Changes are localized to the `ynwa-player` crate (and, if necessary, `replay.sh`); `ynwa-core` and
`ynwa-football` require no changes:

- [`ynwa-player/src/simulation.rs`](ynwa-player/src/simulation.rs:5) — separating pace changes for
  normal play (`Play`, where `step_delta = 1 / rate`) and for `Replay` (speed multiplier with
  unchanged `step_delta = fixed_dt`), the public method `is_finished()`, reset to the beginning, and execution of
  exactly one step while paused;
- [`ynwa-player/src/input.rs`](ynwa-player/src/input.rs:8) — separating input policies by mode
  (or returning action commands: pause, speed `+/-`, restart, single step, exit);
- [`ynwa-player/src/main.rs`](ynwa-player/src/main.rs:126) — keeping the `Record` in `run_replay` for
  restart via `create_football_replay_world(record.clone())`, resetting the event cursor, guarding against
  the startup `get_frame_time()` jump;
- [`ynwa-player/src/ui.rs`](ynwa-player/src/ui.rs:9) — showing `FINISHED`, `Stage`, speed, and
  control hints;
- [`ynwa-player/src/cli.rs`](ynwa-player/src/cli.rs:6) and [`replay.sh`](replay.sh:1) — if
  necessary, new `--replay` CLI parameters.

---

## 4. Questions to Agree

1. **Behavior at `--replay` start:** start paused by default (the developer immediately sees the
   zero frame and presses `Space`), start immediately (relying on the `frame_time` clamp and the `R` key),
   or add a `--paused` flag (and should it be passed to `replay.sh`)?
2. **Control keys in `--replay`:** is the set `Space` (pause), `-`/`+` (speed),
   `R` (restart from step 0), `.` / `Right` (single step while paused) acceptable?
3. **Speed-multiplier range:** is the powers-of-two grid `0.125x ..= 4.0x` sufficient?

---

## 5. Preliminary Definition of Done (DOD)

- `SimulationControl` is covered by unit tests: in replay mode a speed change changes `pace` but
  does **not** change `step_delta`; `is_finished()`, a single step while paused (accounting for
  the `max_steps` boundary), and reset work correctly;
- `cargo test --workspace` green;
- in `--record` mode changing the speed remains blocked;
- in `--replay` mode pause, slow-down/speed-up, step-by-step advance, and restart from
  the zero step without restarting the application work; the final state after a run at any speed
  matches the run at `1.0x`;
- the control panel shows `GameStage`, the speed multiplier, and the `FINISHED` status when the
  recording ends;
- when viewing scenarios via `./replay.sh ~/.ynwa/scenarios/corner.jsonl` and
  `stdlib_kick_to_goal.jsonl`, the initial positioning, step 1 (the kick), smooth
  slowed ball movement, and the transition to the target `Setup` stage are visually observable.
