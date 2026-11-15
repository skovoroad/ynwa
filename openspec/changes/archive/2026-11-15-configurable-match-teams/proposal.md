# Proposal

## Why

Currently, `FootballWorldBuilder` and `ynwa-player` hardcode the team identifiers `"team_a"` and `"team_b"` when loading teams from `TeamRepository`, even though `FsTeamRepository::load_team(team_id)` already supports arbitrary team subdirectory names. As we develop and compare multiple teams and tactical setups in `teams/` (or custom team directories), we need to store more than two teams, select any pair of teams by their repository identifier (`team_id`), and clearly distinguish each team's repository identifier (`team_id`) from its human-readable `display_name` in `meta.toml`.

## What Changes

- Read `meta.toml` (`display_name: String`) in `FsTeamRepository::load_team` (defaulting to `team_id` when `meta.toml` is absent) and include `display_name: String` on `TeamRecord` in `ynwa-core::repository`. Update `teams/team_a/meta.toml` and `teams/team_b/meta.toml` to use `display_name`.
- Store both repository identifiers (`team_a_id: String`, `team_b_id: String`) and display names (`team_a_display_name: String`, `team_b_display_name: String`) in `GameConfig` so match configurations, live UI panels, and recorded `.jsonl` headers preserve both the unique team IDs and their human-readable names.
- Extend `FootballWorldBuilder` with `.with_teams(team_a_id, team_b_id)` (defaulting to `"team_a"` and `"team_b"`) so callers can assemble matches between any two team IDs in the repository (including the same team ID playing against itself), and remove the redundant `create_football_world` wrapper function.
- Add `--team-a <id>` and `--team-b <id>` CLI flags to `ynwa-player` (defaulting to `"team_a"` and `"team_b"`, rejected in `--replay` mode) and display `team_a_display_name` and `team_b_display_name` from `GameConfig` in the right-hand control panel.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `football-rules`: `FootballWorldBuilder` supports configurable `team_a_id` and `team_b_id` repository identifiers (defaulting to `"team_a"` and `"team_b"`) and propagates both team IDs and `display_name` values into `GameConfig`.
- `player-client`: `ynwa-player` accepts `--team-a <id>` and `--team-b <id>` in `Play` and `Record` modes and renders `team_a_display_name` / `team_b_display_name` from `GameConfig` in the telemetry panel.
- `scripting-and-protocols`: `FsTeamRepository::load_team(team_id)` reads optional `meta.toml` (`display_name: String`) into `TeamRecord::display_name` (falling back to `team_id` when `meta.toml` is absent).

## Impact

- `ynwa-core`: `TeamRecord` gains `display_name: String`; `GameConfig` gains `team_a_id`, `team_b_id`, `team_a_display_name`, and `team_b_display_name`.
- `ynwa-repository`: `FsTeamRepository` parses `<team_dir>/meta.toml` (`display_name`).
- `teams/`: `teams/team_a/meta.toml` and `teams/team_b/meta.toml` rename `name` to `display_name`.
- `ynwa-football`: `FootballWorldBuilder` gains `.with_teams(team_a_id, team_b_id)` and populates team IDs and display names in `GameConfig`.
- `ynwa-player`: `cli.rs` parses `--team-a` and `--team-b`; `main.rs` passes them to `FootballWorldBuilder`; `ui.rs` displays `team_a_display_name` and `team_b_display_name`.
