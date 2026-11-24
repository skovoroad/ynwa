# Scripting and Protocols Specification

## Purpose

Defines the isolated sandboxed Lua decision engine (`ynwa-decisions`), `LuaExecutor` and error taxonomy, JSON wire contracts (`GAME_DATA`, `context`, and `LuaDecision`), the 4-layer Lua scripting architecture and authoring rules, `TeamRepository` (`ynwa-repository`), and the Lua script testing infrastructure (`ynwa-script-tests`).

User-authored Lua (`preamble.lua`, `script.lua`) is an intermediate representation for a future custom user-language transpiler; strict separation across engine preambles (`core.lua`, `stdlib.lua`) and team/player scripts preserves a clean compilation target even where the Lua VM does not enforce visibility boundaries. Authoritative function signatures and edge-case semantics for Layers 1–2 live inline in `ynwa-scripts/preambles/{core,stdlib}.lua`.

## Requirements

### Requirement: Per-Player Isolated Lua VM, Sandbox, and Timeout

`DecisionEngine` SHALL maintain one isolated `mlua::Lua` VM per player (`executors[player_index]`). Each VM MUST nil out unsafe host globals (`io`, `os`, `package`, `require`, `loadfile`, `dofile`, `load`, `debug`, `collectgarbage`, `_G`) while keeping pure standard libraries, and MUST enforce a `100` ms wall-clock timeout via an instruction hook every `10,000` VM instructions.

#### Scenario: Allowed standard libraries and infinite loop termination
- **WHEN** a player VM executes `make_decision()`
- **THEN** pure libraries (`math`, `string`, `table`, `ipairs`, `pairs`, `tonumber`, `tostring`, `type`, `select`, `unpack`, `error`, `assert`, `pcall`, `xpcall`, `next`) remain available, while `while true do end` aborts after `100` ms via the `10,000`-instruction hook (`DecisionEngineError::Timeout`) without affecting other player VMs

### Requirement: LuaExecutor Low-Level API and Error Taxonomy

`LuaExecutor` (`ynwa-decisions`) SHALL wrap a single sandboxed `mlua::Lua` VM exposing `new(preamble, timeout)`, `set_global`, and `execute` / `execute_with_args`, and `DecisionEngine` MUST map `ScriptError` variants into `DecisionEngineError` deterministically.

#### Scenario: `ScriptError` to `DecisionEngineError` mapping
- **WHEN** `LuaExecutor` or `DecisionEngine` reports a failure
- **THEN** `SyntaxError`, `FunctionNotFound`, `DeserializationError`, and `LuaDecision` schema failures map to `DecisionEngineError::ScriptError`; `RuntimeError`, `SerializationError`, and VM init failures map to `DecisionEngineError::RuntimeError`; `ScriptError::Timeout` maps to `DecisionEngineError::Timeout`; and invalid player indices or configs yield `InvalidPlayerIndex` or `InvalidConfig`

### Requirement: Lua VM Lifecycle and Stage Gating

At `DecisionEngine::new`, each VM SHALL execute `"{core_preamble}\n{stdlib_preamble}\n{team_preamble}"` **once** and bind `static_data` **once** to global `GAME_DATA`. On each `DecisionEngine::make_decision(idx, context)` call, the VM MUST overwrite global `context`, re-execute `script.lua`, and invoke `make_decision()`. `ScriptedDecisionMaker` MUST delegate in `Play` and return `Err(RuntimeError)` in `Setup(_)`.

#### Scenario: Preamble persistence vs player script re-execution
- **WHEN** `DecisionEngine::make_decision` runs across consecutive ticks for the same player
- **THEN** tables/functions declared in `core.lua`, `stdlib.lua`, and team `preamble.lua` persist across ticks within that player's VM, while top-level statements in `script.lua` re-execute on every call before `make_decision()` runs

### Requirement: Engine Config and `GAME_DATA` JSON Schema

Engine initialization SHALL pass `{"team_preambles": {"team_a", "team_b"}, "players": [{"script", "team", "static_data"}]}` where each player's `static_data` (exposed as `GAME_DATA`) contains `field: {width, length, columns, rows}` and `zones` transformed into that player's team orientation (`Team::B` coordinates and arc angles flipped, while zone keys `<name>` / `<name>_a` / `<name>_b` keep their canonical names).

#### Scenario: Zone JSON representations in `GAME_DATA.zones`
- **WHEN** `Field` zones are serialized into `GAME_DATA.zones`
- **THEN** rectangles serialize as `{"type": "rectangle", "min_x", "max_x", "min_z", "max_z"}`, circles as `{"type": "circle", "center_x", "center_z", "radius"}`, arcs as `{"type": "arc", "center_x", "center_z", "radius", "start_angle", "end_angle"}` in **degrees** (`Team::B` currently emits `[180 - end, 180 - start]`, a known latent defect tracked in `fix-team-b-arc-angle-flip`), and points as `{"type": "point", "x", "z", "tolerance": 0.5}`

### Requirement: Per-Decision `context` JSON Schema and Semantics

Per-tick `context` SHALL be constructed in the caller's team orientation with `me`, `teammates`, `opponents`, `ball`, and `game`. `context.teammates` MUST include **all** players on `me.team` in ascending global `index` order **including the caller itself** (`index == me.index`). `context.ball.owner_index` MUST serialize `possessed_by` (`usize | null`), whereas `context.ball.owner_team` MUST serialize `last_possessing_team` (`"A" | "B" | "None"`).

#### Scenario: Exact `context` fields, `display_notation`, and kick ownership retention
- **WHEN** a player kicks the ball so physical `possessed_by` becomes `None` while `last_possessing_team` remains with that player's team
- **THEN** `context` provides `me: {team, number, index, position: {x, y, z}, regions: {<key>: {min_x, max_x, min_z, max_z, display_notation}}}` (where `display_notation` is `"<display>"` for `Team::A` and `"<display> (<team_b>)"` for `Team::B`), `teammates: [{index, number, position}, ...]` (including `me.index`), `opponents: [{index, number, position}, ...]`, `ball: {position: {x, y, z}, owner_index: null, owner_team: "<me.team>"}`, and `game: {elapsed_time}`

### Requirement: `LuaDecision` Return Schema and Validation

Lua `make_decision()` SHALL return a table deserializable into `LuaDecision` (`action` in `"stop" | "run" | "kick"`) with optional top-level string `reason`, then parsed into domain `Decision` (still in team orientation). Invalid actions, missing targets, non-finite coordinates, or malformed grid notations MUST return an error.

#### Scenario: Valid Lua decision variants
- **WHEN** `make_decision()` returns a decision table
- **THEN** `{action="stop", reason?}` -> `Decision::Stop`; `{action="run", target_type="point", target={x, z, y?=0}, reason?}` -> `Run(Point)`; `{action="run", target_type="cell", target="M10", reason?}` -> `Run(GridCell)`; `{action="run", target_type="region", target={from="A1", to="C3"}, reason?}` -> `Run(Region)`; `{action="run", target_type="ball", reason?}` -> `Run(Ball)`; and `{action="kick", target={x, z, y?=0}, reason?}` -> `Decision::Kick`

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

### Requirement: Team and Player Script Authoring Rules (Layers 3 and 4)

Team-owned scripts — Layer 3 `teams/<id>/preamble.lua` (`team_play` default handlers and shared role tables like `goalkeeper_play`) and Layer 4 `teams/<id>/players/<NN>/script.lua` (optional partial `player_play` override) — SHALL obey 6 rules: (1) call only functions of the user-facing API (the user-facing Layer 1 functions and the public `stdlib.lua` API) and never access `context` or `GAME_DATA` directly, (2) never override `make_decision()`, (3) never hand-craft `{action=...}` tables, (4) never iterate over player collections, (5) never read raw coordinate fields (`.x`, `.y`, `.z`) of positions, and (6) prefer point-free handler assignment (`ball_is_free = chase_ball`).

#### Scenario: Partial `player_play` override falling back to `team_play`
- **WHEN** `preamble.lua` defines `team_play = { ball_is_free = chase_ball, team_has_ball = run_to_attack_position, opponent_has_ball = run_to_defence_position }` and a forward's `script.lua` defines `player_play = { i_have_ball = kick_to_opponent_goal }`
- **THEN** `make_decision()` invokes `player_play.i_have_ball` when that forward owns the ball and falls back to `team_play[state]` for the other three possession states

### Requirement: TeamRepository Contract and Filesystem Team Layout

`FsTeamRepository::new(base)` SHALL implement `TeamRepository::load_team(team_id) -> Result<TeamRecord, String>` by reading `<base>/<team_id>/preamble.lua`, reading optional `<base>/<team_id>/meta.toml` (`display_name: String`, defaulting `TeamRecord::display_name` to `team_id` when `meta.toml` is absent), and scanning subdirectories of `<base>/<team_id>/players/` sorted lexicographically by directory name (requiring `>= 1` player). Each player directory MUST contain `static.toml` and `tactical.toml` and MAY contain `script.lua` (`None` -> empty script inheriting `team_play`).

#### Scenario: Player TOML schemas, tactical squad numbers, and custom play regions
- **WHEN** `FsTeamRepository` loads a team directory
- **THEN** `meta.toml` (when present) provides `display_name: String` (or `TeamRecord::display_name` defaults to `team_id`), `static.toml` provides non-empty `name: String` and `u32` rates (`reaction_rate`, `speed_rate`, `tackle_rate`, `shot_power`, `shot_accuracy` in `10..=100`), and `tactical.toml` provides `number: u32` (`1..=N` contiguous tactical squad index, distinct from future cosmetic shirt numbers), optional `[play_positions]` (keys like `attack`/`defence` consumed only by scripts via `my_regions()`), and `[set_piece_positions]`

### Requirement: Script Testing Infrastructure (`ynwa-script-tests`)

`ynwa-script-tests` SHALL provide deterministic mini-game harnesses (forcing immediate decisions without waiting for `PlayerReactionSystem` timers) to verify `core.lua`, `stdlib.lua`, `make_decision()` dispatch, and `Team::A`/`Team::B` script symmetry, alongside self-contained team sets (`scenarios/<name>/teams/`) for visual verification in `ynwa-player`.

#### Scenario: Timer-bypassed decision assertions and visual scenario sets
- **WHEN** script integration tests run or a developer inspects a tactical scenario visually
- **THEN** automated tests evaluate player scripts deterministically in a single `DecisionSystem` step without reaction-timer delays, and `ynwa-script-tests/scenarios/<name>/teams/` directories load directly into `ynwa-player` for visual match/replay review
