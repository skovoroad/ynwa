# kick_off_placement_team_a

## Purpose

Checks the match-start kick-off placement phase when team A restarts.

## Initial state

- Stage: `Setup("kick off")`.
- Restart team: `A`.
- Players start off the field at the sideline and run to their tactical positions; the taker runs to the centre spot.

## Action

The game runs the setup phase until it reaches `Play`.

## Checks

- The engine assigns `Run` decisions with no script reason during `Setup` (every `DecisionAssigned` before the stage change has `reason = "none"`), so user scripts are not invoked during the setup phase.
- All players stop before `Play` is recorded.
- The taking player ends on the ball (asserted via `final_state.ball.possessed_by`).
- The supporting player of the taking team stands inside its `kick off own` region; the opponents stand inside their `kick off opp` regions.
- The ball is at the centre and free in `Setup`; the first `PossessionChange` is recorded only after the transition to `Play` (exact journal).
