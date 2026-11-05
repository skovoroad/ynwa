# Proposal

## Why

`ynwa-integration-testing` cannot express several checks required by the target integration scenarios in `context/test_scenarios.md`: the decision target, and the `"none"` markers for a center restart position and an absent decision reason. These gaps block `decision_targets`, `decision_symmetry`, and `goal_scored`.

## What Changes

- Add a `target` field to the `decision_assigned` journal expectation so `Decision::Run(DecisionTarget)` and `Decision::Kick(Point3D)` targets can be asserted (point / cell / region / ball), not just the decision kind.
- Add a `"none"` marker to `restart_set.position` and `final_state.setup.restart_position` so a center restart (`restart_position = None`) can be asserted distinctly from "not checked".
- Add a `"none"` marker to `decision_assigned.reason` so an absent reason (an engine-assigned decision) can be asserted distinctly from "not checked".

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `integration-testing`: extend the `decision_assigned`, `restart_set`, and `setup.restart_position` expectation schemas with decision-target matching and `"none"` markers.

## Impact

- `ynwa-integration-testing/src/dto.rs` — new `DecisionTargetDto`, `ReasonOrNoneDto`, and `PointOrNoneDto` types; extended `ExpectedEventDto::DecisionAssigned`, `ExpectedEventDto::RestartSet`, and `SetupDto`.
- `ynwa-integration-testing/src/loader.rs` — map the `"none"` restart-position marker when applying the initial-state spec.
- `ynwa-integration-testing/src/compare.rs` — target and `"none"`-marker matching in `entry_matches` and `compare_final_setup`.
- No changes to `ynwa-core` or `ynwa-football`.
