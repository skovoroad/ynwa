# Proposal

## Why

Four public `stdlib.lua` functions (`chase_ball`, `run_to_defence_position`, `run_to_opponent_penalty_area`, `stop`) have no test in `ynwa-script-tests`, and `run_to_attack_position` is exercised only indirectly by a dispatch test that never asserts its target or reason. There is no rule requiring standard-library functions to stay covered.

## What Changes

- Add direct `ynwa-script-tests` tests for `chase_ball()`, `run_to_defence_position()`, `run_to_opponent_penalty_area()`, `stop(reason)`, and `run_to_attack_position()`, asserting the exact `Decision` and `reason` each produces.
- Add a `scripting-and-protocols` requirement that every public `stdlib.lua` function MUST have a co-located test in `ynwa-script-tests`, added and maintained together with any change to that function.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scripting-and-protocols`: add a standard-library test-coverage requirement.

## Impact

- `ynwa-script-tests/tests/stdlib_functions.rs`: new direct tests only; no production code changes.
- `openspec/specs/scripting-and-protocols/spec.md` (via delta): new coverage requirement.
