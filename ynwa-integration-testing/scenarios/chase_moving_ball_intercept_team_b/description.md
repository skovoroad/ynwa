# chase_moving_ball_intercept_team_b

## Purpose

Mirror of `chase_moving_ball_intercept_team_a`: checks that `chase_ball()` steers the player to the perpendicular lead point of a rolling free ball in team B's mirrored orientation.

This complements `chase_moving_ball_team_b`, where the player and the ball move along the same line and the player arrives only after the ball has stopped.

## Initial state

- Stage: `Play`.
- The ball is free at `(34, 64.61538)` (`(34, 40)` in team B's orientation), rolling with display velocity `(0, -8)` (`(0, 8)` in team B's orientation) and decelerating at the engine's ball friction.
- Player 1 of team B stands at `(48, 52.61538)` (`(20, 52)` in team B's orientation), level with the ball's future lead point, with `speed_rate = 70` (7 m/s, matching `AVG_PLAYER_SPEED`) and `reaction_rate = 10` (a 3.0 s decision interval).
- One team A player stands away from the play.

## Action

- Player 1 receives a single `chase_ball()` decision on the first tick and runs perpendicular to the ball's path towards the computed lead point `ball_pos + ball_vel * t_est`, with `t_est = MAX_LEAD_TIME = 1.5 s`.
- The ball reaches the lead point `(34, 52.61538)` in display coordinates at `t = 2.0 s`, exactly when the player covers the 14 m at 7 m/s.

## Checks

- The recorded `Run` decision targets the lead point `(34, 52.61538)` in display coordinates.
- Player 1 and the ball meet at that point and possession (`PossessionChange`) happens before the 2.6 s safety limit. The ball only stops after `v / a = 4.0 s`, so possession happens while the ball is still moving, and the player runs perpendicular to the ball's path.
- Running the matching `_team_a` and `_team_b` scenarios demonstrates that lead targeting is orientation-independent.
