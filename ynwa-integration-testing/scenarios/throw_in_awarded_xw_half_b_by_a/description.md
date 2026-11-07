# throw_in_awarded_xw_half_b_by_a

## Purpose

Checks that a ball leaving the field over the `xw` touchline in half `b`, after team A's last touch, is awarded as a throw-in to the opponents.

## Initial state

- Stage: `Play`.
- Ball: free at `(35, 80)`, moving towards the touchline, with `last_possessing_team = "A"`.
- Players stand away from the ball path.

## Action

The ball crosses the touchline; no player kicks it.

## Checks

- A `Touchline` event is recorded for team A.
- Decisions are reset and a restart is set at the crossing point `(68.156, 80)` for team B.
- The game enters `Setup("throw in")`.
