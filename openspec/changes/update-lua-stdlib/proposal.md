# Proposal

## Why

The current Lua standard library (`core.lua` and `stdlib.lua`) exposes internal grid/region helpers, duplicated region and pass APIs, and container-based functions, while lacking opponent access, positional predicates, ball-owner identification, and distance/radius queries needed by target tactics and future DSL transpilation. The existing teams (`teams/team_a`, `teams/team_b`) and test scenarios call functions that this cleanup removes or hides, so the library and all its callers are updated together in one change to keep the workspace green at every commit.

## What Changes

- **Layer 1 (`ynwa-scripts/preambles/core.lua`)**:
  - Add `get_opponents()` returning `context.opponents`, matching `get_teammates()`.
  - Mark Layer 1 functions that are not part of the user-facing API (`ball_owner`, `my_regions`, `get_teammates`, `get_opponents`, `my_index`, `get_opponent_goal`, `get_own_goal`, `get_opponent_penalty_area`) as internal by comment (they stay global because `stdlib.lua` uses them).
- **Layer 2 (`ynwa-scripts/preambles/stdlib.lua`)**:
  - Wrap the whole file in a `do ... end` block: an internal section at the top (`local` functions and constants, separated by a dedicated comment) followed by the public API section (global functions). Internal helpers become invisible to team preambles concatenated into the same Lua chunk.
  - **Internal (`local`)**: `parse_col`, `parse_notation`, `is_in_region_obj`, `run_to_region_obj`, `get_teammate_by_number`, `am_i_ball_owner`, plus new lookup helpers.
  - **Add** `count_teammates_around(position, radius)`, `count_opponents_around(position, radius)` (strict `< radius`, all players of that team including me).
  - **Add** opaque position lookups by squad number: `teammate_position(number)`, `opponent_position(number)` (erroring on unknown number).
  - **Add** `ball_owner_number()` (squad number of the possessing player of either team, `nil` when nobody possesses the ball, including an in-flight pass).
  - **Add** paired predicates: `is_teammate_free(number)`, `is_opponent_free(number)`, `is_teammate_ahead/behind/left/right(number)`, `is_opponent_ahead/behind/left/right(number)`.
  - **Add** `kick_to_position(position, reason?)` and the temporary `default_goalkeeper_with_ball()`.
  - **Keep** `distance(pos1, pos2)` public over opaque positions; keep string-notation `is_in_region(from, to)` / `run_to_region(from, to)` as the only public generic region API; keep `run_to_start_position()` alongside `run_to_attack_position()` and `run_to_defence_position()`.
  - **BREAKING** **Remove**: `pass_to_teammate`, `pass_to_players_by_numbers`, `kick_to_cell`, `kick_to_region`.
- **Layer 3–4 authoring rules**: forbid calling functions outside the user-facing whitelist, player-iteration loops, raw coordinate reads (`.x`/`.y`/`.z`), and hand-crafted decision tables.
- **Team migration (`teams/team_a`, `teams/team_b`)**: rewrite both preambles onto the public API (loop-free top-3 pressing, `default_goalkeeper_with_ball`, explicit `if` pass selection, directional predicates); behavior changes are listed in `design.md`.
- **Tests & Docs**: update `ynwa-script-tests` (tests, fixtures, `scenarios/goal_kick_team*`), `ynwa-integration-testing/scenarios/pass_*` scripts, and `context/user-docs/scripting_player.md`.

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `scripting-and-protocols`: Update Layer 1 (`core.lua`) and Layer 2 (`stdlib.lua`) catalogs, define the user-facing API whitelist, hide internal helpers via `do ... end`, remove duplicated/container helpers, and tighten Layer 3–4 authoring rules.

## Impact

- `ynwa-scripts/preambles/core.lua`, `ynwa-scripts/preambles/stdlib.lua`
- `teams/team_a/preamble.lua`, `teams/team_b/preamble.lua`
- `ynwa-script-tests/tests/stdlib_functions.rs`, `ynwa-script-tests/fixtures/`, `ynwa-script-tests/scenarios/goal_kick_team*`
- `ynwa-integration-testing/scenarios/pass_received_team_*/`, `ynwa-integration-testing/scenarios/pass_intercepted_team_*/` (player `01` scripts)
- `context/user-docs/scripting_player.md`
- Supersedes the former `align-team-tactics-with-stdlib` change. `add-target-tactic` depends on this change.
- Must be applied before `narrow-script-tests-scope`, which must then be revised (it references `parse_col`, `parse_notation`, `is_in_region_obj` tests that this change removes).
