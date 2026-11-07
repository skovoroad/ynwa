# goal_scored_a_own

## Purpose

Checks a goal scored into goal A (an own goal by team A).

## Initial state

- Stage: `Play`.
- Ball: free in front of goal A, moving towards the goal, with `last_possessing_team = "A"`.
- Players stand away from the ball path.

## Action

The ball crosses the goal line between the goalposts; no player kicks it.

## Checks

- A `Goal` event is recorded for goal A.
- A `StatUpdate` increases team B's `score` by 1; the score of team A does not change.
- Decisions are reset, a restart is set for the conceding team A at the centre spot, and the game enters `Setup("kick off")`.
