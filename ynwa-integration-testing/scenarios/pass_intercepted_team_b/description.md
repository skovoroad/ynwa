# pass_intercepted_team_b

## Purpose

Checks that a pass by team B is intercepted by an opponent standing on the pass lane.

## Initial state

- Stage: `Play`.
- Player 1 of team B owns the ball at `(34, 62.61538)`.
- Player 2 of team B is 18 m ahead on the pass lane.
- An opponent stands on the lane 9 m from the passer.

## Action

Player 1 passes to player 2; the ball passes through the interceptor's position.

## Checks

- The pass records a `KickOutcome` for player 1.
- The opponent receives the ball at `t = 1.04 s`, more than the 1.0 s `POSSESSION_COOLDOWN` after the kick.
- The `PossessionChange` gives the ball to the opponent and flips `last_possessing_team` to team A.
