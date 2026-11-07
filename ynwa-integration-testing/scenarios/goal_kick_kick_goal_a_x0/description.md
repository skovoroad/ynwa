# goal_kick_kick_goal_a_x0

## Purpose

Checks that the defending team puts the ball into play from a goal kick at goal A.

## Initial state

- Stage: `Play`.
- Ball: free on the 5.5 m line at `(34, 5.5)`.
- Player 1 of team A stands on the ball; all other players stand away.

## Action

The taker receives possession and kicks the ball into play.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team = "A"`.
- A `KickOutcome` is recorded for that player.
- The run lasts about 2.5 s for visual replay.
