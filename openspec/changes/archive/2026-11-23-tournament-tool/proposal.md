# Proposal

## Why

The number of team tactics under `teams/` is growing, and there is no way to compare them head-to-head at scale. A single `ynwa-simulator` run plays one match and does not report the score, so evaluating a team today means running matches by hand and reading `.jsonl` footers manually. We need a small tournament tool that runs a full round-robin over a given list of teams and produces a standings table, so team effectiveness (roster plus tactic, fully determined by its `teams/` folder) can be compared at a glance.

## What Changes

- Add `tools/tournament.py`, a Python 3.10 script that runs a round-robin tournament by shelling out to the `ynwa-simulator` binary and parsing the final score from each match recording's footer. Standard library only (`argparse`, `json`, `pathlib`, `subprocess`, `tempfile`).
- Add a thin `tools/run_tournament.sh` wrapper that holds the tournament parameters and invokes `tournament.py` with them, so parameters are edited once and reused.
- The tournament uses a **double round-robin**: each ordered pair `(home, away)` plays once per round, so every unordered pair meets an equal number of times home and away.
- The script builds the release simulator binary before running (`cargo build --release --bin ynwa-simulator`), because development uses debug builds and the binary may not exist yet.
- Parameters come from the command line: team list, teams/preambles directories, simulator binary path, number of rounds, `--duration`, `--timestep`, `--temperature`, optional `--seed`, output directory, run name, and recording retention.
- The seed is random per match by default and is always recorded in the results file; a fixed `--seed` makes the tournament deterministic.
- Results are written as two human-readable, space-aligned files under `~/.ynwa/tournaments/<run>/`: `results.txt` (one match per row, including the seed) and `standings.txt` (played, W/D/L, goals for/against, goal difference, points on a 3-point system, sorted by points → goal difference → goals for → team id).
- Team identifiers are the `teams/` folder names.
- Match recordings are discarded by default; `--keep-recordings` retains them under `matches/`.
- Move `record_game.sh` and `replay.sh` into `tools/` alongside the new tool.

## Capabilities

### New Capabilities

- `tournament`: Round-robin tournament execution over `ynwa-simulator`, score extraction from match recordings, standings computation, and result file output.

### Modified Capabilities

- `player-client`: The root shell wrappers `./record_game.sh` and `./replay.sh` move to `tools/`, changing their documented invocation paths.
- `integration-testing`: The replay-artifact scenario's `./replay.sh` reference moves to `tools/replay.sh`.

## Impact

- New files under `tools/`: `tournament.py`, `run_tournament.sh`, and relocated `record_game.sh`, `replay.sh`.
- No changes to `ynwa-core`, `ynwa-football`, or `ynwa-simulator`; the tournament tool drives the existing simulator binary.
- Output data lives under `~/.ynwa/tournaments/` (a new directory, separate from `~/.ynwa/games`).
- Relocating the shell wrappers updates their `REPO_ROOT` resolution and two spec references.
