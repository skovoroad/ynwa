# chase_moving_ball_team_b

## Purpose

Mirror of `chase_moving_ball_team_a`: checks that `chase_ball()` intercepts a rolling free ball in team B's mirrored orientation.

## Initial state

- Stage: `Play`.
- The ball is free at `(34, 64.61538)` (`(34, 40)` in team B's orientation), rolling with display velocity `(-2.4, -3.2)` (`(2.4, 3.2)` in team B's orientation) and decelerating at the engine's ball friction.
- Player 1 of team B stands at `(34, 79.61538)` (`(34, 25)` in team B's orientation) with `speed_rate = 70` (7 m/s, matching `AVG_PLAYER_SPEED`) and `reaction_rate = 10` (a 3.0 s decision interval).
- One team A player stands away from the play.

## Action

- Player 1 receives a single `chase_ball()` decision on the first tick and runs to the computed lead point `ball_pos + ball_vel * t_est`, where `t_est` is capped by half the ball's stopping time. The oblique velocity exercises both lead components.
- The ball stops at that lead point shortly afterwards.

## Checks

- The recorded `Run` decision targets the lead point `(31.6, 61.41538)` in display coordinates.
- Player 1 reaches the lead point and gains possession (`PossessionChange`) before his next decision tick.
- Running the matching `_team_a` and `_team_b` scenarios demonstrates that lead targeting is orientation-independent.
