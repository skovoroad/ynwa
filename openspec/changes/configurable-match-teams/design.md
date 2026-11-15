# Design

## Context

See `proposal.md` for motivation. Currently:
- `ynwa_core::repository::TeamRecord` holds `players` and `preamble`, while `teams/<id>/meta.toml` is ignored by `FsTeamRepository`.
- `FootballWorldBuilder::create_game` calls `repo.load_team("team_a")` and `repo.load_team("team_b")` directly.
- `GameConfig` holds `field`, `players`, `ball`, `referees`, and `scripting`, with neither team repository IDs nor display names.
- `ynwa-player` accepts positional `teams_path` and `preambles_path`, and hardcodes `"A"` / `"B"` and `"Team A"` / `"Team B"` labels in `ui.rs`.

## Goals / Non-Goals

**Goals:**
- Clearly separate a team's unique repository identifier (`team_id`, e.g. `"pressing_v1"`) from its human-readable UI label (`display_name`, e.g. `"Team A"`).
- Allow selecting any two `team_id` values from a `TeamRepository` via `FootballWorldBuilder::with_teams(team_a_id, team_b_id)` and `ynwa-player` CLI flags (`--team-a`, `--team-b`).
- Persist both `team_a_id` / `team_b_id` and `team_a_display_name` / `team_b_display_name` in `GameConfig` so `.jsonl` recordings retain the exact repository identifiers alongside UI display names.

**Non-Goals:**
- Splitting roster (`static.toml`) and tactical formations (`tactical.toml`) into separate storage trees (deferred to a dedicated future change; separating `team_id` and `display_name` now keeps the model clean for that split).
- Changing how `Team::A` / `Team::B` enum variants work internally in `ynwa-core`.

## Decisions

1. **Separate `team_id` and `display_name` across `meta.toml`, `TeamRecord`, and `GameConfig`**:
   - In `meta.toml`, use `display_name = "..."` (`struct MetaDto { display_name: String }`) instead of `name` so it is never confused with `team_id` or `PlayerStatic::name`.
   - `FsTeamRepository::load_team(team_id)` reads `<team_dir>/meta.toml` when present and populates `TeamRecord::display_name`; if `meta.toml` is absent (e.g., in minimal test fixtures), `display_name` defaults to `team_id.to_string()`.
   - `GameConfig` stores `pub team_a_id: String`, `pub team_b_id: String`, `pub team_a_display_name: String`, and `pub team_b_display_name: String`.

2. **Builder API on `FootballWorldBuilder` and removal of `create_football_world`**:
   - Store `team_a_id: &'a str` and `team_b_id: &'a str` on `FootballWorldBuilder<'a>`, initialized to `"team_a"` and `"team_b"`.
   - Add `pub fn with_teams(mut self, team_a_id: &'a str, team_b_id: &'a str) -> Self`.
   - Remove the redundant `create_football_world(repo, preambles_path)` wrapper; `ynwa-player` (and future callers like `ynwa-simulator`) use `FootballWorldBuilder` directly.

3. **CLI flags in `ynwa-player`**:
   - Add `--team-a <id>` and `--team-b <id>` flags to `ynwa-player`.
   - Reject duplicate `--team-a` or `--team-b` flags and reject combining `--team-a` / `--team-b` with `--replay` (since replay loads teams from the recording header).

## Risks / Trade-offs

- **[Risk] Copied team folders might retain an old `display_name` in `meta.toml`** → Mitigation: Because `GameConfig` stores both `team_a_id` / `team_b_id` and `team_a_display_name` / `team_b_display_name`, the unique repository identifiers are never lost in recordings even if two teams share a `display_name`.
