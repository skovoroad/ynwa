# Integration Testing Specification

## Purpose

Defines the deterministic TOML-driven scenario testing framework (`ynwa-integration-testing`), including scenario directory layout, strict unknown-field-rejecting TOML schemas (`initial_state.toml`, `scenario.toml`, optional `final_state.toml`), `[[expect.journal]]` matching rules, script error fail-fast enforcement, numeric tolerances, and replay artifact persistence.

## Requirements

### Requirement: Scenario Directory Layout and `{team, number}` Player Resolution

Every scenario in `ynwa-integration-testing/scenarios/<name>/` SHALL contain `initial_state.toml`, `scenario.toml`, optional `final_state.toml` (omission skips final-state checks), and `teams/` (`team_a/` and `team_b/` in standard on-disk team layout), sharing `<workspace>/ynwa-scripts/preambles/`. All TOML player references MUST use `{ team = "A" | "B", number = u32 }` (`number >= 1`) resolving to a **unique** player on that team.

#### Scenario: Team loading and tactical number uniqueness validation
- **WHEN** a scenario directory is loaded
- **THEN** `"team_a"` and `"team_b"` are loaded from `teams/`, `final_state.toml` is parsed only when present, and referencing `number = 0`, an unknown tactical number, or a duplicate tactical number on a team fails scenario loading with an error

### Requirement: Strict Scenario TOML Schemas and Static Validation

Scenario TOML files (`initial_state.toml`, `scenario.toml`, `final_state.toml`, and all nested tables) SHALL reject unknown fields and enforce static validation: (1) `initial_state.toml` MUST reject `[ball]` when `stage` is omitted or `"Setup"` (since `Setup` pins the ball from `[setup]`), (2) `scenario.toml` `[run]` MUST require finite `timestep > 0.0` and at least one `"steps"` or `"time"` safety criterion in `[[run.stop]]`, and (3) `team` MUST be rejected on `event = "GameEnd"`.

#### Scenario: Supported fields in `initial_state.toml`, `final_state.toml`, and `[run]`
- **WHEN** `initial_state.toml`, `final_state.toml`, and `scenario.toml` `[run]` are parsed
- **THEN** `initial_state.toml` and `final_state.toml` accept optional `stage` (`"Play"|"Setup"|"GameOver"`), `setup_reason`, `ball` (`position`, `velocity`, `possessed_by = {team, number} | "none"`, `last_possessing_team = "A"|"B"|"none"`), `setup` (`restart_position = {x, y, z} | "none"`, `restart_team`), `players`, plus `score = { A = u32, B = u32 }` in `final_state.toml`
- **AND** `players` entries are `[{team, number, position}]` in `initial_state.toml` (overriding only `PlayerState.position` while preserving `tactical.toml` regions/roles and leaving unlisted players at stage defaults) and `[{team, number, position}]` or `[{team, number, region}]` in `final_state.toml`
- **AND** `scenario.toml` `[run]` accepts `timestep: f32` and `[[run.stop]]` tagged by `when`: `"stage"` (`stage`, `setup_reason?`, where `stage = "Setup"` without `setup_reason` matches any `Setup` reason, as in `final_state.toml`), `"event"` (`event` in `"Goal"|"Touchline"|"GoalLine"|"GameEnd"`, `team?`), `"journal"` (`type` plus the same optional matcher fields as the matching `[[expect.journal]]` entry, without `at`), `"steps"` (`steps: u64`), or `"time"` (`time: f32`)

#### Scenario: Region-based final-state player assertions
- **WHEN** `final_state.toml` declares a `players` entry with `{ team, number, region = "A1:B2" }` instead of `position`
- **THEN** the player's final position MUST lie within the grid-notation region on the pitch; an entry declaring both `position` and `region` is rejected at parse time, and a region whose notation cannot be parsed fails the scenario with an error

### Requirement: Journal Expectation Per-Type Schema and Matching Rules

`scenario.toml` `[expect]` SHALL accept `journal_match = "exact" | "subsequence"` (default `"exact"`), optional `stop` (`when = "stage"|"event"|"journal"|"steps"|"time"`, with optional `steps: u64` on `"stage"|"event"|"time"` and required `steps: u64` on `"steps"`), and `[[expect.journal]]` entries tagged by `type` with optional `at: f32` (`|actual - expected| <= timestep.max(1e-4)`). Omitted fields act as wildcards. `decision_assigned.decision` compares the kind (`"Stop"|"Run"|"Kick"`); a present `decision_assigned.target` MUST additionally match the recorded decision's target.

#### Scenario: Per-`type` optional and required fields in `[[expect.journal]]`
- **WHEN** non-empty `[[expect.journal]]` entries are validated and matched against recorded `JournalEntry` items
- **THEN** `decision_assigned` accepts `player?: {team, number}`, `decision?: "Stop"|"Run"|"Kick"`, `target?: { kind = "point", point = {x, y, z} } | { kind = "cell", cell = "M22" } | { kind = "region", region = "A1:B2" } | { kind = "ball" }` (compared against the recorded `Run` target or `Kick` point; `Stop` carries no target), `reason?: String | "none"` (where `"none"` asserts the recorded reason is absent), `at?: f32`; `possession_change` accepts `possessed_by?: {team, number} | "none"`, `last_possessing_team?: "A"|"B"|"none"`, `at?: f32`; `kick_outcome` accepts `player?: {team, number}`, `ball_velocity?: {x, y, z}`, `at?: f32`; and `decisions_reset` accepts `at?: f32`
- **AND** `stage_change` accepts `stage?: "Play"|"Setup"|"GameOver"`, `setup_reason?: String` (bare `setup_reason` without `stage` also pins `Setup(reason)`), `at?: f32`; `restart_set` accepts `position?: {x, y, z} | "none"` (where `"none"` asserts the restart position is absent/center), `team?: "A"|"B"|"none"`, `at?: f32`; `stat_update` accepts `team?: "A"|"B"`, `key?: String`, `delta?: f64`, `at?: f32`; and `football_event` requires `event: "Goal"|"Touchline"|"GoalLine"|"GameEnd"` with optional `team?: "A"|"B"` (forbidden on `"GameEnd"`) and `at?: f32`

#### Scenario: Journal-event stop criteria in `[[run.stop]]` and `[expect.stop]`
- **WHEN** a `[[run.stop]]` entry uses `when = "journal"` with a `type` (e.g. `kick_outcome` or `possession_change`) and optional matcher fields, and the run stops on it
- **THEN** the run stops on the first step whose new journal entries contain a matching `JournalEvent`, and a matching `[expect.stop]` (`when = "journal"`) records the terminating event

### Requirement: Deterministic Execution, Script Error Fail-Fast, Tolerances, and Artifacts

Scenario execution SHALL build the world strictly (without placeholder fallback) at `temperature = 0.0` with a fixed RNG seed, evaluate `run.stop` in declaration order after each step, fail the scenario on any player Lua script error (unlike live play, which logs and continues), and always save `<home>/.ynwa/scenarios/<name>.jsonl` even when assertions fail. `YNWA_SCENARIO=<exact_name>` MUST run only that scenario (or fail listing available names).

#### Scenario: Stop satisfaction, comparison tolerances, accumulated diffs, and replay artifact
- **WHEN** a scenario finishes stepping and compares expectations
- **THEN** `steps` stop triggers at `steps >= limit` and `time` stop triggers at `elapsed_time >= limit`, positions/velocities/stat deltas compare within `1e-4` tolerance, non-empty `expect.journal` requires `1:1` parity in `"exact"` mode or ordered inclusion in `"subsequence"` mode, all mismatches are accumulated into a single readable diff report, and `<home>/.ynwa/scenarios/<name>.jsonl` remains on disk for `./replay.sh` even on failure
