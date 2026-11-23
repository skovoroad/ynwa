# Tasks

## 1. Relocate shell wrappers to tools/

- [x] 1.1 Create `tools/` and move `record_game.sh` and `replay.sh` into it; verify both files exist under `tools/` and no longer at the repo root
- [x] 1.2 Update `REPO_ROOT` in both scripts to resolve from `$(dirname "$0")/..`; verify `bash -n` passes on both and the computed repo root points at the project root when invoked from any directory

## 2. Implement tools/tournament.py

- [x] 2.1 Add argparse CLI with all parameters and validation (team list with at least two existing folders, `rounds >= 1`, finite positive `--duration`/`--timestep`, `--temperature` in `[0,1]`, optional `--seed`, output dir, run name, retention flag); verify `--help` lists them and malformed inputs print an error and exit non-zero
- [x] 2.2 Implement the double round-robin schedule generator; verify a 3-team schedule yields 6 ordered pairs per round and each unordered pair appears twice per round (once per side)
- [x] 2.3 Implement match execution: invoke the `ynwa-simulator` binary (home as `--team-a`, away as `--team-b`, explicit `--out`, per-match seed) via `subprocess`; verify a 2-team single-round run spawns the simulator and produces a recording
- [x] 2.4 Implement score extraction by parsing the recording footer's `team_stats` and mapping `A`/`B` to home/away, aborting on a non-zero simulator exit; verify the extracted scores match the recording for a known seed
- [x] 2.5 Implement standings accumulation and ordering (points, goal difference, goals for, team id); verify totals and ordering across a small multi-team run
- [x] 2.6 Implement writing space-aligned `results.txt` and `standings.txt` under `<output>/<run>/`; verify both files exist and columns align
- [x] 2.7 Implement recording retention (temp directory discard by default, `matches/` when `--keep-recordings`); verify no recordings remain by default and recordings persist with the flag

## 3. Add tools/run_tournament.sh wrapper

- [x] 3.1 Create `run_tournament.sh` that sets parameters, runs `cargo build --release --bin ynwa-simulator`, and calls `tools/tournament.py`; verify `bash -n` passes and the release `ynwa-simulator` binary exists after the build step

## 4. End-to-end verification

- [x] 4.1 Run a full tournament over `team_a team_b team_c` with a small round count; verify `results.txt` and `standings.txt` are produced under `~/.ynwa/tournaments/<run>/` with correct scores and standings
