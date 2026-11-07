# pass_received_team_a

## Purpose

Checks that a pass by team A is received by the open partner.

## Initial state

- Stage: `Play`.
- Player 1 of team A owns the ball at `(34, 42)`.
- Player 2 of team A is 9 m ahead on the pass lane.
- Opponents stand away from the lane.

## Action

Player 1 passes to player 2.

## Checks

- The pass records a `KickOutcome` for player 1.
- Player 2 receives the ball at `t = 1.04 s`, more than the 1.0 s `POSSESSION_COOLDOWN` after the kick.
- The `PossessionChange` gives the ball to player 2 and keeps `last_possessing_team` as team A.
