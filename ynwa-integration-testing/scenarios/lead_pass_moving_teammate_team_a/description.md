# lead_pass_moving_teammate_team_a

## Purpose

Checks that a pass by team A leads a moving teammate instead of targeting their snapshot position.

## Initial state

- Stage: `Play`.
- Player 1 of team A owns the ball at `(34, 70)`.
- Player 2 of team A stands at `(34, 40)` and runs straight towards `(34, 95)` under `team_has_ball`.
- One team B player stands away from the play.

## Action

- Player 2 starts running on the first tick.
- Player 1 holds the ball for its first decision and then calls `pass_to(2)`, so player 2 already has a non-zero velocity when the kick is taken.
- `pass_to` leads player 2 by `min(distance / AVG_PASS_SPEED, MAX_LEAD_TIME)` and aims the kick at that point.

## Checks

- Player 1 records a `KickOutcome`.
- Player 2 receives the ball (`PossessionChange`) after the 1.0 s `POSSESSION_COOLDOWN`, because the leading pass intersects his running path within `POSSESSION_RADIUS`.
- The matching `_team_b` scenario mirrors the same play in team B's orientation.
