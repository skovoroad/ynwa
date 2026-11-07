# throw_in_awarded_x0_half_a_by_a

## Purpose

Checks that a ball leaving the field over the `x0` touchline in half `a`, after team A's last touch, is awarded as a throw-in to the opponents.

## Initial state

- Stage: `Play`.
- Ball: free at `(33, 20)`, moving towards the touchline, with `last_possessing_team = "A"`.
- Players stand away from the ball path.

## Action

The ball crosses the touchline; no player kicks it.

## Checks

- A `Touchline` event is recorded for team A.
- Decisions are reset and a restart is set at the crossing point `(-0.156, 20)` for team B.
- The game enters `Setup("throw in")`.
