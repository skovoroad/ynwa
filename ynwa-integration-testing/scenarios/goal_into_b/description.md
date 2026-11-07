# goal_into_b

## Purpose

Legacy preliminary scenario: a ball crossing goal B's line between the posts after team A's last touch is a goal for team A.

## Initial state

- Stage: `Play`.
- Ball: free at `(34, 0, 70)`, moving towards goal B with `last_possessing_team = "A"`.
- Two players stand still.

## Action

The ball enters goal B; no player kicks it.

## Checks

- A `Goal` event is recorded for goal B.
- The score becomes `A = 1, B = 0`.
- The game enters `Setup("kick off")` with team B restarting.
