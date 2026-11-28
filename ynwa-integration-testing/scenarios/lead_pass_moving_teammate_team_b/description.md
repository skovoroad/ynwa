# lead_pass_moving_teammate_team_b

## Purpose

Mirror of `lead_pass_moving_teammate_team_a`: checks that a pass by team B leads a moving teammate in team B's mirrored orientation.

## Initial state

- Stage: `Play`.
- Player 1 of team B owns the ball at `(58, 64.61538)` (`(10, 40)` in team B's orientation).
- Player 2 of team B stands at `(34, 64.61538)` (`(34, 40)` in team B's orientation) and runs towards `(34, 9.61538)` (`(34, 95)` in team B's orientation) under `team_has_ball`.
- One team A player stands away from the play.

## Action

- Player 2 starts running on the first tick.
- Player 1 holds the ball for its first decision and then calls `pass_to(2)`, so player 2 already has a non-zero velocity when the kick is taken.
- `pass_to` leads player 2 by `min(distance / AVG_PASS_SPEED, MAX_LEAD_TIME)` and aims the kick at that point.

## Checks

- Player 1 records a `KickOutcome`.
- Player 2 receives the ball (`PossessionChange`) after the 1.0 s `POSSESSION_COOLDOWN`, because the leading pass intersects his running path within `POSSESSION_RADIUS`.
- Running the matching `_team_a` and `_team_b` scenarios demonstrates that lead targeting is orientation-independent.
