# throw_in_kick_x0_half_b_by_b

## Purpose

Checks that the team awarded a throw-in puts the ball into play from the `x0` touchline in half `b`.

## Initial state

- Stage: `Play`.
- Ball: free on the touchline at `(0, 80)`.
- Player 1 of team A stands on the ball; all other players stand away.

## Action

The taker receives possession and kicks the ball into play.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team = "A"`.
- A `KickOutcome` is recorded for that player.
- The run lasts about 2.5 s for visual replay.
