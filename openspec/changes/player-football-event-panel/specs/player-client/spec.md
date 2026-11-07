# Spec Delta

## MODIFIED Requirements

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `A <score_a> : <score_b> B`, pause hint, and a two-column (`Team A`, `Team B`) `Player Decisions` table (`#`, current decision or `ERROR: <last_error>` / `—`, `last_decision_time`, `decision_reason`). In `Replay` mode ONLY, an `Events:` panel MUST also show the last `5` `FootballEvent`s with `timestamp <= elapsed_time`.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the replay `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, and `GameEnd` renders `"Match end"` (or `"(no events yet)"` before the first event)
- **AND** `GoalLine(position, last)` renders `"Corner; last touch: <last>"` when `last` differs from the attacking team of the crossed goal line, and `"Goal kick; last touch: <last>"` when `last` is that attacking team, where the attacking team is `B` when `position.z < L/2` and `A` otherwise
