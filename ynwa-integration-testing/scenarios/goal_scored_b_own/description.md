# goal_scored_b_own

## Purpose

Checks a goal scored into goal B (an own goal by team B).

## Initial state

- Stage: `Play`.
- Ball: free in front of goal B, moving towards the goal, with `last_possessing_team = "B"`.
- Players stand away from the ball path.

## Action

The ball crosses the goal line between the goalposts; no player kicks it.

## Checks

- A `Goal` event is recorded for goal B.
- A `StatUpdate` increases team A's `score` by 1; the score of team B does not change.
- Decisions are reset, a restart is set for the conceding team B at the centre spot, and the game enters `Setup("kick off")`.
