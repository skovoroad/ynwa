# Design

## Context

See `proposal.md` for motivation and `context/vision/dsl_cycles.md` for the agreed DSL loop design (`for player in teammates ... [order by ...]`, exiting only via `decision:`). Currently, `teams/team_{a,b,c,d}/preamble.lua` unroll candidate checks manually or define arity-specific helpers (`nearest_teammate_number`, `nearest_teammate_ahead_number`, `center_back_with_ball`, `pass_to_first_free_ahead`, `pass_to_free_or_nearest`, `nonforward_with_ball`, and `forward_with_ball`).

## Goals / Non-Goals

**Goals:**
- Replace unrolled candidate checks in `team_a`, `team_b`, `team_c`, and `team_d` preambles with loops matching the future DSL compilation pattern.
- Document at each rewritten loop in Lua that it is intentionally written to match the future DSL loop design.
- Preserve exact tactical behavior (tie-breaking order, pass reasons, fallback actions) across all existing teams.

**Non-Goals:**
- Adding sorting or higher-order iteration helpers to `stdlib.lua`.
- Implementing the DSL parser/transpiler itself.
- Changing `player_play` dispatch or player TOML configurations.

## Decisions

### 1. Provide `teammate_numbers()` and `opponent_numbers()` in `stdlib.lua`, keep loop/sort inline
- **Decision**: Add `teammate_numbers()` (returning squad numbers of all teammates **excluding `me`**) and `opponent_numbers()` (returning squad numbers of all opponents) to `stdlib.lua`. Express both priority-order loops (`ipairs(...)`) and distance-ordered loops (precomputing `{p, key, idx}`, `table.sort` with index tie-breaker, then `ipairs` with early `return`) directly in `teams/*/preamble.lua`.
- **Rationale**: Allows full-team iteration without exposing `INTERNAL` `get_teammates()` / `get_opponents()` tables (and their `.position` / `.index` fields) or risking self-passes (`me` is excluded from `teammate_numbers()`), while keeping `order by` compilation inline until the DSL transpiler emits Lua directly.
- **Alternative considered**: Adding a `sort_by_distance` helper to `stdlib.lua`. Rejected because `order by` in the DSL can sort by different targets (`me`, `opposite_goal`, etc.) and direction (`ascending`/`descending`), which a transpiler emits cleanly per loop.

### 2. Preserve tie-breaking stability via original index
- **Decision**: When sorting candidate numbers with `table.sort`, store `{p = number, key = distance(...), idx = i}` and break ties by `a.idx < b.idx`.
- **Rationale**: Lua's `table.sort` is not guaranteed to be stable. Preserving candidate list order on equal distances matches the existing unrolled `<` comparisons (`d2 < best_distance`) and the future DSL semantics.

### 3. Annotate rewritten loops with an explicit DSL-design comment
- **Decision**: Place a short comment next to each rewritten loop stating that the loop structure intentionally matches the future DSL design (`for player in teammates ... [order by ...]`).
- **Rationale**: Prevents future contributors or reviewers from treating these loops as violations of the "no general collection iteration" authoring principle.

## Risks / Trade-offs

- [Minor behavior drift on edge cases (e.g., `team_d` `forward_with_ball` passing to self or tie-breaking)] -> Verify all existing `ynwa-script-tests` pass and preserve exact conditions (`my_number ~= n`, `is_teammate_free`, `is_teammate_ahead`) and decision reason strings (`"pass_to_#" .. n` / `"pass to " .. n`).

