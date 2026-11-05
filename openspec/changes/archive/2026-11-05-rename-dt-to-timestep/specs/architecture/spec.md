# Spec Delta

## MODIFIED Requirements

### Requirement: Synchronous Poll Loop and Centralized RNG/Journal Ownership

`World` SHALL advance simulation strictly via synchronous caller polling (`World::step(&mut self, timestep: f32)`) without internal threads, timers, or event loops. `Game` MUST own the sole randomness source `Box<dyn RngManager>` and a pluggable `Box<dyn JournalSink>` (defaulting to a zero-overhead no-op sink); simulation systems MUST NOT hold private RNG instances.

#### Scenario: Single-platform deterministic execution
- **WHEN** two `World` instances with identical `GameConfig`, initial `GameStage`, and seeded `RngManager` execute the same `timestep` sequence on the same platform
- **THEN** both instances produce identical `GameState` snapshots and identical `JournalEvent` streams
