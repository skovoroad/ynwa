# throw_in_placement_x0_half_b_by_a

## Purpose

Checks the throw-in placement phase on the `x0` touchline, in half `b`, with the last touch by team A.

## Initial state

- Stage: `Setup("throw in")`.
- Restart position: `(0, 80)` on the `x0` touchline.
- Restart team: `B` (the opponents of the last-touching team).
- Players start off the field at the sideline and run to their tactical positions; the taker runs to the touchline restart point.

## Action

The setup phase runs until the game reaches `Play`.

## Checks

- The engine selects the throw-in key from the taking team's perspective (own/opp, left/right, own/opp half) for both teams.
- Every non-taking player ends inside the correct tactical region.
- The taker ends on the ball (asserted via `final_state.ball.possessed_by`).
- All players stop before `Play` is recorded, and no scripts are invoked during `Setup` (exact journal).
