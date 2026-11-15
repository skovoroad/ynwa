# Design

## Context

See `proposal.md` — Why. After `client-owned-match-duration`, `GameStage` is `Play | Setup(String) | GameOver`, but nothing in production transitions to `GameOver`; it is reachable only through test fixtures and the scenario TOML `stage = "GameOver"` path. Core systems do not branch on `GameOver` as a terminal state: `PlayerReactionSystem` and `BallPossessionSystem` only special-case `Setup(_)`, and `ScriptedDecisionMaker` turns `GameOver` into `Stop`. Removing the variant collapses the model to the two states the football layer actually produces.

## Goals / Non-Goals

**Goals:**
- Delete `GameStage::GameOver` and every branch, DTO variant, and test fixture that references it.
- Keep `GameStage::Play` and `GameStage::Setup(String)` as the only stages; the client loop continues to own "match over".

**Non-Goals:**
- Introducing a replacement terminal state or a "finished" flag for recordings.
- Changing how `World::step`, physics, or the client loop decide when to stop stepping.

## Decisions

1. **Remove the enum variant rather than redefining it as a hard terminal** — a real terminal state (freeze physics, suppress decisions) would need a product requirement it does not yet have. Removing it makes the unreachable state unrepresentable, matching the project's type-safety rule.

2. **`Game::with_stage` collapses `Play | GameOver` to `Play`** — `GameOver` used the same `regions["start"].center()` positioning as `Play`, so the `match &stage` becomes `Setup(_)` vs `Play`.

3. **`ScriptedDecisionMaker::make_decision` drops the `GameOver` arm** — the remaining `match` is `Play` (delegate) and `Setup(_)` (error), which stays exhaustive.

4. **`PlayerReactionSystem` and `BallPossessionSystem` need no code change** — they already express the distinction as `matches!(stage, GameStage::Setup(_))`, so removing the variant only requires the `core-simulation` spec text update, not new branches.

5. **Integration-testing drops the `GameOver` stage everywhere**:
   - `StageNameDto` loses `GameOver`; `StageMatcher` loses `GameOver`; `stage_matcher`, `loader::resolve_stage`, and `compare::{stage_matches, stop_stage_matches}` drop their `GameOver` arms.
   - Scenario TOML `stage` / `stage_change` accept only `"Play" | "Setup"`.
   - The `failing/final_state.toml` fixture currently asserts `stage = "GameOver"` to force a failure; it is rewritten to a valid stage (e.g. `"Play"`) so the scenario still fails on its `score` expectation.

## Risks / Trade-offs

- **[Risk] Serde representation change** — `GameStage` is serialized in `JournalEvent::StageChange`, `GameState`, and `PartialStateSpec`. Removing a variant changes those wire formats. → No accumulated production data or external consumers exist, so a clean break is acceptable (project rule #11).
- **[Risk] A scenario/fixture still references `"GameOver"`** — such a file would now fail TOML parsing instead of running. → The tasks update the known references (`loader_tests.rs`, `dto_tests.rs`, `compare_tests.rs`, `criterion_tests.rs`, `tests/fixtures/failing/final_state.toml`, and `ynwa-player/src/ui_tests.rs`) and a workspace grep for `GameOver` is the acceptance check.

## Migration Plan

None — no production data or external API. Delete the variant and its references in one change.

## Open Questions

None.
