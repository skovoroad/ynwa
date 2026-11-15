# Spec Delta

## MODIFIED Requirements

### Requirement: CLI Modes, Flag Validation, and Recording Paths

`ynwa-player` SHALL parse `[<teams_path="teams">] [<preambles_path="ynwa-scripts/preambles">] [--team-a <id="team_a">] [--team-b <id="team_b">] [--duration <seconds=120.0>] [--record [--out <dir>]] [--replay <file>]` into `Play`, `Record`, or `Replay` mode. It MUST reject duplicate flags, `--record` with `--replay`, `--out` without `--record`, `--duration` with `--replay`, `--team-a` or `--team-b` with `--replay`, non-positive/non-finite `--duration`, unknown `--*` flags, or `> 2` positional arguments. `--record` MUST stream JSON Lines output to `<out_or_~/.ynwa/games>/game_YYYYMMDD-HHMMSS[_2..].jsonl`.

#### Scenario: Mutually exclusive CLI flags and root shell wrappers
- **WHEN** `ynwa-player` is invoked directly or via `./record_game.sh` / `./replay.sh <recording.jsonl>`
- **THEN** `./record_game.sh` forwards arguments to `ynwa-player --record`, `./replay.sh` opens the `.jsonl` recording in replay mode, and invalid flag combinations (`--record --replay`, `--out` without `--record`, `--duration` with `--replay`, `--duration <= 0`, or `--team-a`/`--team-b` with `--replay`) print an error to `stderr` and exit with code `1`

#### Scenario: Selecting custom team directories in Play and Record modes
- **WHEN** `ynwa-player` is invoked with `--team-a high_press --team-b deep_block`
- **THEN** it loads `<teams_path>/high_press` as `Team::A` and `<teams_path>/deep_block` as `Team::B`

#### Scenario: Truncated replay step count recovery
- **WHEN** `ynwa-player` loads a crashed recording via `--replay` where `record.footer.total_steps == 0`
- **THEN** it computes `effective_steps = ceil(last_journal_timestamp / timestep) as u64` (`0` if `journal` is empty) so playback advances through all recovered events

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `<team_a_display_name> <score_a> : <score_b> <team_b_display_name>`, pause hint, and a two-column (`<team_a_display_name>`, `<team_b_display_name>`) `Player Decisions` table (`#`, current decision or `ERROR: <last_error>` / `—`, `last_decision_time`, `decision_reason`). In `Replay` mode ONLY, an `Events:` panel MUST show every journaled `JournalEvent` with `timestamp <= elapsed_time` in journal order and MUST be scrollable through the full history.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the replay `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, `GoalLine(_, last)` renders `"Goal line; last touch: <last>"`, and `GameEnd` renders `"Match end"` (or `"(no events yet)"` before the first event)

#### Scenario: All journal event variants are shown
- **WHEN** a replay recording contains non-football `JournalEvent` variants (`DecisionAssigned`, `PossessionChange`, `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`) alongside football `External` entries
- **THEN** the `Events:` panel lists all of them in journal order with `timestamp <= elapsed_time`, not only the football `External` entries

#### Scenario: Scrolling the event history
- **WHEN** the `Events:` panel has more entries than fit in its viewport
- **THEN** the mouse wheel and `Up`/`Down` keys scroll the visible window through the full history, and while the panel is scrolled to the newest entry it auto-follows newly arrived events
