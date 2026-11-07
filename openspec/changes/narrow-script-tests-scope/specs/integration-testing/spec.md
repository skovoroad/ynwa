# Spec Delta

## ADDED Requirements

### Requirement: Scenario Team Sets as Canonical Visual Review Data

`ynwa-integration-testing/scenarios/<name>/teams/` SHALL be the canonical on-disk team sets for visual review. Each scenario's `teams/` tree (standard `team_a`/`team_b` layout) MUST load directly into `ynwa-player` via its `--teams-path` and `--preambles-path` options, replacing the former `ynwa-script-tests/scenarios/` team sets.

#### Scenario: Visual review of a scenario's team sets
- **WHEN** a developer wants to visually inspect a scenario
- **THEN** they point `ynwa-player` at the scenario's `teams/` directory and the shared `<workspace>/ynwa-scripts/preambles/`, and the match loads for visual review
