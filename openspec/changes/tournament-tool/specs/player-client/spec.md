# Spec Delta

## MODIFIED Requirements

### Requirement: CLI Modes, Flag Validation, and Recording Paths

`ynwa-player` SHALL parse `[<teams_path="teams">] [<preambles_path="ynwa-scripts/preambles">] [--team-a <id="team_a">] [--team-b <id="team_b">] [--duration <seconds=120.0>] [--record [--out <dir>]] [--replay <file>]` into `Play`, `Record`, or `Replay` mode. It MUST reject duplicate flags, `--record` with `--replay`, `--out` without `--record`, `--duration` with `--replay`, `--team-a` or `--team-b` with `--replay`, non-positive/non-finite `--duration`, unknown `--*` flags, or `> 2` positional arguments. `--record` MUST stream JSON Lines output to `<out_or_~/.ynwa/games>/game_YYYYMMDD-HHMMSS[_2..].jsonl`.

#### Scenario: Mutually exclusive CLI flags and root shell wrappers
- **WHEN** `ynwa-player` is invoked directly or via `./tools/record_game.sh` / `./tools/replay.sh <recording.jsonl>`
- **THEN** `./tools/record_game.sh` forwards arguments to `ynwa-player --record`, `./tools/replay.sh` opens the `.jsonl` recording in replay mode, and invalid flag combinations (`--record --replay`, `--out` without `--record`, `--duration` with `--replay`, `--duration <= 0`, or `--team-a`/`--team-b` with `--replay`) print an error to `stderr` and exit with code `1`

#### Scenario: Selecting custom team directories in Play and Record modes
- **WHEN** `ynwa-player` is invoked with `--team-a high_press --team-b deep_block`
- **THEN** it loads `<teams_path>/high_press` as `Team::A` and `<teams_path>/deep_block` as `Team::B`

#### Scenario: Truncated replay step count recovery
- **WHEN** `ynwa-player` loads a crashed recording via `--replay` where `record.footer.total_steps == 0`
- **THEN** it computes `effective_steps = ceil(last_journal_timestamp / timestep) as u64` (`0` if `journal` is empty) so playback advances through all recovered events
