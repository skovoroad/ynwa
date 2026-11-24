# Spec Delta

## MODIFIED Requirements

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `<team_a_display_name> <score_a> : <score_b> <team_b_display_name>`, and pause hint. In all modes (`Play`, `Record`, `Replay`), a scrollable `Events:` panel SHALL show every journaled `JournalEvent` with `timestamp <= elapsed_time` in journal order, expanded to fill the remaining panel height, and MUST be scrollable through the full history. The two-column `Player Decisions` table SHALL be removed.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, `GoalLine(_, last)` renders `"Goal line; last touch: <last>"`, and `GameEnd` renders `"Match end"` (or `"(no events yet)"` before the first event)

#### Scenario: All journal event variants are shown
- **WHEN** a run or recording contains non-football `JournalEvent` variants (`DecisionAssigned`, `PossessionChange`, `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`) alongside football `External` entries
- **THEN** the `Events:` panel lists all of them in journal order with `timestamp <= elapsed_time`, not only the football `External` entries

#### Scenario: Scrolling the event history
- **WHEN** the `Events:` panel has more entries than fit in its viewport
- **THEN** the mouse wheel and `Up`/`Down` keys scroll the visible window through the full history, `PgUp`/`PgDown` scroll by a full viewport, and while the panel is scrolled to the newest entry it auto-follows newly arrived events

#### Scenario: Live events appear in Play and Record modes
- **WHEN** the simulation advances in `Play` or `Record` mode
- **THEN** the `Events:` panel shows the journaled events as they occur, in the same panel position as in `Replay` mode

#### Scenario: Recording writes the file and shows events simultaneously
- **WHEN** `Record` mode runs
- **THEN** each journal event is both streamed to the `.jsonl` output file and displayed in the `Events:` panel
