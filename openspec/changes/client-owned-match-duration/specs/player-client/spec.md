# Spec Delta

## MODIFIED Requirements

### Requirement: CLI Modes, Flag Validation, and Recording Paths

`ynwa-player` SHALL parse `[<teams_path="teams">] [<preambles_path="ynwa-scripts/preambles">] [--duration <seconds=120.0>] [--record [--out <dir>]] [--replay <file>]` into `Play`, `Record`, or `Replay` mode. It MUST reject duplicate flags, `--record` with `--replay`, `--out` without `--record`, `--duration` with `--replay`, non-positive/non-finite `--duration`, unknown `--*` flags, or `> 2` positional arguments. `--record` MUST stream JSON Lines output to `<out_or_~/.ynwa/games>/game_YYYYMMDD-HHMMSS[_2..].jsonl`.

#### Scenario: Mutually exclusive CLI flags and root shell wrappers
- **WHEN** `ynwa-player` is invoked directly or via `./record_game.sh` / `./replay.sh <recording.jsonl>`
- **THEN** `./record_game.sh` forwards arguments to `ynwa-player --record`, `./replay.sh` opens the `.jsonl` recording in replay mode, and invalid flag combinations (`--record --replay`, `--out` without `--record`, `--duration` with `--replay`, or `--duration <= 0`) print an error to `stderr` and exit with code `1`

#### Scenario: Truncated replay step count recovery
- **WHEN** `ynwa-player` loads a crashed recording via `--replay` where `record.total_steps == 0`
- **THEN** it computes `effective_steps = ceil(last_journal_timestamp / timestep) as u64` (`0` if `journal` is empty) so playback advances through all recovered events

### Requirement: Simulation Pacing and Keyboard Controls

`ynwa-player` SHALL open a resizable `1200 × 900` window (`"YNWA Football Simulator"`). `Play` and `Record` start at `60.0` Hz (`timestep = 1/60` s) and advance simulation steps until `elapsed_time >= duration` (default `120.0` s; `Play` stops stepping while keeping the window open, whereas `Record` flushes the recording footer and exits); `Replay` runs at `rate = 1.0 / timestep` and stops stepping at `effective_steps`. `Space` pauses/resumes, `F11` toggles fullscreen, `Escape` exits cleanly (flushing the recording footer in `Record`), and `+`/`=`/`KP_ADD` (`*2` up to `100` Hz) and `-`/`KP_SUBTRACT` (`/2` down to `1` Hz) apply **ONLY in `Play` mode**.

#### Scenario: Preventing timestep drift in Record and Replay modes
- **WHEN** `+` or `-` keys are pressed in `Record` or `Replay` mode
- **THEN** playback rate and simulation `timestep` remain locked so steps stay aligned with `RecordHeader::timestep`

#### Scenario: Client-owned match duration termination in Play and Record modes
- **WHEN** `world.game().state().elapsed_time >= duration` is reached in `Play` or `Record` mode
- **THEN** `Play` mode ceases advancing `World::step` while continuing to render the final state, and `Record` mode calls `finish_journal()` and exits cleanly

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `A <score_a> : <score_b> B`, pause hint, and a two-column (`Team A`, `Team B`) `Player Decisions` table (`#`, current decision or `ERROR: <last_error>` / `—`, `last_decision_time`, `decision_reason`). In `Replay` mode ONLY, an `Events:` panel MUST show every journaled `JournalEvent` with `timestamp <= elapsed_time` in journal order and MUST be scrollable through the full history.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the replay `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, and `GoalLine(_, last)` renders `"Goal line; last touch: <last>"` (or `"(no events yet)"` before the first event)

#### Scenario: All journal event variants are shown
- **WHEN** a replay recording contains non-football `JournalEvent` variants (`DecisionAssigned`, `PossessionChange`, `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`) alongside football `External` entries
- **THEN** the `Events:` panel lists all of them in journal order with `timestamp <= elapsed_time`, not only the football `External` entries

#### Scenario: Scrolling the event history
- **WHEN** the `Events:` panel has more entries than fit in its viewport
- **THEN** the mouse wheel and `Up`/`Down` keys scroll the visible window through the full history, and while the panel is scrolled to the newest entry it auto-follows newly arrived events

