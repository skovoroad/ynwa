# Proposal

## Why

Currently, full matches can only be run interactively in real time via `ynwa-player` (`60` Hz frame loop) or through integration test harnesses designed for short scenario assertions. To develop and compare multiple teams and tactics—such as running a 20-team tournament (~380 matches) via shell scripts—we need a fast, standalone headless CLI tool (`ynwa-simulator`) that simulates a match in model time for a configurable duration, writes recordings to custom directories or descriptive filenames, and embeds final match statistics at the end of the recording.

## Dependencies

This change depends on:
1. **`configurable-match-teams`**: Required so `ynwa-simulator` can load any pair of teams (`--team-a <id>`, `--team-b <id>`) from a teams repository directory via `FootballWorldBuilder::with_teams` and record their IDs and display names.
2. **`record-footer-stats`**: Required so finalized `.jsonl` recordings produced by `ynwa-simulator` include final match score and statistics (`elapsed_time`, `team_stats`, `player_stats`) in their trailing `footer` line.
3. **`client-owned-match-duration`**: Required so match duration is controlled by the client simulation loop (`elapsed_time < duration`) rather than hardcoded inside `ynwa-football`.

## What Changes

- Add a new binary crate `ynwa-simulator` to the Cargo workspace (depending on `ynwa-core`, `ynwa-football`, and `ynwa-repository`, with zero GUI/rendering dependencies).
- Provide a flexible CLI interface for configuring and running a single headless match in model time until `elapsed_time >= duration`:
  - `--teams <dir>` (default `"teams"`) and `--preambles <dir>` (default `"ynwa-scripts/preambles"`)
  - `--team-a <id>` (default `"team_a"`) and `--team-b <id>` (default `"team_b"`)
  - `--out <path>` for specifying either a target output directory (generating a descriptive filename `<team_a>_vs_<team_b>_YYYYMMDD-HHMMSS[_2..].jsonl`) or an explicit `.jsonl` file path
  - `--seed <u64>` (optional deterministic RNG seed) and `--temperature <f32>` (default `0.7`)
  - `--timestep <f32>` (default `1/60` s) and `--duration <f32>` (match duration in seconds, default `120.0` s)
- Stream the match recording via `FileJournalRecorder` and finalize the `.jsonl` footer with match statistics upon completion.

## Capabilities

### New Capabilities
- `simulator-cli`: Standalone headless match simulator CLI (`ynwa-simulator`) for fast model-time match execution, flexible CLI configuration, and descriptive `.jsonl` recording output.

### Modified Capabilities
- `architecture`: Register `ynwa-simulator` in the workspace crate dependency graph alongside `ynwa-player` and `ynwa-integration-testing`.

## Impact

- `Cargo.toml`: Add `ynwa-simulator` to workspace `members`.
- `ynwa-simulator/`: New binary crate (`Cargo.toml`, `src/main.rs`, `src/cli.rs`, `src/runner.rs`, and unit/integration tests).
- `openspec/specs/architecture/spec.md`: Update workspace crate graph requirement.
