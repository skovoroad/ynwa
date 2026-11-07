# Proposal

## Why

Awarded and goal scenarios stop on the football event (`GoalLine`/`Touchline`/`Goal`), which is the same step in which the stage transitions to `Setup`. The recording therefore ends before the first `Setup` tick, so the replay never shows the ball being placed at the restart point. Stopping on a `time` threshold instead would be fragile because it couples the scenario to the exact physics of the crossing.

## What Changes

- In `ynwa-integration-testing`'s runner, when a run stops on a `when = "event"` criterion, advance exactly one additional `World::step` at the scenario `timestep` without evaluating stop criteria, and preserve the triggering event as the stop reason.
- The trailing step's journal entries, step count, and final state become part of the run outcome.
- No change to scenario files or to `stage`/`journal`/`steps`/`time` stop criteria.

## Capabilities

### Modified Capabilities

- `integration-testing`: a run that stops on an event criterion now performs one trailing simulation step before finishing.

## Impact

- `ynwa-integration-testing/src/runner.rs` and its unit tests.
- `openspec/specs/integration-testing/spec.md` (delta).
- Existing event-stopped scenarios keep passing (subsequence journals, no `steps` assertion on event stops); `tests/replay.rs` (`stdlib_kick_to_goal`, a goal event) begins exercising the trailing step.
