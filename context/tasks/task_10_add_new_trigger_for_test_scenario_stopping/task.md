# Task

Extend the test-scenario stop model in `ynwa-integration-testing` so that a scenario can
finish not at the moment a meaningful football event occurs, but **after its consequences have manifested** —
above all, the player and ball positioning after a set piece.

The exact solution mechanism is **not fixed** and is a matter for discussion. This document describes
the problem, the behavior requirements, and possible directions; the concrete approach (the criterion type, its fields,
and semantics) is to be chosen and agreed separately.

Extracted from the `task_9_fix_test_scenarios` analysis.

---

## 1. Problem

The runner stops at the first triggered criterion after each step
([`runner.rs`](ynwa-integration-testing/src/runner.rs:50)). The meaningful event and its
consequences are recorded on the **same step**: `FootballGameManager` records the event
(e.g., `GoalLine`), and then in `handle_event` changes `stage` and sets `restart_position` and
`restart_team` ([`game_manager.rs`](ynwa-football/src/game_manager.rs:250)).

Because of this:

- stopping on `event = "GoalLine"` and stopping on `stage = Setup("corner")` yield the same step;
- the ball physically occupies `restart_position` only on the **next** `Setup` tick
  (`FootballGameManager` in the `Setup` branch overwrites the ball, and in replay — `ReplaySetupBallPlacer`).

As a result, the scenario record is cut off before the set-piece positioning is
applied, and in the player it is impossible to observe, for example, the ball at the corner flag. The existing criteria
(`stage`, `event`) do not allow "continuing the run after the event", because they fire exactly
on the event step.

---

## 2. What Is Required (without tying to an implementation)

The scenario must be able to express a stop that occurs **later** than the meaningful event,
while:

- remaining deterministic and fitting into the typed `[[run.stop]]` / `[expect.stop]` model;
- preserving an understandable stop reason (`StopReason`) and the ability to compare in `compare_stop`;
- not breaking the existing criteria and the "first triggered criterion stops the run" rule;
- not canceling the safety-criterion requirement `steps`/`time`;
- behaving correctly if the target event never occurs (the run ends with the safety
  criterion).

Additionally, it must be fixed how the "stop after the event" fact is verified: in `expect.stop`,
in `[[expect.journal]]`, in `final_state.toml`, or a combination.

---

## 3. Possible Solution Directions (for discussion)

Below are options that are not the final solution. One must be chosen (or combined),
evaluating expressiveness, simplicity, and impact on the existing criterion model.

### 3.1 "Several steps after the event"

Introduce a criterion of the form "stop N steps after the first occurrence of the event". The direction was already
discussed (tentative name `after_event`, a delay parameter in steps).

Pros:

- simple and predictable;
- minimally affects the criterion model.

Cons / questions:

- tying to a fixed step count is fragile: changing `dt`, physics, or the field forces a recalculation
  of the step count;
- it must be decided what counts as the "event step" (by `new_events`, as for the current `event`, or otherwise).

### 3.2 Stopping on a reached state / the next event

Stop not by a step count, but by an observable state or the next journal event
that signals the positioning is complete. For example:

- on the appearance of `restart_position`/`restart_team` + the `Setup` stage;
- on the next `StageChange` (e.g., the reverse transition to `Play`);
- on a `DecisionAssigned` entry produced by the positioning.

Pros:

- does not depend on the exact step count, more robust to `dt`/physics changes;
- the stop reason is semantically closer to "positioning applied".

Cons / questions:

- it must be clearly defined which exact state/event means "positioning complete";
- may require a new criterion type or extending `stage`/`event`.

### 3.3 Extending the existing `event` with an optional delay

Instead of a new criterion type, add an optional delay parameter (in steps
or by condition) to `when = "event"`. This keeps a single type but changes the semantics of the existing criterion.

Pros:

- fewer new types;
- backward compatibility at a default delay value.

Cons / questions:

- the meaning of `event` blurs ("stop on the event" vs "some time after the event");
- still leaves the "steps or condition" question open.

### 3.4 Separating semantic stop from record length

Leave the stop criterion unchanged, but make the record (`Record`) continue for a few more
steps after the criterion fires — e.g., a separate "append N steps" parameter.

Pros:

- does not change the criterion model or `StopReason`;
- affects only what goes into the record/`final_state`.

Cons / questions:

- a divergence between the "run steps" and the record's `total_steps`;
- it must be decided how to compare `expect.stop` and `final_state` with this tail.

---

## 4. What Any Solution Will Probably Touch (approximately)

- [`dto.rs`](ynwa-integration-testing/src/dto.rs:184) — `StopCriterionDto` / `StopExpectationDto`;
- [`criterion.rs`](ynwa-integration-testing/src/criterion.rs:78) — `StopCriterion` / `StopReason`;
- [`runner.rs`](ynwa-integration-testing/src/runner.rs:50) — the run loop and criterion checking;
- [`compare.rs`](ynwa-integration-testing/src/compare.rs:212) — `expect.stop` comparison;
- documentation: [`protocols.md`](context/docs/protocols.md:224) §4.2–4.3 and
  [`integration_testing.md`](context/docs/integration_testing.md:126) §4.3;
- scenarios from `task_9_fix_test_scenarios` (e.g., `corner`) — as consumers of the new mechanism.

---

## 5. Discussion Questions

1. Which mechanism do we choose: a fixed step count, a state/event condition, extending
   `event`, or a record "tail"?
2. If "by event/state" — what exactly counts as the positioning-complete marker?
3. How is it expressed in `[expect.stop]` and compared in `compare_stop`?
4. How to behave when the target event does not occur?
5. Is backward compatibility with the current `when = "event"` needed, and how does it affect the semantics?

---

## 6. Preliminary Definition of Done (DOD)

To be made concrete after the mechanism is chosen. Approximately:

- `cargo test -p ynwa-integration-testing` green;
- the new mechanism parses, validates, and deterministically stops the run;
- `compare_stop` correctly compares the new stop reason;
- at least one scenario (e.g., `corner`) uses the mechanism so that the record/`final_state`
  shows the positioning after the event;
- documentation updated.
