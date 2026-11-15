# Spec Delta

## MODIFIED Requirements

### Requirement: Football Event Detection Priority and Boundary Conditions

During `GameStage::Play` ONLY, `check_events(game)` SHALL return at most **one** `FootballEvent` per step in priority order: (1) `Goal(conceding_team)` (scorer is `conceding_team.opposite()`), (2) `Touchline(ball_pos, last_team)`, (3) `GoalLine(ball_pos, last_team)`. Boundary checks MUST account for `BALL_RADIUS = 0.11` m, and `last_team` uses `last_possessing_team` (defaulting to `Team::A` when `None`).

#### Scenario: Exact geometric predicates for Goal, Touchline, and GoalLine
- **WHEN** `check_events` evaluates ball center `(bx, by, bz)` with `r = 0.11` m and goal X-span `[gx_min, gx_max]`
- **THEN** `Goal(team)` triggers iff `bz - r >= gz_min && bz + r <= gz_max && gx_min <= bx && bx <= gx_max`; `Touchline` triggers iff `bx + r < 0.0 || bx - r > W`; and `GoalLine` triggers iff `(bz + r < 0.0 || bz - r > L)` AND `!(gx_min <= bx && bx <= gx_max)`

### Requirement: FootballGameManager Event Transitions and Restart Assignment

When `check_events` yields an event in `Play`, `FootballGameManager` SHALL first emit `JournalEvent::External { kind: "football_event", data }` (`{"event": "goal"|"touchline"|"goal_line", "team": "A"|"B"}`) and then apply stage/restart transitions: `Goal`, `Touchline`, and `GoalLine` reset all players (`is_ready=false, current_decision=None, needs_decision=true`, emitting `DecisionsReset`), emit `RestartSet` and `StageChange`, and enter `Setup(reason)`.

#### Scenario: Restart parameters for Goal, Touchline, Goal Kick, and Corner
- **WHEN** `FootballGameManager` handles a boundary or scoring event on pitch `(W, L)`
- **THEN** `Goal(conceding)` increments `team_stats[conceding.opposite()]["score"]` by `1.0` (`StatUpdate`) and sets `Setup("kick off")` with `restart_position = None`, `restart_team = Some(conceding)`
- **AND** `Touchline(pos, last)` sets `Setup("throw in")` with `restart_position = Some(pos)`, `restart_team = Some(last.opposite())`
- **AND** `GoalLine(pos, last)` (with `attacking = B` if `pos.z < L/2` else `A`) sets `Setup("goal kick")` at `(W/2, 0, 5.5)` or `(W/2, 0, L - 5.5)` with `restart_team = Some(last.opposite())` when `last == attacking`, or `Setup("corner")` at nearest corner `{(0,0,0), (W,0,0), (0,0,L), (W,0,L)}` with `restart_team = Some(attacking)` when `last != attacking`
- **AND** `restart_position` and `restart_team` are latched before `Setup` ticks clear `ball_state.last_possessing_team = None`

