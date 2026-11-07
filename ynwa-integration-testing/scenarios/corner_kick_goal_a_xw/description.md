# corner_kick_goal_a_xw

## Purpose

Checks that the attacking team puts the ball into play from a corner at goal A's `xw` corner by crossing it into the penalty area.

## Initial state

- Stage: `Play`.
- Ball: free at the corner `(68, 0)`.
- Player 1 of team B stands on the ball; all other players stand away.

## Action

The taker receives possession and crosses the ball into the penalty area.

## Checks

- A `PossessionChange` gives the ball to the taker with `last_possessing_team = "B"`.
- A `KickOutcome` is recorded for that player.
- The ball stays in play, so the game remains in `Play` when the run stops (asserted by `final_state.toml`).
- The run lasts about 2.5 s for visual replay.
