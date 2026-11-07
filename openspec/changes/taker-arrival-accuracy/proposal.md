# Proposal

## Why

The placement scenarios (`kick_off_placement_*`, `corner_placement_*`, `goal_kick_placement_*`, `throw_in_placement_*`) must verify that the set-piece taker actually arrives **at** the restart point. Today that is impossible once the taker is allowed to run: the engine marks a running player as arrived within `ARRIVAL_THRESHOLD = 0.5 m`, while the integration framework compares final player positions at `1e-4` tolerance. A running taker therefore always stops 0.3–0.5 m short of the point, so an exact `position` assertion cannot pass.

## What Changes

The exact fix is **not yet decided**; it is one of two directions:

- **(A) Higher arrival precision in the engine.** Make the `on_ball` taker end exactly on the restart point when it arrives (e.g. snap to the restart point on the arrival `Stop`), so the existing exact-position check passes.
- **(B) Lower comparison precision in the integration framework.** Allow a final-state player assertion to accept "within a radius of a point" (e.g. a `within = { point, radius }` placement), so the scenario can assert the taker is within the arrival radius of the restart point.

Whichever direction is chosen, the placement scenarios must be able to assert "the taker is on the restart point" without pre-placing the player.

## Capabilities

Decision pending; exactly one of the following will be modified:

- `football-rules` — if **(A)**: the setup orchestration guarantees the taker is placed exactly on the restart point on arrival.
- `integration-testing` — if **(B)**: final-state player placement gains a tolerance/within-radius form.

No spec delta is written until the decision is made, so this change is marked `skip_specs` for now.

## Impact

- **(A)** `ynwa-football`/`ynwa-core` setup/arrival logic and its tests.
- **(B)** `ynwa-integration-testing` `compare.rs` final-state comparison and DTOs.
- Placement scenario fixtures (`ynwa-integration-testing/scenarios/*_placement_*`), which will assert the taker's arrival.
