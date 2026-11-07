# Design

## Context

`run()` in `ynwa-integration-testing/src/runner.rs` steps a loaded world, evaluates stop criteria after each step, and breaks on the first satisfied one. Football boundary/scoring events (`GoalLine`, `Touchline`, `Goal`) are handled by `FootballGameManager` in the same `Play` step that `check_events` produces them: it emits `DecisionsReset`, `RestartSet`, and `StageChange(Setup)`, but the ball is pinned to `restart_position` only on the *next* `Setup` tick (see `FootballGameManager::update` and the replay-side `ReplaySetupBallPlacer`). A scenario that stops on that event therefore ends one tick before the ball reaches the restart point.

## Goals / Non-Goals

**Goals:**
- Let an event-stopped run advance one settling step so the stage transition's first full tick (ball placement, setup decision assignment) is captured.
- Keep the triggering event as the reported stop reason and keep all other stop semantics unchanged.

**Non-Goals:**
- No change to `stage`/`journal`/`steps`/`time` stops.
- No change to scenario data or to the player.
- Not addressing award/goal scenario duration (that is a separate concern).

## Decisions

1. **Trailing step only for `StopReason::Event`.** Event stops are the only kind where the stage flips in the triggering step and a settle tick is needed; `stage` stops are used by exact-journal placement scenarios and `time`/`steps` stops have a precise stop contract. (Alternative: unconditional trailing step — rejected, it would break exact-journal and time-stop semantics.)
2. **Do not evaluate criteria on the trailing step and keep the original stop reason.** The run still "stopped because the event fired"; the extra tick is a settle, not a new decision point. (Alternative: re-evaluate and possibly re-stop — rejected, ambiguous and changes the stop reason.)
3. **Insert the trailing step after the loop, before the script-error check, `finish_journal`, and final-state capture.** This records the trailing tick and still fails the run if it produced a script error (event stops lead to `Setup`/`GameOver` ticks, which never call Lua, so none are expected).
4. **`RunOutcome.steps` and the recording footer count the trailing step for event stops.** Document that `expected_steps` on an event stop would now mean "triggering step + 1"; no current scenario asserts `steps` on an event stop.

## Risks / Trade-offs

- [An event-stopped scenario that used `journal_match = "exact"` or asserted `steps` would change] → none exist today; exact journals are used by `stage`-stopped placement scenarios, and the runner change is scoped to event stops.
- [Extra journal entries from the trailing `Setup` tick] → event-stopped scenarios use `journal_match = "subsequence"`, so ordered inclusion still holds.
- [Live run vs physics-only replay must stay equivalent on the trailing tick] → the replay world already runs `ReplaySetupBallPlacer` on `Setup` ticks and `ReplayDriver` replays the trailing `DecisionAssigned` entries; `tests/replay.rs` (`stdlib_kick_to_goal`, a `Goal` event) exercises this path.
