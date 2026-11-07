# stdlib_kick_to_goal

## Purpose

Legacy preliminary scenario: a player scripted with the standard library's `kick_to_opponent_goal()` scores.

## Initial state

- Stage: `Play`.
- Player 1 of team A owns the ball at `(34, 0, 82)` in front of goal B.
- Player 1 of team B stands at `(34, 0, 10)`.

## Action

Team A's script returns a kick towards the opponent's goal.

## Checks

- A `KickOutcome` is recorded for team A's player.
- A `Goal` event is recorded for goal B.
- The score becomes `A = 1, B = 0`.
- The game enters `Setup("kick off")`.
