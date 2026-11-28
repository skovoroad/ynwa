# chase_moving_ball_intercept_team_a

## Purpose

Checks that `chase_ball()` steers the player to the lead point of a rolling free ball when the player's and the ball's paths are perpendicular, so a wrong lead direction or magnitude makes the player miss.

This complements `chase_moving_ball_team_a`, where the player and the ball move along the same line and the player arrives only after the ball has stopped.

## Initial state

- Stage: `Play`.
- The ball is free at `(34, 40)`, rolling towards the opponent goal with velocity `(0, 8)` and decelerating at the engine's ball friction.
- Player 1 of team A stands at `(20, 52)`, level with the ball's future lead point and 14 m to its left, with `speed_rate = 70` (7 m/s, matching `AVG_PLAYER_SPEED`) and `reaction_rate = 10` (a 3.0 s decision interval).
- One team B player stands away from the play.

## Action

- Player 1 receives a single `chase_ball()` decision on the first tick and runs perpendicular to the ball's path towards the computed lead point `ball_pos + ball_vel * t_est`, with `t_est = MAX_LEAD_TIME = 1.5 s`.
- The ball reaches the lead point `(34, 52)` after `8t - t² = 12`, i.e. at `t = 2.0 s`, exactly when the player covers the 14 m at 7 m/s.

## Checks

- The recorded `Run` decision targets the lead point `(34, 52)`.
- Player 1 and the ball meet at that point and possession (`PossessionChange`) happens before the 2.6 s safety limit. The ball only stops after `v / a = 4.0 s`, so possession happens while the ball is still moving, and the player runs along X while the ball rolls along Z.
- The matching `_team_b` scenario mirrors the same play in team B's orientation.
