# Proposal

## Why

The replay `Events:` panel decodes only football `External` journal entries (`decode_football_events`) and silently drops the other seven `JournalEvent` variants, so a recording such as `corner_awarded_*` shows only `Goal line` while hiding `DecisionAssigned`, `DecisionsReset`, `RestartSet`, and `StageChange`. The full composition of a test recording therefore cannot be inspected visually in the player.

## What Changes

- In `ynwa-player` replay mode, the events panel collects and renders every `JournalEvent` in the recording (all 8 variants) instead of only football `External` entries.
- Football `External` entries keep their existing labels; other `External` kinds render generically as `External(<kind>): <data>`.
- The events panel becomes scrollable (mouse wheel and `Up`/`Down` keys) so the full event history can be inspected, with auto-follow pinned to the bottom while not scrolled up.
- No changes to `ynwa-core` or `ynwa-football`; `decode_football_events` remains in place for replay-equivalence tests.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player-client`: the replay `Events:` panel now shows all `JournalEvent` variants with scrolling instead of only the last 5 football events.

## Impact

- `ynwa-player/src/main.rs`, `ynwa-player/src/ui.rs`, `ynwa-player/src/input.rs`.
- `openspec/specs/player-client/spec.md` (delta).
- No dependency or API changes.
