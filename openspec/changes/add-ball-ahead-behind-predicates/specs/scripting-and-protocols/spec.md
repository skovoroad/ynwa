# Spec Delta

## MODIFIED Requirements

### Requirement: Engine Preamble Layer Boundaries (Layers 1 and 2)

Engine-owned Lua preambles in `ynwa-scripts/preambles/` SHALL separate state access from tactical helpers: Layer 1 `core.lua` MUST contain only read-only wrappers over `context` and team-indexed `GAME_DATA.zones` (`_a`/`_b`) with zero game logic or decision construction; Layer 2 `stdlib.lua` MUST build utilities, predicates, decision constructors, and `make_decision()` over `core.lua` without reading `context` or raw `GAME_DATA.zones.*_a/b` directly (team-invariant `GAME_DATA.field` is allowed). `stdlib.lua` MUST be wrapped in a single `do ... end` block whose top section holds internal `local` helpers and constants, separated by a dedicated comment from the public API section, so internal helpers are not visible to team preambles concatenated into the same Lua chunk.

#### Scenario: Layer 1 `core.lua` function catalog
- **WHEN** Layer 2–4 scripts query game state via `core.lua`
- **THEN** `core.lua` provides `ball_owner()`, `my_position()`, `my_regions()`, `ball_position()`, `get_teammates()` (includes me), `get_opponents()`, `my_index()`, `my_team_name()`, `get_ball_owner_team()`, `get_opponent_goal()` (`goal_b` for `"A"`, `goal_a` for `"B"`), `get_own_goal()` (`goal_a` for `"A"`, `goal_b` for `"B"`), and `get_opponent_penalty_area()` (`penalty_area_b` for `"A"`, `penalty_area_a` for `"B"`)
- **AND** only `my_position()`, `ball_position()`, `my_team_name()`, and `get_ball_owner_team()` belong to the user-facing API; the other Layer 1 functions are marked internal by comment

#### Scenario: Layer 2 `stdlib.lua` utilities, actions, and `make_decision()` dispatch
- **WHEN** Layer 3–4 scripts call `stdlib.lua` public functions
- **THEN** `stdlib.lua` provides:
  - queries: `distance(p1, p2)` (2D XZ, ignoring `y`), `teammate_position(number)`, `opponent_position(number)`, `ball_owner_number()`, `count_teammates_around(pos, radius)`, `count_opponents_around(pos, radius)`
  - predicates: `is_in_region(from, to)`, `is_in_opponent_penalty_area()`, `is_teammate_free(number)`, `is_opponent_free(number)`, `is_teammate_ahead(number)`, `is_teammate_behind(number)`, `is_teammate_left(number)`, `is_teammate_right(number)`, `is_opponent_ahead(number)`, `is_opponent_behind(number)`, `is_opponent_left(number)`, `is_opponent_right(number)`, `is_ball_ahead()`, `is_ball_behind()`
  - actions: `stop(reason?)`, `chase_ball()`, `run_to_region(from, to)`, `run_to_start_position()`, `run_to_attack_position()`, `run_to_defence_position()`, `run_to_opponent_penalty_area()`, `kick_to_position(pos, reason?)`, `kick_to_opponent_goal()`, `default_goalkeeper_cover_position()` (runs to `z` center of the `"defence"` region and `x = clamp(ball.x, own_goal.min_x, own_goal.max_x)`, or falls back to `run_to_defence_position()` if `"defence"` is absent), and `default_goalkeeper_with_ball()` (passes to the nearest teammate at least 15 m away with reason `"pass_to_#<n>"`, or `kick_to_opponent_goal()` if none)
- **AND** `parse_col`, `parse_notation`, `is_in_region_obj`, `run_to_region_obj`, `get_teammate_by_number`, and `am_i_ball_owner` are internal `local` helpers that evaluate to `nil` in the team preamble scope
- **AND** `pass_to_teammate`, `pass_to_players_by_numbers`, `kick_to_cell`, and `kick_to_region` do not exist
- **AND** `make_decision()` classifies state in priority order `"i_have_ball"` (the current player is the ball owner) -> `"ball_is_free"` (`get_ball_owner_team() == "None"`; does NOT trigger on an in-flight pass because `owner_team` stays `"A"`/`"B"`) -> `"team_has_ball"` (`get_ball_owner_team() == my_team_name()`) -> `"opponent_has_ball"`, invoking `player_play[state]` -> `team_play[state]` -> `error(...)`

#### Scenario: Counting players around a position
- **WHEN** a script calls `count_teammates_around(pos, radius)` or `count_opponents_around(pos, radius)`
- **THEN** the result is the number of players of that team (for teammates, including me) whose `distance` to `pos` is strictly less than `radius`

#### Scenario: Loop-free pressing among the three closest teammates
- **WHEN** a script evaluates `count_teammates_around(ball_position(), distance(my_position(), ball_position())) < 3`
- **THEN** the result is `true` exactly when fewer than 3 teammates are strictly closer to the ball than me

#### Scenario: Free player predicates
- **WHEN** a script calls `is_teammate_free(number)` or `is_opponent_free(number)`
- **THEN** the result is `true` exactly when no player of the other team is strictly within 5 m of that player

#### Scenario: Directional predicates in team orientation
- **WHEN** a script calls `is_teammate_ahead/behind/left/right(number)` or `is_opponent_ahead/behind/left/right(number)`
- **THEN** with tolerance 0.5 m, "ahead" is `target.z > my.z + 0.5` (closer to the opponent goal line), "behind" is `target.z < my.z - 0.5`, "left" is `target.x < my.x - 0.5`, and "right" is `target.x > my.x + 0.5`

#### Scenario: Ball directional predicates without tolerance
- **WHEN** a script calls `is_ball_ahead()` or `is_ball_behind()`
- **THEN** with no tolerance, `is_ball_ahead()` is `true` exactly when `ball_position().z > my_position().z` (the ball is strictly closer to the opponent goal line than me), and `is_ball_behind()` is `true` exactly when `ball_position().z < my_position().z` (the ball is strictly closer to the own goal line than me)
- **AND** at exact equality (`ball_position().z == my_position().z`) both return `false`, and there are no `is_ball_left`/`is_ball_right` predicates

#### Scenario: Ball owner number
- **WHEN** a script calls `ball_owner_number()`
- **THEN** it returns the squad number of the player of either team who possesses the ball, or `nil` when nobody possesses it (including during an in-flight pass, while `get_ball_owner_team()` still reports the passing team)

#### Scenario: Kick to an opaque position
- **WHEN** a script calls `kick_to_position(teammate_position(7), "pass_to_#7")`
- **THEN** the decision is a kick targeting teammate 7's position with reason `"pass_to_#7"`; without `reason` the reason is `"kick_to_position(<x>,<z>)"`

#### Scenario: Unknown player number error handling
- **WHEN** a script calls `teammate_position(number)`, `opponent_position(number)`, or any `is_teammate_*` / `is_opponent_*` predicate with a squad number that does not exist on that team
- **THEN** `stdlib.lua` raises a descriptive Lua `error(...)` instead of returning `nil` or falling back to a default kick
- **AND** squad numbers are a provisional player identifier, to be revisited together with the DSL "player" type
