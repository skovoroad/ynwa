# Design

## Context

- A running player is considered arrived at a `Run` target when `distance < ARRIVAL_THRESHOLD_METERS = 0.5` (`ynwa-core/src/systems/decision/decision_system.rs`), at which point `DecisionSystem` overrides the decision with `Stop`.
- The ball is claimed within `POSSESSION_RADIUS = 1.0` (`ynwa-core/src/systems/ball_possession.rs`), which is why the taker (within 0.5 m of the ball) acquires it after the transition to `Play`.
- Integration final-state positions are compared at `TOLERANCE = 1e-4` (`ynwa-integration-testing/src/compare.rs`).

So the taker ends on the ball (possession) but never at the exact restart point after running.

## Goals / Non-Goals

**Goals:**
- Give placement scenarios a way to assert the taker's arrival at the restart point.
- Preserve observable, movement-based placement scenarios (no pre-placement).

**Non-Goals:**
- Not changing how non-takers are asserted (regions remain correct).
- Not changing possession semantics.

## Decisions

None yet — see Open Questions.

## Open Questions

1. **Direction of the fix.** Two options, each with a different owning capability:
   - **(A) Engine precision.** Snap the `on_ball` taker to `restart_position` when it reaches the arrival threshold. Pro: keeps the existing exact-position assertion and matches "the taker stands on the ball" literally. Con: changes football setup semantics (the taker teleports the last ~0.5 m), needs `football-rules`/`core-simulation` spec + tests.
   - **(B) Framework tolerance.** Add a final-state `within = { point, radius }` (or equivalent) placement. Pro: minimal, stays in `integration-testing`, no engine change. Con: the check becomes "within the arrival radius" rather than "exactly at the point"; the framework gains a new assertion form.

This question must be resolved before implementation; the chosen option determines the spec delta and tasks.
