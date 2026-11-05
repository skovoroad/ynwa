# Design

## Context

`runner.rs` steps the world and checks `StopCriterion`s after each step. `criterion.rs` defines `StopCriterion::{OnStage, OnEvent, OnSteps, OnTime}`; `OnEvent` matches football events decoded from `JournalEntry::External`. `compare.rs::entry_matches` already matches `ExpectedEventDto` patterns against `JournalEntry`. See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Stop a run on a matching journal event (`KickOutcome`, `PossessionChange`, `DecisionAssigned`, and the other `JournalEvent` variants).
- Assert the terminating journal event via `[expect.stop]`.

**Non-Goals:**
- No new football events; `when = "event"` keeps covering football events.
- No changes to how `[[expect.journal]]` matches (see `scenario-expectation-matchers`).

## Decisions

1. **Reuse the journal matcher for stop.** The `when = "journal"` criterion reuses the same per-type pattern matching as `[[expect.journal]]` (`entry_matches`), so stop and expectation share one matcher. `at` is omitted from the stop schema because stop timing cannot be predicted.

2. **Evaluate against per-step entries.** The runner already slices `collection.entries()[decoded_upto..]` each step; it will apply journal stop criteria to those same new entries, mirroring how `OnEvent` works for football events.

3. **Supported `type` set.** Support the journal-event types the scenarios require (`kick_outcome`, `possession_change`) and reuse the matcher for the remaining `JournalEvent` variants for uniformity. The runner matches `JournalEvent` directly, not decoded football events.

4. **`StopReason` extension.** Add a `StopReason::Journal(...)` variant carrying the matched pattern, compared in `compare_stop` like the existing `Event` variant.

## Risks / Trade-offs

- [Journal stop criteria resolve player references via `GameConfig`] → compile journal stop criteria with the same `resolve_player_index` used by `compare.rs`, or defer resolution to match time with the config in scope.
- [A journal criterion could be declared but never fire] → the existing safety-criterion requirement (`steps`/`time`) is unchanged, so a run still terminates.
