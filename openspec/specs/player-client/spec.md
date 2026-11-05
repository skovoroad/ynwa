# Player Client Specification

## Purpose

Defines the interactive `macroquad` desktop client (`ynwa-player`), including CLI modes (`Play`, `Record`, `Replay`) and flag validation, simulation pacing and keyboard bindings, top-down 2D screen projection of the 3D field, zone/sprite rendering, and UI/replay telemetry panels.

## Requirements

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

### Requirement: Top-Down 2D Field Projection and Pitch/Zone Rendering

`ynwa-player` SHALL project Display Orientation 3D coordinates onto the left `65%` of the window (`margin = 20` px, uniform `scale`) via `screen_x = offset_x + x * scale` and `screen_y = offset_y + (L - z) * scale` (`Y` ignored; `Z = 0` / `Team::A` goal at **bottom**, `Z = L` / `Team::B` goal at **top**). `ArcZone`s MUST render accurately across screen Y inversion (converting negative spans via `+ TAU`).

#### Scenario: Grid checkerboard, spreadsheet labels, nets, and corner flags
- **WHEN** the pitch is rendered with uniform fit scale `min((area_w - 40)/W, (screen_h - 40)/L)`
- **THEN** alternating green grid squares, column labels `A..Z..` along the top border, row numbers `1..rows` ascending from bottom (`Z = 0`) to top (`Z = L`) along the left border, off-field goal nets, and animated corner flags are drawn

### Requirement: Pixel-Art Player Sprites and Ball Rendering

Each player SHALL render at their projected `(x, z)` screen position as a pixel-art sprite with team/goalkeeper shirt colors, tactical number, run animation when horizontal `speed > 0.5` m/s, and directional facing (`facing_back` when moving toward `+Z` at `vz > 0.5` m/s or stationary). A standalone field ball MUST be drawn ONLY when `ball_state.possessed_by.is_none()`; otherwise the ball renders attached to its possessor's sprite.

#### Scenario: Shirt colors and possessed ball occlusion by facing direction
- **WHEN** player sprites are rendered
- **THEN** shirt colors are `Team::A` `#1` black, `Team::A` outfielders red, `Team::B` `#1` yellow, `Team::B` outfielders blue; when a player possesses the ball, it renders partially behind the body when facing upfield (`facing_back`) and in front of the feet otherwise

### Requirement: Right-Hand Control Panel and Replay Events Panel

The right `35%` of the window SHALL display match telemetry: elapsed time, `PAUSED`/`Running` status, score `A <score_a> : <score_b> B`, pause hint, and a two-column (`Team A`, `Team B`) `Player Decisions` table (`#`, current decision or `ERROR: <last_error>` / `—`, `last_decision_time`, `decision_reason`). In `Replay` mode ONLY, an `Events:` panel MUST also show the last `5` `FootballEvent`s with `timestamp <= elapsed_time`.

#### Scenario: Replay event labels and scoring team inversion
- **WHEN** the replay `Events:` panel formats decoded `FootballEvent` entries
- **THEN** `Goal(conceding)` renders `"Goal: team <conceding.opposite()>"`, `Touchline(_, last)` renders `"Throw-in; last touch: <last>"`, `GoalLine(_, last)` renders `"Goal line; last touch: <last>"`, and `GameEnd` renders `"Match end"` (or `"(no events yet)"` before the first event)
