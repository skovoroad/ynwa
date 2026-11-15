# Tasks

## 1. Team Identifiers and Display Names in Core and Repository

- [ ] 1.1 Add `pub display_name: String` to `TeamRecord` in `ynwa-core/src/repository.rs` and `pub team_a_id: String`, `pub team_b_id: String`, `pub team_a_display_name: String`, and `pub team_b_display_name: String` to `GameConfig` in `ynwa-core/src/game.rs`, and verify `cargo test -p ynwa-core` passes.
- [ ] 1.2 Update `teams/team_a/meta.toml` and `teams/team_b/meta.toml` to use `display_name = "..."`, parse `<team_dir>/meta.toml` (`display_name: String`, falling back to `team_id` when `meta.toml` is absent) in `FsTeamRepository::load_team` (`ynwa-repository/src/fs_team_repository.rs`), and add unit tests in `ynwa-repository/src/player_tests.rs` verifying both explicit `meta.toml` and missing `meta.toml` fallback via `cargo test -p ynwa-repository`.

## 2. Configurable Team IDs in `ynwa-football`

- [ ] 2.1 Add `team_a_id` and `team_b_id` fields (defaulting to `"team_a"` and `"team_b"`) and `.with_teams(team_a_id, team_b_id)` to `FootballWorldBuilder` in `ynwa-football/src/lib.rs`, wire `team_a_id`, `team_b_id`, `team_a_record.display_name`, and `team_b_record.display_name` into `GameConfig`, remove `create_football_world`, and verify `cargo test -p ynwa-football` passes.
- [ ] 2.2 Add unit tests in `ynwa-football/src/tests/world_builder_tests.rs` covering custom team IDs (including playing a team ID against itself, populating `GameConfig` team IDs and display names, and error messages referencing custom team IDs) and verify with `cargo test -p ynwa-football`.

## 3. Team Selection CLI Flags and UI Display Names in `ynwa-player`

- [ ] 3.1 Extend `Cli` and `parse` in `ynwa-player/src/cli.rs` to support `--team-a <id>` and `--team-b <id>` (defaulting to `"team_a"` and `"team_b"`, rejecting duplicates or combination with `--replay`), wire them into `FootballWorldBuilder` in `ynwa-player/src/main.rs`, and add CLI unit tests verified by `cargo test -p ynwa-player`.
- [ ] 3.2 Update `ynwa-player/src/ui.rs` (and `ynwa-player/src/ui_tests.rs`) to display `config.team_a_display_name` and `config.team_b_display_name` in the score line and player decisions table headers, and verify all workspace tests pass with `cargo test` and `cargo clippy`.
