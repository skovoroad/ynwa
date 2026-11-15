# Design

## Context

See `proposal.md` for motivation. Currently:
- `ynwa-football/src/events.rs` defines `pub const GAME_DURATION: f32 = 120.0`, `FootballEvent::GameEnd`, and `check_game_end(game)`.
- `FootballGameManager` evaluates `check_events(game)` only during `GameStage::Play`, transitioning `game.state.stage` to `GameStage::GameOver` upon `FootballEvent::GameEnd`.
- `ynwa-player` in `Record` mode exits when `stage == GameStage::GameOver`, while `Play` mode ignores `GameStage::GameOver` and continues stepping physics indefinitely.

## Goals / Non-Goals

**Goals:**
- Remove `GAME_DURATION`, `check_game_end`, and `FootballEvent::GameEnd` from `ynwa-football` so the football rules layer simulates an unbounded stream of play and set-piece restarts as long as the caller polls `World::step`.
- Make the loop owner (`ynwa-player`, `ynwa-simulator`, `ynwa-integration-testing`) solely responsible for deciding how long a simulation runs.
- Support `--duration <seconds>` (default `120.0` s) in `ynwa-player` for both `Play` and `Record` modes.

**Non-Goals:**
- Introducing half-time breaks or passing remaining match time into Lua decision scripts (can be added later if tactical scripts need match clock awareness).

## Decisions

1. **Remove `FootballEvent::GameEnd` and `GAME_DURATION` from `ynwa-football`**:
   - Delete `FootballEvent::GameEnd`, `GAME_DURATION`, and `check_game_end` from `ynwa-football/src/events.rs`, and remove the `FootballEvent::GameEnd` arm in `FootballGameManager::handle_event`.
   - Every remaining `FootballEvent` variant (`Goal`, `Touchline`, `GoalLine`) now carries a `Team`, eliminating the special-case `GameEnd` team-rejection validation in `ynwa-integration-testing/src/dto.rs`.

2. **Client-owned duration in `ynwa-player`**:
   - Add `pub const DEFAULT_MATCH_DURATION: f32 = 120.0;` and an optional `--duration <seconds>` CLI flag (`f32 > 0.0` and finite, rejected with `--replay`) in `ynwa-player/src/cli.rs`.
   - In `advance(world, simulation, duration)`: only step `world.step(timestep)` while `world.game().state().elapsed_time < duration`.
   - In `run_record`: once `world.game().state().elapsed_time >= cli.duration`, call `world.game_mut().finish_journal()` and exit the loop.
   - In `run_play`: once `world.game().state().elapsed_time >= cli.duration`, stop stepping while keeping the window open for inspection until `Escape`.

## Risks / Trade-offs

- **[Risk] `Play` mode in `ynwa-player` previously kept stepping past `120.0` s into broken `GameOver` physics** → Mitigation: Stopping `World::step` once `elapsed_time >= duration` in `Play` mode fixes that bug cleanly while keeping the UI responsive.

