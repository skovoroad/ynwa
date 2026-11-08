# Design

## Context

Replay mode builds a reduced world from the recording and re-simulates physics only. For display it calls `decode_football_events(&record.journal)` (`ynwa-football/src/replay.rs`), which returns only `External { kind: "football_event" }` entries as `Vec<(f32, FootballEvent)>`. `run_replay` accumulates these into `visible_events` and passes them through `render_scene` → `draw_control_panel` → `draw_event_panel` → `event_text`, all typed to `FootballEvent` and capped at the last `EVENT_PANEL_SIZE = 5` entries. The remaining seven `JournalEvent` variants are dropped before rendering. See `proposal.md` for motivation.

`JournalEntry`/`JournalEvent` are re-exported from `ynwa-core` and `FootballEvent` from `ynwa-football`, so the player can already reference every type it needs without touching either crate.

## Goals / Non-Goals

**Goals:**
- Show every `JournalEvent` variant in the replay `Events:` panel, in journal order.
- Keep the existing football `External` labels.
- Make the panel scrollable over the full history.

**Non-Goals:**
- No changes to `ynwa-core` or `ynwa-football`; `decode_football_events` stays for replay-equivalence tests.
- No generic multi-sport framework: the player remains football-only; non-football `External` kinds render as a generic fallback.
- No variable-height panel, search, filtering, or persisted scroll state.

## Decisions

1. **Panel data type becomes `Vec<JournalEntry>` instead of `Vec<(f32, FootballEvent)>`.** `JournalEntry` already pairs the timestamp with the full `JournalEvent`, so no new decode step is needed. In `run_replay`, capture `let events = record.journal.clone();` before `record` is moved into `create_football_replay_world(record)`. (Alternative: decode all variants into a bespoke display enum — rejected, it duplicates `JournalEvent` and adds a second code path.)

2. **Generic `event_text(&JournalEvent)` covers all 8 variants.** `DecisionAssigned` reuses the same decision formatting already used by the player decisions table. `External` decodes to `FootballEvent` via `serde_json::from_value` when `kind == "football_event"` and keeps the current labels; any other kind renders `External(<kind>): <data>`. (Alternative: keep `decode_football_events` and merge — rejected, it forces two lists and two ordering paths.)

3. **Scroll as a small panel state owned by `run_replay`.** A `scroll: usize` offset (number of rows scrolled up from the newest entry) is threaded into both `handle_input` (to update it) and `render_scene` (to pick the visible window). `scroll == 0` means pinned to the newest entry with auto-follow; scrolling up increments it, scrolling down back to `0` re-pins. The offset is clamped to `[0, entries.len().saturating_sub(viewport_rows)]`. (Alternative: fixed `EVENT_PANEL_SIZE` with no scroll — rejected, hides earlier events.)

4. **Input: mouse wheel and `Up`/`Down` keys.** `input.rs` currently handles only `Escape`, `F11`, `Space`, and `+`/`-`; none conflict. The wheel delta is polled per frame and `KeyCode::Up`/`KeyCode::Down` via `is_key_pressed`.

5. **Fixed viewport height is preserved.** The events panel keeps a fixed number of visible rows so the `Player Decisions` table below keeps its layout; scrolling, not resizing, reveals the full history.

## Risks / Trade-offs

- [Cloning `record.journal` duplicates `serde_json::Value` `External` payloads] → acceptable for a debug replay; the alternative (borrowing) is impossible because `create_football_replay_world` consumes the record.
- [No automated UI test exists for the panel] → validation is `cargo check`/`clippy`/`fmt` plus a manual replay of a recording such as `corner_awarded_*`; the spec scenarios describe the expected behavior.
- [Many events share one timestamp (e.g., the `corner_awarded_*` burst)] → journal order is stable within a timestamp, so the panel still shows a deterministic sequence.
