# Proposal

## Why

The replay `Events:` panel in `ynwa-player` is built from `FootballEvent`s, and `FootballEvent::GoalLine` is rendered as `"Goal line; last touch: X"` regardless of whether the award is a corner or a goal kick. The award is already decided by the engine but is invisible in the player, so a recorded corner shows no "Corner" event.

## What Changes

- Add a pure, tested classifier in `ynwa-football` that derives the awarded restart (corner vs goal kick) from a `GoalLine` event's crossing position and last-touch team, and reuse it in `FootballGameManager::handle_event`.
- In `ynwa-player`, label `GoalLine` events in the replay `Events:` panel as `"Corner; last touch: X"` or `"Goal kick; last touch: Y"` using that classifier and the field length.

## Capabilities

### Modified Capabilities

- `player-client`: the replay events panel distinguishes corner from goal kick for `GoalLine` events.

## Impact

- `ynwa-football/src/events.rs` (classifier) and `ynwa-football/src/game_manager.rs` (reuse in `handle_event`).
- `ynwa-player/src/ui.rs` (label formatting, field length threaded through).
- `openspec/specs/player-client/spec.md` (delta).
- No `ynwa-core` change; the `FootballEvent` model is unchanged.
