# goal_kick_kick_goal_b_xw

## Purpose

Checks that the defending team puts the ball into play from a goal kick at goal B.

## Initial state

- Stage: `Play`.
- Ball: free on the 5.5 m line at `(34, 99.11538)`.
- Player 1 of team B stands on the ball; all other players stand away.

## Action

The taker receives possession and kicks the ball into play.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team = "B"`.
- A `KickOutcome` is recorded for that player.
- The run lasts about 2.5 s for visual replay.
