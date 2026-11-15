# Proposal

## Why

After `client-owned-match-duration` removed `FootballEvent::GameEnd` (the only production path into `GameStage::GameOver`), `GameStage::GameOver` is unreachable from live play and no longer expresses a terminal state: in the core it behaves like `Play` with every decision forced to `Stop`. The simulation model should express only the states the football layer actually produces — `Play` and `Setup(reason)` — and the client loop now owns "match over" entirely. This change removes the dead `GameOver` variant and the now-impossible `team`/stage handling built around it.

## What Changes

- **BREAKING**: Remove `GameStage::GameOver` from `ynwa-core::game`, simplifying `Game::with_stage` player positioning so only `Play` uses `regions["start"].center()`.
- Remove the `GameOver` arm from `ScriptedDecisionMaker::make_decision` (`Play` delegates, `Setup(_)` errors) in `ynwa-core`.
- Remove the now-unreachable `GameStage::GameOver => {}` arm in `FootballGameManager::update` in `ynwa-football`.
- **BREAKING**: Remove `"GameOver"` from the `ynwa-integration-testing` stage schema (`StageNameDto`, `StageMatcher`, `stage_matcher`, `loader::resolve_stage`, `compare`, and the `stage` / `stage_change` values accepted in scenario TOML).

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `core-simulation`: `GameStage` no longer has a `GameOver` variant; `Game::with_stage` and `PlayerReactionSystem` only distinguish `Play` from `Setup(_)`.
- `scripting-and-protocols`: `ScriptedDecisionMaker` delegates in `Play` and errors in `Setup(_)`, with no `GameOver` fallback.
- `integration-testing`: scenario TOML `stage` and `stage_change` values drop `"GameOver"`.

## Impact

- `ynwa-core`: `src/game.rs`, `src/systems/decision/scripted_decision_maker.rs`, and tests (`src/tests/serde_tests.rs`, decision/scripted tests).
- `ynwa-football`: `src/game_manager.rs`.
- `ynwa-player`: `src/ui_tests.rs`.
- `ynwa-integration-testing`: `src/dto.rs`, `src/criterion.rs`, `src/loader.rs`, `src/compare.rs`, and tests (`compare_tests.rs`, `criterion_tests.rs`, `dto_tests.rs`, `loader_tests.rs`).

Prerequisite: archive/apply `client-owned-match-duration` first, since that change removes the production path that made `GameOver` reachable.
