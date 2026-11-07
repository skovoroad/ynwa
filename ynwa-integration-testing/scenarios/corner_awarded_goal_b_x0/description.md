# corner_awarded_goal_b_x0

## Purpose

Checks that a ball leaving the field over the `x0` side of goal B's goal line, after the defending team's last touch, is awarded as a corner.

## Initial state

- Stage: `Play`.
- Ball: free, moving towards the goal line, with `last_possessing_team = "B"`.
- Players stand away from the ball path.

## Action

The ball crosses the goal line outside the goalposts; no player kicks it.

## Checks

- A `GoalLine` event is recorded for team B.
- Decisions are reset and a restart is set at the nearest corner `(0, 104.61538)` for team A.
- The game enters `Setup("corner")`.
