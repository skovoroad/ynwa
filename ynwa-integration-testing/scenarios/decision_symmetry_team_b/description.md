# decision_symmetry_team_b

## Purpose

Checks that coordinate-dependent decisions made by team B are mirrored into display orientation, making them symmetric with the other team's decisions.

## Initial state

- Stage: `Play`.
- Players 1-4 of team B always return a point, cell, region and kick decision respectively; all other players stop.
- The ball is free at the centre spot and away from all players.

## Action

Every scripted player decides on the first tick.

## Checks

- Player 1 is assigned `Run` to the point target, in display coordinates.
- Player 2 is assigned `Run` to the cell target, in display coordinates.
- Player 3 is assigned `Run` to the region target, in display coordinates.
- Player 4 is assigned `Kick` to the point target, in display coordinates.
- The team-local coordinates used by the scripts are flipped exactly once by the engine; running the matching `_team_a` and `_team_b` scenarios demonstrates the symmetry.
- The run stops on time (about 2 s) for visual replay.
