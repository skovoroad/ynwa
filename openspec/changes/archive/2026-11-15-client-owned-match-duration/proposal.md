# Proposal

## Why

Currently, match duration (`GAME_DURATION = 120.0` s) is hardcoded inside `ynwa-football` (`check_game_end` emitting `FootballEvent::GameEnd`), even though the caller (`ynwa-player`, `ynwa-simulator`, `ynwa-integration-testing`) owns the simulation step loop. This causes contradictions across clients: `check_game_end` only runs during `GameStage::Play` (so a match stalled in `Setup` never ends), `ynwa-player` in `Play` mode ignores `GameOver` and keeps stepping physics after `120.0` s, and clients cannot run matches longer than `120.0` s. Moving match duration control to the client loop makes simulation duration predictable and configurable per client.

## What Changes

- **BREAKING**: Remove `GAME_DURATION`, `check_game_end`, and `FootballEvent::GameEnd` from `ynwa-football::events` and remove the `FootballEvent::GameEnd` transition from `FootballGameManager::handle_event`.
- Update `ynwa-player` to control match duration (`DEFAULT_MATCH_DURATION = 120.0` s, overridable via `--duration <seconds>`) directly in its simulation loop:
  - In `Play` mode, stop advancing simulation steps once `elapsed_time >= duration`.
  - In `Record` mode, finalize the recording once `elapsed_time >= duration` and keep the window open for visual inspection until `Escape`.
- Remove `FootballEvent::GameEnd` formatting from `ynwa-player` (`ui.rs`) and `EventKindDto::GameEnd` / `EventMatcher::GameEnd` from `ynwa-integration-testing`.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `football-rules`: Remove `GAME_DURATION`, `check_game_end`, and `FootballEvent::GameEnd` from `ynwa-football` event detection and `FootballGameManager`.
- `player-client`: `ynwa-player` controls match duration in its client loop (`--duration <seconds>`, default `120.0` s) and removes `FootballEvent::GameEnd` from the replay events panel.
- `integration-testing`: Remove `"GameEnd"` from `football_event` / `when = "event"` TOML schemas and matchers (scenarios already use `when = "time"` for time-based stops).

## Impact

- `ynwa-football`: `src/events.rs`, `src/game_manager.rs`, and tests (`events_tests.rs`, `journal_integration_tests.rs`, `replay_tests.rs`).
- `ynwa-player`: `src/cli.rs`, `src/main.rs`, `src/ui.rs`, and `src/ui_tests.rs`.
- `ynwa-integration-testing`: `src/dto.rs`, `src/criterion.rs`, and tests (`compare_tests.rs`, `criterion_tests.rs`, `dto_tests.rs`, `runner_tests.rs`).

