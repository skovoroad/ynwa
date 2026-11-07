# decision_targets_stop_team_b

## Purpose

Checks the `DecisionAssigned` record for a `Stop` decision with a `none` target produced by team B's script.

## Initial state

- Stage: `Play`.
- The deciding player is player 1 of team B; the ball is free and away from all players.

## Action

The player script always returns the same decision, so the assignment is evaluated on the first tick.

## Checks

- A `DecisionAssigned` is recorded for player 1 of team B.
- Its decision kind is `Stop` and its reason is `target:stop`.
- Its target is `absent`; coordinate targets are asserted in display orientation (team B's local target converted to the field orientation).
- The run stops on time (about 2 s) so the recording can be replayed visually.
