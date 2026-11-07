# corner

## Purpose

Legacy preliminary scenario: a ball driven over goal A's line outside the goalposts after team A's last touch must produce a corner for team B.

## Initial state

- Stage: `Play`.
- Ball: free at `(10, 0, 40)`, moving towards goal A with `last_possessing_team = "A"`.
- Two players stand still.

## Action

The ball crosses the goal line; no player kicks it.

## Checks

- A `GoalLine` event is recorded for team A.
- The game enters `Setup("corner")` with team B restarting.
