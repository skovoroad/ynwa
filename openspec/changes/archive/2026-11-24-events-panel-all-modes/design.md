# Design

## Context

`ynwa-player` owns the right-hand panel. `draw_control_panel` (in `ui.rs`) draws telemetry, then — only when `events: Option<&[JournalEntry]>` is `Some` — an events panel capped at `EVENT_PANEL_SIZE = 5` rows, then a two-column Player Decisions table. `run_play`/`run_record` pass `None`; only `run_replay` builds a filtered `visible_events` list and passes `Some`. `Game` holds a single `Box<dyn JournalSink>`; Play uses `NullJournalSink`, Record uses `FileJournalRecorder`, and `CollectJournalRecorder` (in-memory) already exists but is unused by the player.

See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- Show a scrollable, full-height events list in Play, Record, and Replay.
- Keep the `.jsonl` recording stream intact while also collecting events for live display.

**Non-Goals:**

- Changing the journal wire format (that is `journal-player-identity`).
- Changing event label formatting beyond what `journal-player-identity` does.
- Restoring a per-player "current decision" snapshot.

## Decisions

1. **Tee sink for Record.** Add `TeeJournalSink { first, second }` to `ynwa-core/src/journal.rs`, fanning each `push` to both sinks (cloning the event) and running `finish` on both. Record sets `TeeJournalSink::new(FileJournalRecorder, CollectJournalRecorder)`.
2. **Play collects in memory.** `run_play` sets a `CollectJournalRecorder` over a shared buffer on the world.
3. **Shared in-memory buffer** via `Rc<RefCell<Vec<JournalEntry>>>` (the existing `CollectJournalRecorder` pattern); the render loop reads it each frame. Live events are pushed during `world.step`, so they are naturally ordered and `timestamp <= elapsed`.
4. **Always-present events slice.** `draw_control_panel`/`render_scene` take `events: &[JournalEntry]` and a scroll offset in every mode; Replay keeps its filtered `visible_events` list, Play/Record pass the live buffer.
5. **Dynamic visible rows.** Replace the `EVENT_PANEL_SIZE` constant with a row count computed from the available panel height; `EventPanelScroll` clamps against that count instead of the constant.
6. **Remove the decisions table** and its helpers (`draw_player_decisions_table`, `draw_player_column`, `player_decision_text`, `decision_text`).

## Risks / Trade-offs

- [Play-mode journaling overhead] → Negligible (a small `Vec` push per event); accepted.
- [`TeeJournalSink::finish` must not swallow file errors] → Finish both sinks and combine errors, preserving `FileJournalRecorder`'s first-error semantics.
- [Scroll clamp now depends on window height] → Pass the visible-row count into `EventPanelScroll::update`; no constant drift.
- [Removing the table loses the live per-player decision snapshot] → Accepted; the event log carries the same decision/reason information over time.
