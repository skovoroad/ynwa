# Proposal

## Why

`ynwa-integration-testing/scenarios/` currently holds only 6 preliminary, flat scenarios that do not cover the agreed target set in `context/test_scenarios.md`. The framework (stop criteria, journal expectations, decision-target checks, final-state regions) is now capable of expressing the full set. We need to author the target scenarios, organized so they are easy to navigate and can be reviewed both by `cargo test` and visually in the player.

## What Changes

- Add the target integration scenarios from `context/test_scenarios.md` as flat-named directories under `ynwa-integration-testing/scenarios/`.
- Use a `<group>_<case>_<combination>` naming convention so the existing alphabetical discovery reproduces the document's grouping without changing the framework; the `<combination>` suffix uses physical field naming (goal line, side line, half, last-touch team) rather than team-relative `own/opp`/`left/right`.
- Author each scenario for visual replay: a small `run.timestep` and several seconds of model time, with `*_kick_*` and `decision_*` scenarios stopping on a `time` criterion rather than their first emitted event so a recorded run replays smoothly in the player.
- Keep the existing 6 legacy scenarios in place for now; their retirement is handled separately and manually.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This change authors scenario fixtures only; no spec-level behavior changes (`skip_specs: true`).

## Impact

- `ynwa-integration-testing/scenarios/` — new scenario directories (data only: `initial_state.toml`, `final_state.toml`, `scenario.toml`, `teams/`).
- No framework or core code changes.
