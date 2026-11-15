# Spec Delta

## MODIFIED Requirements

### Requirement: Acyclic Workspace Crate Dependency Graph

The Rust 2021 Cargo workspace SHALL enforce a strictly acyclic crate graph:
- `ynwa-decisions` -> zero YNWA crates.
- `ynwa-core` -> `ynwa-decisions`.
- `ynwa-football` -> `ynwa-core` (`ynwa-repository` allowed in `[dev-dependencies]` only).
- `ynwa-repository` -> `ynwa-core`.
- `ynwa-player`, `ynwa-simulator`, and `ynwa-integration-testing` -> `ynwa-core`, `ynwa-football`, `ynwa-repository`.
- `ynwa-script-tests` -> `ynwa-core`, `ynwa-football`; Lua assets live in `ynwa-scripts/` and `teams/`.

#### Scenario: Prohibiting reverse dependencies across layers
- **WHEN** workspace dependencies are resolved by Cargo
- **THEN** lower-level crates (`ynwa-decisions`, `ynwa-core`) never depend on sport crates (`ynwa-football`), storage crates (`ynwa-repository`), or client/test crates (`ynwa-player`, `ynwa-simulator`, `ynwa-integration-testing`)

