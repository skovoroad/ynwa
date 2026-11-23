# Proposal

## Why

The `+`/`-` keys currently change the simulation's temporal resolution (`rate`/`timestep`) rather than the playback speed, so they have no user-facing value and silently alter the match. Repurposing them to control playback speed (a wall-clock time-scale) makes them genuinely useful and is safe because the simulation `timestep` stays fixed.

## What Changes

- `+`/`=`/`KP_ADD` and `-`/`KP_SUBTRACT` now double/halve the playback `speed` (clamped to `0.25x`–`8x`) instead of changing the simulation resolution.
- The speed control applies in all three modes (`Play`, `Record`, `Replay`); changing `speed` never changes the simulation `timestep`, so recording and replay stay byte-identical to an unaccelerated run.
- Manual `speed` applies during the `Play` stage; during `Setup` the automatic `4x` from `speed-up-setup-stage` takes precedence.
- Remove the now-unused `rate`/`increase_rate`/`decrease_rate` machinery.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player-client`: the "Simulation Pacing and Keyboard Controls" requirement changes from resolution control to playback-speed control.

## Impact

- `ynwa-player/src/simulation.rs`: `speed` becomes the user-adjustable playback multiplier; `rate` machinery removed.
- `ynwa-player/src/input.rs`: `+`/`-` bind to speed instead of rate; the `rate_locked` guard is removed.
- `ynwa-player/src/main.rs`: `advance` applies the Setup override vs. the manual speed.
- Depends on `speed-up-setup-stage` (introduces the `speed` concept and Setup auto-`4x`).
- No changes to `ynwa-core`, `ynwa-football`, the recording format, or replay semantics.
