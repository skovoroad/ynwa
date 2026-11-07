# decision_targets_kick_point_team_a

## Purpose

Checks the `DecisionAssigned` record for a `Kick` decision with a `point` target produced by team A's script.

## Initial state

- Stage: `Play`.
- The deciding player is player 1 of team A; the ball is free and away from all players.

## Action

The player script always returns the same decision, so the assignment is evaluated on the first tick.

## Checks

- A `DecisionAssigned` is recorded for player 1 of team A.
- Its decision kind is `Kick` and its reason is `target:kick`.
- Its target is `point`; coordinate targets are asserted in display orientation (team A's local target converted to the field orientation).
- The run stops on time (about 2 s) so the recording can be replayed visually.
