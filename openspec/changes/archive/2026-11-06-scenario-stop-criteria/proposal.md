# Proposal

## Why

The kick and pass scenarios (`kick_off_kick`, `corner_kick`, `goal_kick_kick`, `throw_in_kick`, `kick_off_after_goal_kick`, `pass_received`, `pass_intercepted`) need to stop on `KickOutcome` or `PossessionChange`. Those are journal events, not football events, so the current `when = "event"` stop criterion cannot target them; scenarios would fall back to a loose `steps` limit.

## What Changes

- Add a `when = "journal"` stop criterion (`[[run.stop]]`) that stops when a journal entry matches a per-type pattern (the same matching used by `[[expect.journal]]`).
- Add a matching `when = "journal"` variant to `[expect.stop]` so the terminating journal event can be asserted.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `integration-testing`: extend the `[[run.stop]]` and `[expect.stop]` schemas with journal-event stop criteria.

## Impact

- `ynwa-integration-testing/src/dto.rs` — `StopCriterionDto` and `StopExpectationDto` gain a `journal` variant.
- `ynwa-integration-testing/src/criterion.rs` — a journal-event stop matcher and `StopReason` variant.
- `ynwa-integration-testing/src/runner.rs` — evaluate journal stop criteria against per-step journal entries.
- `ynwa-integration-testing/src/compare.rs` — compare `[expect.stop]` journal reasons.
- No changes to `ynwa-core` or `ynwa-football`.
