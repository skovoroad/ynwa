# Spec Delta

## MODIFIED Requirements

### Requirement: CLI Modes, Flag Validation, and Recording Paths

`ynwa-player` SHALL parse `[<teams_path="teams">] [<preambles_path="ynwa-scripts/preambles">] [--record [--out <dir>]] [--replay <file>]` into `Play`, `Record`, or `Replay` mode. It MUST reject duplicate flags, `--record` with `--replay`, `--out` without `--record`, unknown `--*` flags, or `> 2` positional arguments. `--record` MUST stream JSON Lines output to `<out_or_~/.ynwa/games>/game_YYYYMMDD-HHMMSS[_2..].jsonl`.

#### Scenario: Mutually exclusive CLI flags and root shell wrappers
- **WHEN** `ynwa-player` is invoked directly or via `./record_game.sh` / `./replay.sh <recording.jsonl>`
- **THEN** `./record_game.sh` forwards arguments to `ynwa-player --record`, `./replay.sh` opens the `.jsonl` recording in replay mode, and invalid flag combinations (`--record --replay` or `--out` without `--record`) print an error to `stderr` and exit with code `1`

#### Scenario: Truncated replay step count recovery
- **WHEN** `ynwa-player` loads a crashed recording via `--replay` where `record.total_steps == 0`
- **THEN** it computes `effective_steps = ceil(last_journal_timestamp / timestep) as u64` (`0` if `journal` is empty) so playback advances through all recovered events

### Requirement: Simulation Pacing and Keyboard Controls

`ynwa-player` SHALL open a resizable `1200 × 900` window (`"YNWA Football Simulator"`). `Play` and `Record` start at `60.0` Hz (`timestep = 1/60` s); `Replay` runs at `rate = 1.0 / timestep` and stops stepping at `effective_steps`. `Space` pauses/resumes, `F11` toggles fullscreen, `Escape` exits cleanly (flushing the recording footer in `Record`), and `+`/`=`/`KP_ADD` (`*2` up to `100` Hz) and `-`/`KP_SUBTRACT` (`/2` down to `1` Hz) apply **ONLY in `Play` mode**.

#### Scenario: Preventing timestep drift in Record and Replay modes
- **WHEN** `+` or `-` keys are pressed in `Record` or `Replay` mode
- **THEN** playback rate and simulation `timestep` remain locked so steps stay aligned with `RecordHeader::timestep`
