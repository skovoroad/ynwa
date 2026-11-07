# kick_off_after_goal_kick_team_b

## Purpose

Checks that the conceding team B restarts play from the centre spot after a goal.

## Initial state

- Stage: `Play`.
- Ball: free at the centre spot.
- Player 1 of team B stands on the ball; all other players stand away.

## Action

The taker receives possession and kicks the ball into play.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team = "B"`.
- A `KickOutcome` is recorded for that player.
- The run lasts about 2.5 s for visual replay.
