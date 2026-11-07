# Proposal

## Why

`ynwa-script-tests` has drifted from its original purpose. It now mixes its unique value — executing the real `core.lua` and `stdlib.lua` against the real Lua engine with exact `Decision`/`reason` assertions — with coverage already owned by `ynwa-core` unit tests and `ynwa-integration-testing` scenarios, plus a dead `scenarios/` directory. Narrowing it to its unique niche removes redundant maintenance while preserving the only layer that verifies the authored Lua preambles end-to-end.

## What Changes

- Narrow `ynwa-script-tests` to **Lua preamble contract tests**: execute the real `core.lua` and `stdlib.lua`, assert exact `Decision` and `reason`, assert the boundaries of pure stdlib query functions (`parse_col`, `parse_notation`, `distance`, `is_in_region`, `is_in_region_obj`), and verify `make_decision()` dispatch routing.
- Remove coverage from `ynwa-script-tests` that duplicates other layers:
  - JSON -> `Decision` parsing tests (stop/cell/region/point/kick/run-to-ball) — already owned by `ynwa-core` `decision_parser` unit tests and `decision_targets_*` scenarios.
  - Team coordinate orientation tests (`team_orientation.rs`) — already owned by `ynwa-core` `scripted_decision_maker` orientation tests and `decision_symmetry_*` scenarios.
  - Context-structure assertions — already owned by the `scripting-and-protocols` context-schema requirement and `scripted_decision_maker` context tests.
- Delete the dead `ynwa-script-tests/scenarios/` directory (no test references it).
- Officially assign the "visual review in `ynwa-player`" role to `ynwa-integration-testing/scenarios/<name>/teams/`, which already use the standard on-disk team layout.
- Renaming `ynwa-script-tests` was considered and is **out of scope**: this change keeps the existing crate and folder layout unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scripting-and-protocols`: narrow the `Script Testing Infrastructure (ynwa-script-tests)` requirement to Lua preamble contract testing; drop the visual scenario-set responsibility and the team-symmetry/parsing re-testing responsibilities.
- `integration-testing`: designate `ynwa-integration-testing/scenarios/<name>/teams/` as the canonical on-disk team sets for visual review in `ynwa-player`.

## Impact

- `ynwa-script-tests`: removes redundant/duplicate tests and the `scenarios/` directory; keeps the Lua preamble contract tests. No crate or dependency-graph changes.
- `ynwa-integration-testing`: gains explicit ownership of the visual-review scenario sets (the data already exists).
- No changes to `ynwa-core`, `ynwa-football`, `ynwa-decisions`, `ynwa-player`, or the workspace dependency graph.
- Specs updated: `scripting-and-protocols`, `integration-testing`.
