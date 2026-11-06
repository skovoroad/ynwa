# Proposal

## Why

`StopCriterion::evaluate` returns `Result<Option<StopReason>, ScenarioError>` only because journal stop criteria resolve player references and parse `cell`/`region` targets on every step. Those are static configuration errors, not per-step conditions: the same resolution is repeated inside the run loop, and every criterion evaluation is forced to be fallible even though only one variant can fail.

## What Changes

- Compile `[[run.stop]]` journal criteria once, before the first step, against the loaded `GameConfig`: resolve `{team, number}` references and parse `cell`/`region` targets there.
- Make `StopCriterion::evaluate` return `Option<StopReason>` instead of `Result<Option<StopReason>, ScenarioError>`.
- Replace the shared DTO-based journal matcher with a single compiled journal matcher used by both journal stop criteria and `[[expect.journal]]` comparison, removing per-call resolution.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `integration-testing`: journal stop criteria are compiled and validated against the loaded `GameConfig` before the first step; an unknown/duplicate player or unparsable `cell`/`region` notation fails the scenario before stepping.

## Impact

- `ynwa-integration-testing/src/compare.rs` — compiled journal pattern type, compile function, and an infallible matcher replace `entry_matches`.
- `ynwa-integration-testing/src/criterion.rs` — `JournalMatcher` holds a compiled pattern; `StopCriterion::evaluate` becomes infallible.
- `ynwa-integration-testing/src/runner.rs` — `compile_criteria` takes `&GameConfig` and resolves journal criteria; the step loop no longer propagates a matcher error.
- `ynwa-integration-testing/src/tests/*` — update call sites and add a compile-time failure test for invalid journal stop criteria.
- No changes to the TOML schema or the on-disk scenario format.
