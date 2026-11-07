# kick_off_after_goal_placement_team_a

## Purpose

Checks the kick-off placement after a goal when team A conceded and restarts play.

## Initial state

- Stage: `Setup("kick off")`.
- Restart team: `A` (the conceding team).
- Players are placed at arbitrary in-field positions that are not the centres of any tactical set-piece region, unlike the match-start scenario.
- The taker starts 14 m from the centre spot on the centre line.

## Action

Players run to their kick-off positions; the game reaches `Play`.

## Checks

- The taker is assigned a `Run` decision towards the centre spot.
- The taker stops within the arrival threshold of the centre spot; its exact final position is asserted.
- The supporting player of the restarting team ends in its `kick off own` region and both opponents end in their `kick off opp` regions.
- On the transition step the ball is picked up by the taker, so the final ball state is possession by the taker.
