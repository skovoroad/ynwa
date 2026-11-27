## MODIFIED Requirements

### Requirement: Engine Config and `GAME_DATA` JSON Schema

Engine initialization SHALL pass `{"team_preambles": {"team_a", "team_b"}, "players": [{"script", "team", "static_data"}]}` where each player's `static_data` (exposed as `GAME_DATA`) contains `field: {width, length, columns, rows}`, `physics: {max_player_speed, max_kick_speed, ball_friction}` sourced from `ynwa-core` constants, and `zones` transformed into that player's team orientation (`Team::B` coordinates and arc angles flipped, while zone keys `<name>` / `<name>_a` / `<name>_b` keep their canonical names).

#### Scenario: Zone JSON representations in `GAME_DATA.zones`
- **WHEN** `Field` zones are serialized into `GAME_DATA.zones`
- **THEN** rectangles serialize as `{"type": "rectangle", "min_x", "max_x", "min_z", "max_z"}`, circles as `{"type": "circle", "center_x", "center_z", "radius"}`, arcs as `{"type": "arc", "center_x", "center_z", "radius", "start_angle", "end_angle"}` in **degrees** (`Team::B` currently emits `[180 - end, 180 - start]`, a known latent defect tracked in `fix-team-b-arc-angle-flip`), and points as `{"type": "point", "x", "z", "tolerance": 0.5}`

#### Scenario: Physics constants in `GAME_DATA.physics`
- **WHEN** a player's `GAME_DATA` is initialized
- **THEN** `GAME_DATA.physics` contains `max_player_speed`, `max_kick_speed`, and `ball_friction` matching the engine's `MAX_SPEED_METERS_PER_SECOND`, `kick_speed(100)`, and `BALL_FRICTION_DECELERATION` constants

### Requirement: Per-Decision `context` JSON Schema and Semantics

Per-tick `context` SHALL be constructed in the caller's team orientation with `me`, `teammates`, `opponents`, `ball`, and `game`. `context.teammates` MUST include **all** players on `me.team` in ascending global `index` order **including the caller itself** (`index == me.index`). `context.ball.owner_index` MUST serialize `possessed_by` (`usize | null`), whereas `context.ball.owner_team` MUST serialize `last_possessing_team` (`"A" | "B" | "None"`). `context.me`, `context.teammates[*]`, `context.opponents[*]`, and `context.ball` MUST each include a `velocity: {x, z}` vector transformed into the caller's team orientation (`x` and `z` negated for `Team::B`).

#### Scenario: Exact `context` fields, `display_notation`, and kick ownership retention
- **WHEN** a player kicks the ball so physical `possessed_by` becomes `None` while `last_possessing_team` remains with that player's team
- **THEN** `context` provides `me: {team, number, index, position: {x, y, z}, velocity: {x, z}, regions: {<key>: {min_x, max_x, min_z, max_z, display_notation}}}` (where `display_notation` is `"<display>"` for `Team::A` and `"<display> (<team_b>)"` for `Team::B`), `teammates: [{index, number, position, velocity}, ...]` (including `me.index`), `opponents: [{index, number, position, velocity}, ...]`, `ball: {position: {x, y, z}, velocity: {x, z}, owner_index: null, owner_team: "<me.team>"}`, and `game: {elapsed_time}`

#### Scenario: Team B velocity orientation flip
- **WHEN** `context` is constructed for a `Team::B` player while a player or the ball has display-orientation velocity `(vx, vz)`
- **THEN** the serialized `velocity` in `context` is `{x: -vx, z: -vz}` so positive `z` always points toward the opponent goal and positive `x` toward the right wing

### Requirement: Engine Preamble Layer Boundaries (Layers 1 and 2)

Engine-owned Lua preambles in `ynwa-scripts/preambles/` SHALL separate state access from tactical helpers: Layer 1 `core.lua` MUST contain only read-only wrappers over `context` and team-indexed `GAME_DATA.zones` (`_a`/`_b`) with zero game logic or decision construction; Layer 2 `stdlib.lua` MUST build utilities, predicates, decision constructors, and `make_decision()` over `core.lua` without reading `context` or raw `GAME_DATA.zones.*_a/b` directly (team-invariant `GAME_DATA.field` and `GAME_DATA.physics` are allowed). `stdlib.lua` MUST be wrapped in a single `do ... end` block whose top section holds internal `local` helpers and constants, separated by a dedicated comment from the public API section, so internal helpers are not visible to team preambles concatenated into the same Lua chunk.

#### Scenario: Layer 1 `core.lua` function catalog
- **WHEN** Layer 2–4 scripts query game state via `core.lua`
- **THEN** `core.lua` provides `ball_owner()`, `my_position()`, `my_velocity()`, `my_regions()`, `ball_position()`, `ball_velocity()`, `get_teammates()` (includes me), `get_opponents()`, `my_index()`, `my_team_name()`, `get_ball_owner_team()`, `get_opponent_goal()` (`goal_b` for `"A"`, `goal_a` for `"B"`), `get_own_goal()` (`goal_a` for `"A"`, `goal_b` for `"B"`), and `get_opponent_penalty_area()` (`penalty_area_b` for `"A"`, `penalty_area_a` for `"B"`)
- **AND** only `my_position()`, `ball_position()`, `my_team_name()`, and `get_ball_owner_team()` belong to the user-facing API; the other Layer 1 functions (including `my_velocity()` and `ball_velocity()`) are marked internal by comment

#### Scenario: Layer 2 `stdlib.lua` utilities, actions, and `make_decision()` dispatch
- **WHEN** Layer 3–4 scripts call `stdlib.lua` public functions
- **THEN** `stdlib.lua` provides:
  - queries: `distance(p1, p2)` (2D XZ, ignoring `y`), `teammate_position(number)`, `opponent_position(number)`, `ball_owner_number()`, `count_teammates_around(pos, radius)`, `count_opponents_around(pos, radius)`
  - predicates: `is_in_region(from, to)`, `is_in_opponent_penalty_area()`, `is_teammate_free(number)`, `is_opponent_free(number)`, `is_teammate_ahead(number)`, `is_teammate_behind(number)`, `is_teammate_left(number)`, `is_teammate_right(number)`, `is_opponent_ahead(number)`, `is_opponent_behind(number)`, `is_opponent_left(number)`, `is_opponent_right(number)`, `is_ball_ahead()`, `is_ball_behind()`
  - actions: `stop(reason?)`, `chase_ball(reason?)`, `run_to_region(from, to, reason?)`, `run_to_start_position(reason?)`, `run_to_attack_position(reason?)`, `run_to_defence_position(reason?)`, `run_to_opponent_penalty_area(reason?)`, `kick_to_position(pos, reason?)`, `pass_to(number, reason?)`, `kick_to_opponent_goal(reason?)`, `default_goalkeeper_cover_position(reason?)`, and `default_goalkeeper_with_ball(reason?)` (passes via `pass_to(best_number, reason)` to the nearest teammate at least 15 m away, or `kick_to_opponent_goal(reason)` if none)
- **AND** `parse_col`, `parse_notation`, `is_in_region_obj`, `run_to_region_obj`, `get_teammate_by_number`, `am_i_ball_owner`, `clamp_to_field`, `teammate_velocity`, and `opponent_velocity` are internal `local` helpers that evaluate to `nil` in the team preamble scope
- **AND** `pass_to_teammate`, `pass_to_players_by_numbers`, `kick_to_cell`, and `kick_to_region` do not exist

#### Scenario: Lead-targeted `chase_ball`
- **WHEN** a script calls `chase_ball(reason?)` while the ball is moving
- **THEN** `stdlib.lua` computes a linear lead point `ball_pos + ball_vel * t_est` using `t_est = min(distance(my_pos, ball_pos) / AVG_PLAYER_SPEED, 0.5 * ball_speed / BALL_FRICTION, MAX_LEAD_TIME)`, clamps the target point inside `[0, field.width] x [0, field.length]`, and returns `{action = "run", target_type = "point", target = {x, z, y = 0}, reason = compose_reason(reason, "chase_ball")}`
- **AND** when the ball is stationary (`ball_speed == 0`), the target point equals `ball_position()`

#### Scenario: Lead-targeted `pass_to`
- **WHEN** a script calls `pass_to(number, reason?)`
- **THEN** `stdlib.lua` computes a linear lead point `teammate_pos + teammate_vel * t_est` using `t_est = min(distance(my_pos, teammate_pos) / AVG_PASS_SPEED, MAX_LEAD_TIME)`, clamps the target point inside `[0, field.width] x [0, field.length]` (with field margin), and returns `kick_to_position(target, compose_reason(reason, "pass_to_#" .. number))`
- **AND** when `number` does not exist on the caller's team, `pass_to` raises a descriptive Lua `error(...)`

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
