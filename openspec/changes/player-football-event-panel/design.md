# Design

## Context

The replay `Events:` panel is built in `ynwa-player/src/main.rs::run_replay` from `decode_football_events(&record.journal)` — a `Vec<(f32, FootballEvent)>` — and drawn by `draw_control_panel` → `draw_event_panel` → `event_text` (`ynwa-player/src/ui.rs`). `event_text` maps `FootballEvent::GoalLine(_, last)` uniformly to `"Goal line; last touch: X"`, so a corner and a goal kick are indistinguishable. The award decision already exists inside `FootballGameManager::handle_event` (`ynwa-football/src/game_manager.rs`): the attacking team is `B` when the ball crossed at `z < L/2` else `A`, a goal kick is awarded when `last == attacking`, and a corner otherwise.

## Goals / Non-Goals

**Goals:**
- Reuse the single football award rule so the player and the engine never disagree.
- Label `GoalLine` events in the replay panel as "Corner" or "Goal kick".

**Non-Goals:**
- Kick-off after a goal and other journal-only restarts (`RestartSet` + `StageChange`) are not surfaced; the panel contract remains "last 5 `FootballEvent`s".
- No change to the event model or to `ynwa-core`.

## Decisions

1. **Expose a pure classifier in `ynwa-football` and reuse it in `handle_event`.** A `pub` function (with a small enum result) computes corner-vs-goal-kick from `(position, last_team, field_length)`, so `FootballGameManager` and the player share one source of truth. (Alternative: duplicate the three-line rule in the player — rejected, it would drift from the engine.)
2. **Label format.** Keep the existing "last touch" text: `GoalLine` renders `"Corner; last touch: <last>"` or `"Goal kick; last touch: <last>"`; `Goal`, `Throw-in`, and `GameEnd` are unchanged.
3. **Thread field length into the formatter.** `draw_control_panel` already receives `GameConfig`; pass `field.length()` down through `draw_event_panel` to `event_text`. No new data source is needed.
4. **Placement of the classifier.** Put it next to `FootballEvent` in `ynwa-football/src/events.rs` (pure, event-level, importable by the player via the existing `ynwa-football` dependency).

## Risks / Trade-offs

- [Player and engine could still diverge if `handle_event` is not actually refactored to use the classifier] → the implementation task explicitly rewires `handle_event` to call it, and `ynwa-football` tests cover the four cases.
- [Adding a `pub` API to `ynwa-football` grows its surface] → small, pure, and justified by cross-crate reuse; no `ynwa-core` change.
