# Design

## Context

`ynwa-scripts/preambles/core.lua` and `stdlib.lua` form the engine-owned Lua runtime layers (Layer 1 and Layer 2). `DecisionEngine` concatenates `core.lua`, `stdlib.lua`, and the team `preamble.lua` into a single Lua chunk, so top-level `local` declarations in `stdlib.lua` are still visible to the team preamble. Currently:
- `context.opponents` is already serialized by `ScriptedDecisionMaker` in `ynwa-core`, but `core.lua` has no wrapper for it.
- `stdlib.lua` exposes internal grid/region helpers (`parse_col`, `parse_notation`, `is_in_region_obj`, `run_to_region_obj`), duplicated pass APIs (`get_teammate_by_number` + `pass_to_teammate`), and container/unused functions (`pass_to_players_by_numbers`, `run_to_start_position`, `kick_to_cell`, `kick_to_region`).
- `teams/team_a/preamble.lua` and `teams/team_b/preamble.lua` use player loops (`am_i_top3_closest_to_ball`, `pass_to_nearest_teammate`), raw coordinate reads (`my_position().z`, `fwd10.position.z`), hand-crafted `{action = "kick", ...}` tables, and functions that become internal or are removed (`pass_to_players_by_numbers`, `is_in_region_obj`, `my_regions`, `get_teammate_by_number`, `pass_to_teammate`).
- `ynwa-script-tests` (`tests/stdlib_functions.rs`, `fixtures/team_a.lua`, `fixtures/team_b.lua`, `scenarios/goal_kick_team*`) and `ynwa-integration-testing/scenarios/pass_*` (`pass_to_teammate(get_teammate_by_number(2))`) call several of these functions.

## Goals / Non-Goals

**Goals:**
- Expose `get_opponents()` in `core.lua` symmetrically with `get_teammates()`.
- Define an explicit user-facing API whitelist spanning `core.lua` and `stdlib.lua`; hide `stdlib.lua` internals via `do ... end`.
- Eliminate duplicated capabilities: one generic region API (string notation), one kick/pass API (`kick_to_position`).
- Provide `teammate_*` / `opponent_*` paired functions taking a squad `number` so team scripts never touch raw coordinates, player loops, or manual decision tables.
- Migrate `teams/team_a` and `teams/team_b` onto the public API in the same change.
- Keep `ynwa-script-tests` and `ynwa-integration-testing` green.

**Non-Goals:**
- Changing Rust simulation systems (`ynwa-core` / `ynwa-football`) or `GAME_DATA` / `context` JSON schemas.
- Modifying `static.toml`, `tactical.toml`, or player `script.lua` role assignments of `team_a` / `team_b`.
- Designing the custom DSL syntax or transpiler (later stages of `context/vision/plan.md`).

## Decisions

1. **User-facing API whitelist** (the only functions Layer 3–4 may call):
   - State (Layer 1): `my_position()`, `ball_position()`, `my_team_name()`, `get_ball_owner_team()`.
   - Queries (Layer 2): `distance(p1, p2)`, `teammate_position(n)`, `opponent_position(n)`, `ball_owner_number()`, `count_teammates_around(pos, r)`, `count_opponents_around(pos, r)`.
   - Predicates: `is_in_region(from, to)`, `is_in_opponent_penalty_area()`, `is_teammate_free(n)`, `is_opponent_free(n)`, `is_teammate_ahead/behind/left/right(n)`, `is_opponent_ahead/behind/left/right(n)`.
   - Actions: `stop(reason?)`, `chase_ball()`, `run_to_region(from, to)`, `run_to_attack_position()`, `run_to_defence_position()`, `run_to_opponent_penalty_area()`, `kick_to_position(pos, reason?)`, `kick_to_opponent_goal()`, `default_goalkeeper_cover_position()`, `default_goalkeeper_with_ball()`.
   - Everything else in `core.lua` is internal by comment (it must stay global for `stdlib.lua`); everything else in `stdlib.lua` is `local`. `make_decision()` stays global (called by the engine), but is not part of the user-facing API.

2. **`stdlib.lua` layout and hiding:**
   ```lua
   do
       -- =====================================================================
       -- Internal implementation (not part of the public API)
       -- =====================================================================
       local FREE_PLAYER_RADIUS = 5.0
       local DIRECTION_TOLERANCE = 0.5
       local function parse_col(s) ... end
       -- parse_notation, is_in_region_obj, run_to_region_obj,
       -- get_teammate_by_number, find_opponent_by_number, am_i_ball_owner, ...

       -- =====================================================================
       -- Public API
       -- =====================================================================
       function distance(p1, p2) ... end
       -- ...
       function make_decision() ... end
   end
   ```
   Internal locals live at the top (Lua requires declaration before use); public functions are globals defined inside the block and close over the internals, which are invisible after `end` to the concatenated team preamble.

3. **Regions — single public form:** the generic region API is the string-notation pair `is_in_region(from, to)` / `run_to_region(from, to)`. `is_in_region_obj` / `run_to_region_obj` / `my_regions()` stay internal and back the named tactical shortcuts (`run_to_attack_position`, `run_to_defence_position`, `is_in_opponent_penalty_area`, `run_to_opponent_penalty_area`). Richer region constructs are deferred to the DSL.

4. **Counting around a position:** `count_teammates_around(pos, r)` / `count_opponents_around(pos, r)` count all players of that team (including me for teammates) with `distance(player, pos) < r` (strict).

5. **Pressing (loop-free top-3):** "I am among the 3 teammates closest to the ball" is expressed with my own distance as the radius:
   ```lua
   local d = distance(my_position(), ball_position())
   return count_teammates_around(ball_position(), d) < 3
   ```
   Strict `<` excludes me (my distance is not less than itself), so this reproduces the former `am_i_top3_closest_to_ball` exactly. `challenge_distance()` is not introduced: no tactic needs it.

6. **Squad-number lookups:** `teammate_position(n)` / `opponent_position(n)` return the player's `position` table (opaque to Layer 3–4) and raise `error("unknown teammate number: " .. tostring(n))` (or opponent) when absent. All `is_teammate_*` / `is_opponent_*` predicates use the same lookups and errors.
   - *DSL note:* squad numbers serve as player identifiers here; `context/vision/dsl.md` foresees a dedicated "player" type. Revisit in plan step 4 (also: error-on-unknown-number vs. future substitutions). The same note goes into the spec and as a code comment.

7. **`ball_owner_number()`:** maps the internal `ball_owner()` index to a squad number via `get_teammates()` / `get_opponents()`. Returns `nil` when nobody possesses the ball — this includes an in-flight pass, where `get_ball_owner_team()` still reports the passing team. Combined with `get_ball_owner_team()` it identifies the owner fully.
   - *DSL note:* the DSL type list has no "nothing" value; revisit in plan step 4.

8. **Free player and directions:**
   - `is_teammate_free(n)` = `count_opponents_around(teammate_position(n), FREE_PLAYER_RADIUS) == 0`; `is_opponent_free(n)` = `count_teammates_around(opponent_position(n), FREE_PLAYER_RADIUS) == 0`. `FREE_PLAYER_RADIUS = 5.0`.
   - Team orientation: "ahead" means closer to the opponent goal line, i.e. `target.z > my.z + DIRECTION_TOLERANCE`; "behind" `target.z < my.z - DIRECTION_TOLERANCE`; "left" `target.x < my.x - DIRECTION_TOLERANCE` (`X = 0` is the left wing); "right" `target.x > my.x + DIRECTION_TOLERANCE`. `DIRECTION_TOLERANCE = 0.5` m.

9. **`kick_to_position(pos, reason?)`:** returns a kick to `pos`; `reason` defaults to `string.format("kick_to_position(%.1f,%.1f)", pos.x, pos.z)`. Passes use `kick_to_position(teammate_position(n), "pass_to_#" .. n)`, preserving the former `pass_to_teammate` reason (asserted by integration scenarios).

10. **`default_goalkeeper_with_ball()` (temporary):** pass to the nearest teammate at least `15.0` m away with reason `"pass_to_#<n>"`; if none, `kick_to_opponent_goal()`. Identical to the former team-level `pass_to_nearest_teammate`; the loop lives inside `stdlib.lua` until dedicated goalkeeper primitives are designed for the DSL.

11. **Team migration and behavior changes:**

    | Location | Before | After | Behavior change |
    |---|---|---|---|
    | `team_a`, `team_b` `team_play` pressing | `am_i_top3_closest_to_ball()` loop | Decision 5 formula | None |
    | `team_a`, `team_b` `goalkeeper_play.i_have_ball` | team-local `pass_to_nearest_teammate` | `default_goalkeeper_with_ball` | None |
    | `team_a` `defender_with_ball` / `midfielder_with_ball` | `pass_to_players_by_numbers({6,7,8})` / `({9,10,11})` | sequential `if` distance comparisons picking the nearest of the listed numbers, then `kick_to_position(teammate_position(n), "pass_to_#" .. n)` | None in practice (the former goal-kick fallback only fired when no listed number existed; now an unknown number raises an error) |
    | `team_b` `nonforward_with_ball` | `fwd.position.z > my_z` | `is_teammate_ahead(10)` / `(11)` + `kick_to_position(...)` | A forward must be more than 0.5 m ahead to receive the pass |
    | `team_b` `forward_with_ball` | shoot if in penalty area or in own `attack` region; otherwise `run_to_attack_position()` | shoot if `is_in_opponent_penalty_area()`; otherwise `run_to_opponent_penalty_area()` | Forward no longer shoots from its attack region; it carries the ball into the penalty area first |

12. **Test & fixture migration:**
    - `ynwa-script-tests/fixtures/team_a.lua`, `team_b.lua`: replace `run_to_start_position()` with `run_to_defence_position()`.
    - `ynwa-script-tests/scenarios/goal_kick_team*`: replace `kick_to_cell(...)` / `kick_to_region(...)` with `kick_to_position(...)` / `kick_to_opponent_goal()` / `run_to_region(...)`.
    - `ynwa-integration-testing/scenarios/pass_*/teams/*/players/01/script.lua`: replace `pass_to_teammate(get_teammate_by_number(2))` with `kick_to_position(teammate_position(2), "pass_to_#2")`.
    - `ynwa-script-tests/tests/stdlib_functions.rs`: remove direct tests of now-internal/removed functions (`parse_col`, `parse_notation`, `is_in_region_obj`, `get_teammate_by_number`, `am_i_ball_owner`, `pass_to_teammate`, `pass_to_players_by_numbers`) — their behavior stays covered through public functions (`is_in_region`, `run_to_region`, `teammate_position`, `make_decision`); add tests for every new public function and assert that internal helpers are `nil` from the team-preamble scope.

## Risks / Trade-offs

- **Hiding only `stdlib.lua`:** internal `core.lua` functions remain reachable globals; the whitelist is enforced by the authoring rules and, later, by the DSL.
- **Temporary `default_goalkeeper_with_ball()`:** encapsulates a loop inside `stdlib.lua`; replaced once goalkeeper distribution primitives are designed for the DSL.
- **Ordering with `narrow-script-tests-scope`:** that change is applied later and must be revised to drop the tests of internal helpers removed here.
