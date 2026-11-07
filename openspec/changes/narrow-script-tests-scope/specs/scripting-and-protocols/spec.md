# Spec Delta

## MODIFIED Requirements

### Requirement: Script Testing Infrastructure (`ynwa-script-tests`)

`ynwa-script-tests` SHALL provide deterministic mini-game harnesses (forcing immediate decisions without waiting for `PlayerReactionSystem` timers) that execute the real `core.lua` and `stdlib.lua` against the Lua engine and assert the exact `Decision` and `reason` they produce, including `make_decision()` dispatch routing and the boundaries of pure stdlib query functions (`parse_col`, `parse_notation`, `distance`, `is_in_region`, `is_in_region_obj`). It MUST NOT re-test decision JSON parsing or team coordinate orientation (covered by `ynwa-core` unit tests and `ynwa-integration-testing` scenarios), and MUST NOT contain a `scenarios/` directory of team sets.

#### Scenario: Timer-bypassed decision assertions and visual scenario sets
- **WHEN** `ynwa-script-tests` integration tests run
- **THEN** they evaluate the real `core.lua` and `stdlib.lua` deterministically in a single `DecisionSystem` step without reaction-timer delays, and assert exact `Decision`/`reason` outputs and pure-function boundary behavior
