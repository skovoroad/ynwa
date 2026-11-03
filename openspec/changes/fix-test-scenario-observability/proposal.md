# Proposal

## Why

After the game-serialization tasks, test scenarios can be replayed in the player, but the
replay finishes almost instantly: in five of the six scenarios the stop event fires in ~3
steps (~0.3 s of model time) because the ball starts right next to the line at high speed.
This makes it impossible to visually verify the initial positioning, the ball going out, and
the stop at the event.

## What Changes

Adjust only the `initial_state.toml` of the six integration-test scenarios so the ball starts
farther from the line and/or slower, stretching the phase before the event to ~3–3.5 s. No
other files change.

- `corner`: ball `z` moved `5 → 40`, `vz` slowed `-20 → -15`.
- `goal_kick`: same as `corner` (`z: 5 → 40`, `vz: -20 → -15`).
- `throw_in`: ball `x` moved `34 → 10`, `vx` stays `20`.
- `goal_into_b`: ball `z` moved `99 → 70`, `vz` slowed `20 → 15`.
- `goal_into_a`: ball `z` moved `5 → 35`, `vz` slowed `-20 → -15`.
- `stdlib_kick_to_goal`: ball and player A#1 `z` moved `102 → 82`.

`dt` stays `0.1` in every `scenario.toml`. Stop criteria, `[expect]`, and `final_state.toml`
are unchanged: expectations are not tied to the exact step count or coordinates, so the same
events and final states are still asserted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None.

This change only tunes test fixtures; system behavior does not change, so no spec delta is
needed (`skip_specs: true`).

## Impact

- Only `ynwa-integration-testing/scenarios/*/initial_state.toml` (six files).
- No code, API, dependency, or engine change.
- Integration tests keep asserting the same events (`GoalLine`, `Touchline`, `Goal`) and
  final states; only the duration of the recorded run changes.
