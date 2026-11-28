# Proposal

## Why

Team tactics currently express bounded loops in raw Lua (`ipairs`, a `_` placeholder for the numeric key, and — for distance-ordered searches — hand-built `{number, key, idx}` tables plus `table.sort` with an index tie-breaker). This leaks Lua iteration mechanics into user scripts and forces authors to think about language specifics instead of game logic, at a time when the only goal of the transitional Lua is to stay close to the future DSL (`for player in teammates ... [order by ...]`).

## What Changes

- Add two public iterator helpers to `ynwa-scripts/preambles/stdlib.lua` that encapsulate the loop mechanics and are used identically at the call site:
  - `each(numbers)` — yields each number in list order (the priority-order loop).
  - `order_by_distance_asc(numbers)` — yields numbers sorted by ascending distance to `me`, preserving list order on ties (the "nearest eligible" loop).
- Rewrite the loops in `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`, `teams/team_c/preamble.lua`, and `teams/team_d/preamble.lua` to use these iterators with `for player in ... do`, so tactical behavior is unchanged but the Lua-specific sorting/iteration code moves into `stdlib.lua`.
- Refine Rule (4) of the team/player script authoring rules and the Layer 2 `stdlib.lua` catalog so that these iterator-based loops are explicitly permitted, while raw `get_teammates()` / `get_opponents()` and nested loops remain forbidden.
- Add tests in `ynwa-script-tests/tests/stdlib_functions.rs` covering `each` and `order_by_distance_asc` (order, tie-break stability, empty list, and interaction with `teammate_numbers()` / `opponent_numbers()`).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scripting-and-protocols`: Add `each(numbers)` and `order_by_distance_asc(numbers)` to the `stdlib.lua` public API, and refine Rule (4) to permit non-nested iterator loops over explicit squad-number lists or `teammate_numbers()` / `opponent_numbers()`.

## Impact

- `ynwa-scripts/preambles/stdlib.lua`
- `teams/team_a/preamble.lua`
- `teams/team_b/preamble.lua`
- `teams/team_c/preamble.lua`
- `teams/team_d/preamble.lua`
- `ynwa-script-tests/tests/stdlib_functions.rs`
- `openspec/specs/scripting-and-protocols/spec.md`

Spec scenarios added or modified by this change:

- MODIFIED scenario "Layer 2 `stdlib.lua` utilities, actions, and `make_decision()` dispatch" (catalog gains the two iterators, and records `teammate_numbers()` / `opponent_numbers()` alongside them).
- MODIFIED requirement "Team and Player Script Authoring Rules (Layers 3 and 4)" (Rule (4) wording).
- ADDED scenario "DSL-aligned loop via iterator helpers" under that requirement.
