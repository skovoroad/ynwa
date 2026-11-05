# Proposal

## Why

Placement scenarios (`kick_off_placement`, `corner_placement`, `goal_kick_placement`, `throw_in_placement`) check that players reach their tactical regions. `final_state.players` currently accepts only exact `Point3D` positions, which is verbose and brittle for region-shaped expectations.

## What Changes

- Extend `final_state.players` so a player can be asserted to be inside a tactical region (grid notation) instead of an exact position.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `integration-testing`: extend the `final_state.players` expectation with region-based position assertions.

## Impact

- `ynwa-integration-testing/src/dto.rs` — a region/point variant for player placement.
- `ynwa-integration-testing/src/compare.rs` — region containment matching using `ynwa_core::Region::contains_point` and the field `GridDimensions`.
- No changes to `ynwa-core` or `ynwa-football`.
