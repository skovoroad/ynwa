# Spec Delta

## MODIFIED Requirements

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `A <score_a> : <score_b> B`, pause hint, and a two-column (`Team A`, `Team B`) `Player Decisions` table (`#`, current decision or `ERROR: <last_error>` / `—`, `last_decision_time`, `decision_reason`). In `Replay` mode ONLY, an `Events:` panel MUST show every journaled `JournalEvent` with `timestamp <= elapsed_time` in journal order and MUST be scrollable through the full history.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the replay `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, `GoalLine(_, last)` renders `"Goal line; last touch: <last>"`, and `GameEnd` renders `"Match end"` (or `"(no events yet)"` before the first event)

#### Scenario: All journal event variants are shown
- **WHEN** a replay recording contains non-football `JournalEvent` variants (`DecisionAssigned`, `PossessionChange`, `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`) alongside football `External` entries
- **THEN** the `Events:` panel lists all of them in journal order with `timestamp <= elapsed_time`, not only the football `External` entries

#### Scenario: Scrolling the event history
- **WHEN** the `Events:` panel has more entries than fit in its viewport
- **THEN** the mouse wheel and `Up`/`Down` keys scroll the visible window through the full history, and while the panel is scrolled to the newest entry it auto-follows newly arrived events
