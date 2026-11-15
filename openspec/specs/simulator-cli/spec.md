# Simulator CLI Specification

## Purpose

Defines the standalone headless match simulator CLI (`ynwa-simulator`) for fast model-time match execution, configurable team/RNG/stepping/duration parameters, and `.jsonl` match recording output with descriptive filenames.

## Requirements

### Requirement: Headless Simulator CLI Flags and Validation

`ynwa-simulator` SHALL parse `--teams <dir="teams">`, `--preambles <dir="ynwa-scripts/preambles">`, `--team-a <id="team_a">`, `--team-b <id="team_b">`, `--out <path>`, `--seed <u64>`, `--temperature <f32=0.7>`, `--timestep <f32=1/60>`, and `--duration <f32=120.0>`. It MUST reject duplicate flags, unknown flags, positional arguments, `--temperature` outside `0.0..=1.0`, non-finite or `<= 0.0` `--timestep`, or non-finite or `<= 0.0` `--duration` with an error to `stderr` and exit code `1`.

#### Scenario: Default CLI execution and flag validation
- **WHEN** `ynwa-simulator` is invoked with no arguments or valid flag overrides
- **THEN** it defaults to `teams/`, `ynwa-scripts/preambles`, `team_a`, `team_b`, `temperature = 0.7`, `timestep = 1.0 / 60.0`, and `duration = 120.0`
- **AND** invalid flags, duplicate flags, or out-of-range numeric values print an error to `stderr` and exit with code `1`

### Requirement: Client-Driven Model-Time Simulation Loop

`ynwa-simulator` SHALL assemble a live football `World` via `FootballWorldBuilder` (without placeholder decision fallback, failing immediately on Lua initialization errors) and advance `World::step(timestep)` synchronously in model time without frame pacing while `world.game().state().elapsed_time < duration`, then call `finish_journal()` and exit with code `0`.

#### Scenario: Normal match completion and seeded determinism
- **WHEN** two `ynwa-simulator` runs execute on the same platform with identical teams, `--seed`, `--temperature`, `--timestep`, and `--duration`
- **THEN** both runs step until `elapsed_time >= duration`, finalize their `.jsonl` recordings with identical events and `RecordFooter` statistics, and exit with code `0`

### Requirement: Custom Recording Output Paths and Descriptive Filenames

`ynwa-simulator` SHALL stream the match recording as JSON Lines using `FileJournalRecorder`:
- If `--out <path>` ends with `.jsonl`, it writes directly to that file path (creating parent directories if needed).
- Otherwise, it treats `--out <dir>` (or default `<home>/.ynwa/games` when `--out` is omitted) as a directory and writes to `<dir>/<team_a>_vs_<team_b>_YYYYMMDD-HHMMSS[_2..].jsonl`.

#### Scenario: Explicit file path vs directory descriptive auto-naming
- **WHEN** `ynwa-simulator --team-a pressing --team-b counter --out /tmp/tourney/match01.jsonl` is run
- **THEN** the recording is written to `/tmp/tourney/match01.jsonl`
- **AND** when `--out /tmp/tourney` is passed instead, the recording is written to `/tmp/tourney/pressing_vs_counter_YYYYMMDD-HHMMSS.jsonl` (appending `_2`, `_3`, ... on filename collision)
