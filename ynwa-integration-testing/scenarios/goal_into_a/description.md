# goal_into_a

## Purpose

Legacy preliminary scenario: a ball crossing goal A's line between the posts after team B's last touch is a goal for team B.

## Initial state

- Stage: `Play`.
- Ball: free at `(34, 0, 35)`, moving towards goal A with `last_possessing_team = "B"`.
- Two players stand still.

## Action

The ball enters goal A; no player kicks it.

## Checks

- A `Goal` event is recorded for goal A.
- The score becomes `A = 0, B = 1`.
- The game enters `Setup("kick off")` with team A restarting.
