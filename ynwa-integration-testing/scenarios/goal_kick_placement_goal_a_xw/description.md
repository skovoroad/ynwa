# goal_kick_placement_goal_a_xw

## Purpose

Checks the goal-kick placement phase at goal A.

## Initial state

- Stage: `Setup("goal kick")`.
- Restart position: the 5.5 m line at goal A (`(34, 5.5)`).
- Restart team: `A` (the defending team takes the goal kick).
- Players start off the field at the sideline and run to their tactical positions; the taker runs to the restart point.

## Action

The setup phase runs until the game reaches `Play`.

## Checks

- The engine selects `goal kick own` for the defending team and `goal kick opp` for the attacking team.
- Every non-taking player ends inside the correct tactical region.
- The taker ends on the ball (asserted via `final_state.ball.possessed_by`).
- All players stop before `Play` is recorded, and no scripts are invoked during `Setup` (exact journal).
