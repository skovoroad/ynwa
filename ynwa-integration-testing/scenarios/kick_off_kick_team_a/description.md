# kick_off_kick_team_a

## Purpose

Checks that the designated kick-off team puts the ball into play from the centre spot.

## Initial state

- Stage: `Play`.
- Ball: free (owned by nobody), lying at the centre spot.
- Player 1 of team A stands on the ball; all other players stand away from it.

## Action

The taking player receives possession and kicks the ball into play.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team` equal to team A.
- A `KickOutcome` is recorded for that player.
- The run lasts long enough for the recorded replay to show the kick (about 2.5 s).
