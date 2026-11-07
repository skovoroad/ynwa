# throw_in

## Purpose

Legacy preliminary scenario: a ball driven over the touchline after team A's last touch must produce a throw-in for team B at the crossing point.

## Initial state

- Stage: `Play`.
- Ball: free at `(10, 0, 52.3)`, moving towards the `x = width` touchline with `last_possessing_team = "A"`.
- Two players stand still.

## Action

The ball crosses the touchline; no player kicks it.

## Checks

- A `Touchline` event is recorded for team A.
- The game enters `Setup("throw in")` with team B restarting.
