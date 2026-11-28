# chase_moving_ball_team_a

## Purpose

Checks that `chase_ball()` intercepts a rolling free ball by running to its lead point instead of its snapshot position.

## Initial state

- Stage: `Play`.
- The ball is free at `(34, 40)`, rolling obliquely with velocity `(2.4, 3.2)` and decelerating at the engine's ball friction.
- Player 1 of team A stands at `(34, 25)` with `speed_rate = 70` (7 m/s, matching `AVG_PLAYER_SPEED`) and `reaction_rate = 10` (a 3.0 s decision interval).
- One team B player stands away from the play.

## Action

- Player 1 receives a single `chase_ball()` decision on the first tick and runs to the computed lead point `ball_pos + ball_vel * t_est`, where `t_est` is capped by half the ball's stopping time. The oblique velocity exercises both lead components.
- The ball stops at that lead point shortly afterwards.

## Checks

- The recorded `Run` decision targets the lead point `(36.4, 43.2)`.
- Player 1 reaches the lead point and gains possession (`PossessionChange`) before his next decision tick.
- The matching `_team_b` scenario mirrors the same play in team B's orientation.
