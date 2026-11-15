# Spec Delta

## MODIFIED Requirements

### Requirement: TeamRepository Contract and Filesystem Team Layout

`FsTeamRepository::new(base)` SHALL implement `TeamRepository::load_team(team_id) -> Result<TeamRecord, String>` by reading `<base>/<team_id>/preamble.lua`, reading optional `<base>/<team_id>/meta.toml` (`display_name: String`, defaulting `TeamRecord::display_name` to `team_id` when `meta.toml` is absent), and scanning subdirectories of `<base>/<team_id>/players/` sorted lexicographically by directory name (requiring `>= 1` player). Each player directory MUST contain `static.toml` and `tactical.toml` and MAY contain `script.lua` (`None` -> empty script inheriting `team_play`).

#### Scenario: Player TOML schemas, tactical squad numbers, and custom play regions
- **WHEN** `FsTeamRepository` loads a team directory
- **THEN** `meta.toml` (when present) provides `display_name: String` (or `TeamRecord::display_name` defaults to `team_id`), `static.toml` provides non-empty `name: String` and `u32` rates (`reaction_rate`, `speed_rate`, `tackle_rate`, `shot_power`, `shot_accuracy` in `10..=100`), and `tactical.toml` provides `number: u32` (`1..=N` contiguous tactical squad index, distinct from future cosmetic shirt numbers), optional `[play_positions]` (keys like `attack`/`defence` consumed only by scripts via `my_regions()`), and `[set_piece_positions]`

