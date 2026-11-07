# corner_placement_goal_b_xw

## Purpose

Checks the corner placement phase for a corner at the `xw` corner next to goal B.

## Initial state

- Stage: `Setup("corner")`.
- Restart position: the `xw` corner of goal B (`(68, 104.61538)`).
- Restart team: `A` (the attacking team takes the corner).
- Players start off the field at the sideline and run to their tactical positions; the taker runs to the corner.

## Action

The setup phase runs until the game reaches `Play`.

## Checks

- The engine selects the `corner own left/right` key for the attacking team and `corner opp left/right` for the defending team (B), according to each team's own orientation.
- Every non-taking player ends inside the correct tactical region.
- The taker ends on the ball (asserted via `final_state.ball.possessed_by`).
- All players stop before `Play` is recorded, and no scripts are invoked during `Setup` (exact journal; script decisions only appear after the stage change).
