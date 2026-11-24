# Proposal

## Why

The two-column Player Decisions table occupies most of the right-hand panel but is not needed. The actually useful telemetry — the journal of events — is shown only in Replay mode and is capped at five rows. A maximally-expanded, scrollable event list is wanted in Play, Record, and Replay alike.

## What Changes

- Remove the two-column Player Decisions table entirely.
- Show a scrollable `Events:` panel in all three modes (`Play`, `Record`, `Replay`), expanded to fill the panel below the score, scrollable through the full history.
- Collect journal events in memory during `Play` and `Record` so the live list has data; in `Record` fan each event out to both the `.jsonl` file and the in-memory buffer.
- Wire the mouse-wheel / `Up` / `Down` scroll controls in all modes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `player-client`: the right-hand panel drops the Player Decisions table and shows a full-height, scrollable events list in every mode.

## Impact

- `ynwa-player` — `main.rs` (live event collection, tee sink wiring), `ui.rs` (panel rewrite), `input.rs` (scroll wiring).
- `ynwa-core` — `journal.rs` + `lib.rs` (new `TeeJournalSink`).
- `player-client` spec — right-hand control panel requirement.
