# corner_awarded_goal_a_x0

## Purpose

Checks that a ball leaving the field over the `x0` side of goal A's goal line, after the defending team's last touch, is awarded as a corner.

## Initial state

- Stage: `Play`.
- Ball: free, moving towards the goal line, with `last_possessing_team = "A"`.
- Players stand away from the ball path.

## Action

The ball crosses the goal line outside the goalposts; no player kicks it.

## Checks

- A `GoalLine` event is recorded for team A.
- Decisions are reset and a restart is set at the nearest corner `(0, 0)` for team B.
- The game enters `Setup("corner")`.
