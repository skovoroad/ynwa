# goal_kick

## Purpose

Legacy preliminary scenario: a ball driven over goal A's line outside the goalposts after team B's last touch must produce a goal kick for team A.

## Initial state

- Stage: `Play`.
- Ball: free at `(10, 0, 40)`, moving towards goal A with `last_possessing_team = "B"`.
- Two players stand still.

## Action

The ball crosses the goal line; no player kicks it.

## Checks

- A `GoalLine` event is recorded for team B.
- The game enters `Setup("goal kick")` with team A restarting.
