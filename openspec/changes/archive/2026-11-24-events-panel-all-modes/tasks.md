# Tasks

## 1. Add the tee journal sink

- [x] 1.1 Add `TeeJournalSink` to `ynwa-core/src/journal.rs` holding two `Box<dyn JournalSink>`; `push` forwards the event to both (cloning), and `finish` runs both sinks and combines errors; verify `cargo build -p ynwa-core`
- [x] 1.2 Re-export `TeeJournalSink` from `ynwa-core/src/lib.rs` in the `pub use journal::{...}` list; verify `cargo build -p ynwa-core`
- [x] 1.3 Add a unit test asserting a `TeeJournalSink` delivers each event to both inner sinks and runs both `finish` calls; verify `cargo test -p ynwa-core journal`

## 2. Collect events live in Play and Record

- [x] 2.1 In `run_play` (`ynwa-player/src/main.rs`), create a `CollectJournalRecorder` over a shared `Rc<RefCell<Vec<JournalEntry>>>` and call `set_journal_sink` on the world; verify `cargo build -p ynwa-player`
- [x] 2.2 In `run_record`, replace the direct `FileJournalRecorder` with `TeeJournalSink::new(FileJournalRecorder, CollectJournalRecorder)`; verify `cargo build -p ynwa-player` and that a recording still produces a valid `.jsonl` footer

## 3. Wire the events list and scroll in all modes

- [x] 3.1 Change `draw_control_panel` and `render_scene` to accept `events: &[JournalEntry]` plus a scroll offset in every mode, and pass the live buffer in `run_play`/`run_record`; verify `cargo build -p ynwa-player`
- [x] 3.2 Change `handle_input` and `EventPanelScroll::update` to take a visible-row count and clamp against it instead of `EVENT_PANEL_SIZE`; verify `cargo test -p ynwa-player`

## 4. Replace the decisions table with an expanded events panel

- [x] 4.1 Remove `draw_player_decisions_table`, `draw_player_column`, `player_decision_text`, and `decision_text` from `ynwa-player/src/ui.rs`; verify `cargo build -p ynwa-player`
- [x] 4.2 Rewrite `draw_event_panel` to compute the visible row count from the available panel height and fill the panel below the score; verify `cargo test -p ynwa-player`

## 5. Verify the workspace

- [x] 5.1 Run `cargo test --workspace`, `cargo clippy --workspace`, and `cargo fmt --check --all`; verify all tests pass and there are no warnings
