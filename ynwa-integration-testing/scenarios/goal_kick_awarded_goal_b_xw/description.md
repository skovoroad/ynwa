# goal_kick_awarded_goal_b_xw

## Purpose

Checks that a ball leaving the field over the `xw` side of goal B's goal line, after the attacking team's last touch, is awarded as a goal kick.

## Initial state

- Stage: `Play`.
- Ball: free, moving towards the goal line, with `last_possessing_team = "A"`.
- Players stand away from the ball path.

## Action

The ball crosses the goal line outside the goalposts; no player kicks it.

## Checks

- A `GoalLine` event is recorded for team A.
- Decisions are reset and a restart is set on the 5.5 m line at `(34, 99.11538)` for team B.
- The game enters `Setup("goal kick")`.
