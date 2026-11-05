# Spec Delta

## MODIFIED Requirements

### Requirement: Strict Scenario TOML Schemas and Static Validation

Scenario TOML files (`initial_state.toml`, `scenario.toml`, `final_state.toml`, and all nested tables) SHALL reject unknown fields and enforce static validation: (1) `initial_state.toml` MUST reject `[ball]` when `stage` is omitted or `"Setup"` (since `Setup` pins the ball from `[setup]`), (2) `scenario.toml` `[run]` MUST require finite `timestep > 0.0` and at least one `"steps"` or `"time"` safety criterion in `[[run.stop]]`, and (3) `team` MUST be rejected on `event = "GameEnd"`.

#### Scenario: Supported fields in `initial_state.toml`, `final_state.toml`, and `[run]`
- **WHEN** `initial_state.toml`, `final_state.toml`, and `scenario.toml` `[run]` are parsed
- **THEN** `initial_state.toml` and `final_state.toml` accept optional `stage` (`"Play"|"Setup"|"GameOver"`), `setup_reason`, `ball` (`position`, `velocity`, `possessed_by = {team, number} | "none"`, `last_possessing_team = "A"|"B"|"none"`), `setup` (`restart_position`, `restart_team`), `players` (`[{team, number, position}]`, overriding only `PlayerState.position` while preserving `tactical.toml` regions/roles and leaving unlisted players at stage defaults), plus `score = { A = u32, B = u32 }` in `final_state.toml`
- **AND** `scenario.toml` `[run]` accepts `timestep: f32` and `[[run.stop]]` tagged by `when`: `"stage"` (`stage`, `setup_reason?`, where `stage = "Setup"` without `setup_reason` matches any `Setup` reason, as in `final_state.toml`), `"event"` (`event` in `"Goal"|"Touchline"|"GoalLine"|"GameEnd"`, `team?`), `"steps"` (`steps: u64`), or `"time"` (`time: f32`)

#### Scenario: Region-based final-state player assertions
- **WHEN** `final_state.toml` declares a `players` entry with `{ team, number, region = "A1:B2" }` instead of `position`
- **THEN** the player's final position MUST lie within the grid-notation region on the pitch, and an entry declaring both `position` and `region` (or an invalid region notation) is rejected
