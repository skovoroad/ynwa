# Tasks

## 1. Football goal-line restart classification

- [ ] 1.1 Add a pure goal-line restart classifier in `ynwa-football/src/events.rs` (an enum + function deriving corner vs goal kick from position, last team, and field length) and rewire `FootballGameManager::handle_event` to use it; verify `cargo test -p ynwa-football` passes
- [ ] 1.2 Add unit tests for the classifier covering goal A/B × `last == attacking` / `last != attacking`; verify `cargo test -p ynwa-football` passes

## 2. Player replay events panel

- [ ] 2.1 Extend `ynwa-player`'s `event_text`/`draw_event_panel` to label `GoalLine` as `"Corner; last touch: X"` or `"Goal kick; last touch: Y"` using the classifier and the field length threaded from `draw_control_panel`; verify `cargo build -p ynwa-player` succeeds
- [ ] 2.2 Add a unit test for the label formatter covering goal/throw-in/corner/goal kick/game end; verify `cargo test -p ynwa-player` passes

## 3. Integration verification

- [ ] 3.1 Run `cargo test --workspace`, `cargo clippy --workspace --all-targets`, and `cargo fmt --all -- --check` and confirm no warnings
