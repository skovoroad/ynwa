# Spec Delta

## MODIFIED Requirements

### Requirement: FootballWorldBuilder vs `create_football_world` Error Policy

`FootballWorldBuilder::new(repo, preambles_path)` SHALL load `team_a_id` then `team_b_id` (defaulting to `"team_a"` and `"team_b"`, overridable via `.with_teams(team_a_id, team_b_id)`), store `team_a_id`, `team_b_id`, `team_a_display_name`, and `team_b_display_name` in `GameConfig`, flip `Team::B` regions to Display Orientation, load `core.lua` and `stdlib.lua`, default to `PLAYER_RNG_TEMPERATURE = 0.7` (unseeded) and `Setup("kick off")`, and wire the 6-system live football pipeline. `build()` MUST return `Err` on `ScriptedDecisionMaker` failure unless `.with_placeholder_fallback()` is enabled.

#### Scenario: Strict builder failure vs playable fallback
- **WHEN** `team_a/preamble.lua` contains a Lua syntax error
- **THEN** `FootballWorldBuilder::new(repo, preambles).build()` returns `Err`, whereas `FootballWorldBuilder::new(repo, preambles).with_placeholder_fallback().build()` logs a warning to `stderr` and returns `Ok(World)` with `PlaceholderDecisionMaker` (while missing TOML files or invalid set-piece keys still return `Err` in both)

#### Scenario: Custom team identifiers and display name loading
- **WHEN** `FootballWorldBuilder::new(repo, preambles).with_teams("pressing", "counter").build()` is called
- **THEN** `repo.load_team("pressing")` is loaded as `Team::A` and `repo.load_team("counter")` as `Team::B`, `GameConfig` stores both their IDs (`"pressing"`, `"counter"`) and their loaded `display_name` values, and validation errors reference the configured team identifiers
