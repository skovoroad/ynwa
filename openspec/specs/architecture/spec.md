# Architecture Specification

## Purpose

Defines the Cargo workspace crate topology, strict acyclic dependency boundaries, hybrid parallel-array entity model, immutable-config versus mutable-state separation, synchronous poll simulation lifecycle, centralized randomness/journaling ownership, and SI base-unit serialization contracts for the YNWA simulation engine.

## Requirements

### Requirement: Acyclic Workspace Crate Dependency Graph

The Rust 2021 Cargo workspace SHALL enforce a strictly acyclic crate graph:
- `ynwa-decisions` -> zero YNWA crates.
- `ynwa-core` -> `ynwa-decisions`.
- `ynwa-football` -> `ynwa-core` (`ynwa-repository` allowed in `[dev-dependencies]` only).
- `ynwa-repository` -> `ynwa-core`.
- `ynwa-player` and `ynwa-integration-testing` -> `ynwa-core`, `ynwa-football`, `ynwa-repository`.
- `ynwa-script-tests` -> `ynwa-core`, `ynwa-football`; Lua assets live in `ynwa-scripts/` and `teams/`.

#### Scenario: Prohibiting reverse dependencies across layers
- **WHEN** workspace dependencies are resolved by Cargo
- **THEN** lower-level crates (`ynwa-decisions`, `ynwa-core`) never depend on sport crates (`ynwa-football`), storage crates (`ynwa-repository`), or client/test crates (`ynwa-player`, `ynwa-integration-testing`)

### Requirement: Sport, Storage, and UI Isolation of Core Engine

`ynwa-core` SHALL remain strictly sport-agnostic, storage-agnostic, and UI-agnostic. It MUST NOT contain football-specific concepts (goals, corners, throw-ins, match duration), filesystem team loaders, or rendering dependencies. Sport rules MUST reside in `ynwa-football`, filesystem persistence in `ynwa-repository`, and desktop visualization in `ynwa-player`.

#### Scenario: Adding or modifying sport-specific rules
- **WHEN** a football rule, pitch zone, or set-piece transition is added or changed
- **THEN** all changes are confined to `ynwa-football` and `ynwa-core` compiles unchanged without sport-specific symbols

#### Scenario: Loading teams via repository abstraction
- **WHEN** `ynwa-football` assembles a match `World`
- **THEN** team definitions are fetched exclusively through the `ynwa_core::repository::TeamRepository` trait without coupling to filesystem storage

### Requirement: JSON-Only Boundary Around Decision Engine

`ynwa-decisions` SHALL be a standalone, game-agnostic Lua execution crate with zero dependencies on `ynwa-core` or `ynwa-football`. All configuration (`static_data`, preambles, scripts), per-tick state (`context`), and returned decisions MUST cross the `ynwa-core` <-> `ynwa-decisions` boundary exclusively as `serde_json::Value`.

#### Scenario: Invoking Lua decisions from core
- **WHEN** `ynwa-core` initializes or queries `ynwa-decisions`
- **THEN** domain types are serialized to and parsed from `serde_json::Value` inside `ynwa-core` without exposing domain structs to `ynwa-decisions`

### Requirement: Immutable GameConfig vs Mutable GameState Separation

`Game` SHALL separate match-invariant configuration (`GameConfig`, exposed read-only via `Game::config()`) from tick-mutable runtime state (`GameState`). `GameConfig` MUST NOT be mutated after `Game` creation, and both `GameConfig` and `GameState` MUST support deterministic `serde` serialization and equality comparison.

#### Scenario: Partition of invariant configuration and mutable match state
- **WHEN** systems inspect or update a `Game` during a simulation step
- **THEN** `game.config()` provides read-only pitch geometry, static entity definitions, and Lua preambles, while `game.state` holds mutable elapsed time, stage, entity kinematics/decisions, per-team and per-player `StatSet` counters (defaulting missing keys to `0.0`), and set-piece restart state

### Requirement: Parallel-Array Entity Indexing and Atomic State Replacement

Entity definitions and runtime states SHALL use `1:1` parallel `Vec` arrays indexed in `O(1)` by global `player_index` (`Team::A` at `0..N_a`, `Team::B` at `N_a..N_a + N_b`) without external ECS crates or a shared entity trait (`RefereeDef`/`RefereeState` are currently unsimulated structural stubs). Full state replacement via `Game::apply_state` MUST validate structural consistency against `GameConfig` atomically.

#### Scenario: Atomic rejection of inconsistent state snapshot
- **WHEN** `Game::apply_state(state)` receives a snapshot incompatible with `GameConfig` (mismatched parallel-array lengths, out-of-bounds ball possessor index, or stats referencing a team absent from config)
- **THEN** it returns `Err` and leaves the existing `GameState` completely unmodified

### Requirement: Synchronous Poll Loop and Centralized RNG/Journal Ownership

`World` SHALL advance simulation strictly via synchronous caller polling (`World::step(&mut self, timestep: f32)`) without internal threads, timers, or event loops. `Game` MUST own the sole randomness source `Box<dyn RngManager>` and a pluggable `Box<dyn JournalSink>` (defaulting to a zero-overhead no-op sink); simulation systems MUST NOT hold private RNG instances.

#### Scenario: Single-platform deterministic execution
- **WHEN** two `World` instances with identical `GameConfig`, initial `GameStage`, and seeded `RngManager` execute the same `timestep` sequence on the same platform
- **THEN** both instances produce identical `GameState` snapshots and identical `JournalEvent` streams

### Requirement: SI Base-Unit Dimensional Quantities

Spatial and kinematic quantities in `ynwa-core` and `ynwa-football` SHALL use typed `uom` physical quantities (`Length`, `Velocity`, `Angle`) serialized in SI base units (`meter`, `meter_per_second`, `radian`), converting to raw `f32` only at JSON, TOML, or screen-rendering boundaries.

#### Scenario: Base SI unit serialization round-trip
- **WHEN** `Point3D`, `Velocity3D`, `Length`, or `Angle` values inside `GameConfig`, `GameState`, or `JournalEvent` are serialized and deserialized via `serde`
- **THEN** their numeric magnitudes represent `meter`, `meter_per_second`, and `radian` without unit-conversion drift
