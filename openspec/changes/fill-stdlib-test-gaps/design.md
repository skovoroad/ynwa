# Design

## Context

`ynwa-script-tests/tests/stdlib_functions.rs` is the deterministic harness that executes the real `core.lua`/`stdlib.lua` and asserts exact `Decision`/`reason`. Four public functions have no test (`chase_ball`, `run_to_defence_position`, `run_to_opponent_penalty_area`, `stop`), and `run_to_attack_position` is not exercised — the existing `test_dispatch_team_has_ball_runs_to_attack` actually routes through `ball_is_free` → `run_to_start_position` because it never sets ball ownership. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- Give every public `stdlib.lua` function a direct test asserting its `Decision` and `reason` (or return value).
- Add a spec requirement enforcing this coverage.

**Non-Goals:**
- No production (`stdlib.lua`) code changes.
- No changes to `ynwa-integration-testing` (full-simulation) or `ynwa-core`.

## Decisions

- **Direct tests, not dispatch**: each new test calls the function from `make_decision()` and asserts the resulting `Decision` and `reason`, matching the existing `test_run_to_start_position` / `test_kick_to_position` style.
- **Region-dependent actions** (`run_to_attack_position`, `run_to_defence_position`) require `"attack"`/`"defence"` in `my_regions()`; the current `start_regions` helper only injects `"start"`. Extend the test helper to provide all three keys (or add a dedicated regions helper), reusing the exact keys consumed by `my_regions()`.
- **`run_to_opponent_penalty_area`** needs the football field's penalty-area zone: use `create_test_game_football_field_with_preambles` (already used by `test_kick_to_opponent_goal`).
- **`stop(reason)`**: direct test asserting `Decision::Stop` with the given reason.

## Risks / Trade-offs

- [A new region helper could drift from real `tactical.toml` region names] → Reuse the exact keys `"attack"`/`"defence"`/`"start"` already consumed by `my_regions()`.
- [A test could pass while the function is silently unexercised] → Each test asserts a distinctive `reason` string, so a fallback (e.g. `stop("no_attack_position")`) would fail the assertion.
