# Proposal

## Why

Writing team tactics in Lua without loops leads to duplicated per-player branches and arity-specific helpers (such as `nearest_teammate_number` with 3 or 4 parameters, `nearest_teammate_ahead_number`, `center_back_with_ball`, `pass_to_first_free_ahead`, `pass_to_free_or_nearest`, and `nonforward_with_ball`). Allowing bounded loops over explicit player lists—mirroring the future DSL `for player in teammates ... [order by ...]` construct—eliminates this boilerplate while preserving the DSL's simplicity constraints.

## What Changes

- Add safe public `teammate_numbers()` (excluding `me`) and `opponent_numbers()` helpers to `ynwa-scripts/preambles/stdlib.lua` so team scripts can iterate over full teams without touching raw `INTERNAL` `get_teammates()` / `get_opponents()` tables.
- Clarify the team/player script authoring rules in `scripting-and-protocols` so that DSL-aligned loops over explicit squad-number lists or `teammate_numbers()` / `opponent_numbers()` (including an `order by`-style stable sort) are permitted, while calling raw `INTERNAL` `get_teammates()` / `get_opponents()` or nesting loops remains forbidden.
- Rewrite the duplicated unrolled functions in `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`, `teams/team_c/preamble.lua`, and `teams/team_d/preamble.lua` using loops structured to match the future DSL design.
- Add comments next to each rewritten loop implementation noting that the loop is intentionally structured to match the future DSL design (as documented in `context/vision/dsl_cycles.md`).

## Capabilities

### New Capabilities

### Modified Capabilities
- `scripting-and-protocols`: Add `teammate_numbers()` and `opponent_numbers()` to `stdlib.lua`, and refine Rule (4) to permit DSL-aligned loops over explicit candidate lists or full-team number lists (`teammate_numbers()`, `opponent_numbers()`), while keeping raw `get_teammates()` / `get_opponents()` and nested loops forbidden.

## Impact

- `openspec/specs/scripting-and-protocols/spec.md`
- `ynwa-scripts/preambles/stdlib.lua`
- `teams/team_a/preamble.lua`
- `teams/team_b/preamble.lua`
- `teams/team_c/preamble.lua`
- `teams/team_d/preamble.lua`
- `ynwa-script-tests` (tests for `teammate_numbers()` / `opponent_numbers()` and regression check)

