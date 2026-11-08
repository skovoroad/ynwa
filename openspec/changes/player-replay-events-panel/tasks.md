# Tasks

## 1. Replay data flow

- [ ] 1.1 In `ynwa-player/src/main.rs` `run_replay`, capture the full journal for display with `let events: Vec<JournalEntry> = record.journal.clone();` before `record` is moved into `create_football_replay_world`, and switch `visible_events` and the `render_scene` parameter from `Vec<(f32, FootballEvent)>` to `Vec<JournalEntry>`; verify `cargo check -p ynwa-player` passes

## 2. Events panel rendering

- [ ] 2.1 In `ynwa-player/src/ui.rs`, replace `event_text(&FootballEvent)` with `event_text(&JournalEvent)` covering all 8 variants — reuse the existing decision formatting for `DecisionAssigned`, decode football `External` via `serde_json::from_value::<FootballEvent>` to keep the current labels, and fall back to `External(<kind>): <data>` for other kinds; verify `cargo check -p ynwa-player` passes
- [ ] 2.2 Add a `#[cfg(test)]` unit-test module in `ui.rs` covering `event_text` for every `JournalEvent` variant plus the football and non-football `External` cases; verify `cargo test -p ynwa-player` passes
- [ ] 2.3 Change `draw_control_panel`/`draw_event_panel` to accept the full `Vec<JournalEntry>` plus a scroll offset and render the clamped visible window (newest at the bottom when offset 0); verify `cargo check -p ynwa-player` passes

## 3. Scroll input

- [ ] 3.1 In `ynwa-player/src/input.rs`, add mouse-wheel and `Up`/`Down` scroll handling that updates the panel scroll offset, clamps it to the available history, and re-pins to the newest entry when scrolled back to 0; verify `cargo check -p ynwa-player` passes

## 4. Integration validation

- [ ] 4.1 Run `cargo clippy --workspace --all-targets` and `cargo fmt --all -- --check` and confirm no warnings; verify `./replay.sh` on a `corner_awarded_*` recording shows every journal event (not just `Goal line`) and that scrolling works
