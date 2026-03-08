# YNWA - Football Manager

## Project Description

A football manager where the player sets characteristics and instructions for their team's players, launches the game, and observes its flow, adjusting instructions in real-time. The main focus is observing the game flow.

## Architecture

### Modularity

The project is divided into independent modules using Rust workspace:

- **Core (`ynwa-core`)** - a library that simulates the game
  - Knows only game simulation logic
  - Receives all parameters (characteristics, commands, instructions) from outside via API
  - Does not depend on specific client implementations or sport-specific modules
  - Uses `ynwa-decisions` for Lua scripting support
  - No external physics engine - custom physics in PhysicsSystem
  
- **Football (`ynwa-football`)** - football-specific rules and world factory
  - Depends on `ynwa-core`; `ynwa-core` does NOT depend on this crate (guaranteed by Cargo's cycle detection)
  - `field_builder` - standard football field with FIFA regulation zones
  - `game_manager` - `FootballGameManager` system: stage transitions (Setup → Play), player readiness, event handling
  - `events` - goal detection, out-of-bounds (touchline/goal line), game end
  - Entry point: `create_football_world(repo, preambles_path)` — creates a ready-to-use `World`
  - Design decision: keeping sport rules separate from simulation core enables reuse of `ynwa-core` for other sports

- **Decision Engine (`ynwa-decisions`)** - game-agnostic decision-making library
  - Independent crate with Lua scripting support
  - No dependencies on game domain types (no GridCell, Region, Point3D)
  - JSON-based contract: config → context → decision
  - Can be published separately and reused in other games
  
- **Scripts (`ynwa-scripts`)** - Lua scripts library (data only, no Rust code)
  - `preambles/` - core.lua (elementary functions), stdlib.lua (utilities + dispatch)
  - Three-level preamble system: core → stdlib → team → user script
  - **Dispatch model**: `make_decision()` and `get_setup_position(reason)` are defined in stdlib and dispatch to `team_play`/`team_setup` (team preamble) or `player_play`/`player_setup` (player script). Player tables override team tables. Player script (`script.lua`) is optional per player; if absent, player uses team tactics entirely.
  - See `ynwa-scripts/context.md` for full scripting API documentation
  
- **Repository (`ynwa-repository`)** - filesystem implementation of `TeamRepository` trait from `ynwa-core`
  - Isolates storage concerns: `ynwa-core` depends only on the trait; this crate can be replaced with a DB-backed implementation without touching the rest of the codebase
  - `FsTeamRepository::new(base_path)` — loads team data from `<base>/<team_id>/`
  - Reads `preamble.lua`, `players/NN/{static.toml, tactical.toml, script.lua}`
  - `script.lua` is optional per player
  - `ynwa-football` depends only on the `TeamRepository` trait (from `ynwa-core`), not on this crate. `ynwa-player` creates `FsTeamRepository` and passes it to `create_football_world()`

- **Clients** - applications using the core:
  - `ynwa-player` - local client, depends on `ynwa-core` + `ynwa-football` + `ynwa-repository`, simulates the game locally and interacts with the player. Creates `FsTeamRepository` and passes it to `create_football_world()`. Default paths: `teams/` and `ynwa-scripts/preambles/`; overridable via CLI: `ynwa-player [teams_path] [preambles_path]`.
  - Game server (future) - simulates multiple games, transmits data over network
  - `ynwa-simulator` - local client, simulates the game locally and write the game to the file
  
- **Test suites:**
  - `ynwa-script-tests` - integration tests for Lua scripts, depends on `ynwa-core` + `ynwa-football`
  - Verifies that scripts produce correct decisions through the full system pipeline
  - `fixtures/team_a.lua`, `fixtures/team_b.lua` — minimal dispatch tables for tests (no real game tactics)
  - `fixtures/dispatch_spy.lua` — reusable spy script for dispatch testing; loaded via `load_test_script()`

- **Visual test scenarios (`ynwa-script-tests/scenarios/`):**
  - Minimal two-player team configurations for visual verification of specific game situations
  - Each scenario is a self-contained `teams/` directory that replaces the standard `teams/` path
  - Launched via `./run_scenario.sh <scenario_name>` (root-level script) — passes `ynwa-script-tests/scenarios/<name>/teams` and `ynwa-scripts/preambles` to `ynwa-player`
  - Current scenarios:
    - `goal_kick_teamA_left` — Team A player runs down left flank and kicks past goal line → verifies goal_kick Setup restart: Team B (restarting) walks to ball; Team A retreats 25m from center

### Universality (optional requirement)

Goal - ability to use the core for other team sports (American football, basketball, hockey).

Common traits of target games:

- Team game on a playing field
- Game object must be placed into a goal

### Team Repository (`teams/`)

Top-level directory containing team data in a structured format. Each team occupies its own subdirectory:

```
teams/
  team_a/
    meta.toml        # team display name
    preamble.lua     # team tactics (dispatch tables)
    players/
      NN/            # player directory, zero-padded number
        static.toml    # immutable attributes: name, reaction_rate, speed_rate, tackle_rate, shot_power, shot_accuracy
        tactical.toml  # tactical attributes: number; [play_positions]: attack, defence; [set_piece_positions]: 16 mandatory keys ("kick off own/opp", "goal kick own/opp", "corner own/opp left/right", "throw in own/opp left/right own/opp half")
        script.lua     # optional player script: player_play / player_setup overrides
```

Read access is intended to go through a `TeamRepository` trait (`ynwa-core/src/repository.rs`), allowing the filesystem implementation to be replaced with a database backend without changing the rest of the codebase.

**Player number semantics**: `tactical.toml` field `number` is the tactical number (1–N, contiguous within the team). Individual jersey numbers are a separate concept and will be introduced when players are decoupled from tactics.

**Status**: data files in `teams/`; `TeamRepository` trait in `ynwa-core/src/repository.rs`; `FsTeamRepository` in `ynwa-repository`; integrated into `ynwa-football` and `ynwa-player`.

### Deferred Aspects

The following aspects are considered in the design but implementation is postponed:
- Data storage logic
- Network play
- Server architecture for multiple games

### Implemented Components

**Game API (`game.rs`):**
- Poll-based model: client owns the game loop
- API design: `state()` provides access to state
- `GameState` has `restart_position: Option<Point3D>` and `restart_team: Option<Team>` — set by `FootballGameManager::handle_event` on each Setup transition; used by scripting layer and ball placement in Setup tick
- When `restart_position` is `Some`, `ScriptedDecisionMaker` adds `setup_info` to `context.game` in the Setup branch: `{ restart_x, restart_z, restarting_team }`. Coordinates are transformed for the player's team perspective (Team B sees flipped). Field is absent when `restart_position` is `None` (e.g. `"kick off"`).
- `stdlib.lua` provides `get_restart_position()` → `{x, z}` or `nil`, `is_my_team_restarting()` → `bool` or `nil`, and `run_to_restart_position()` → run action to the restart point or `nil` — wrappers over `setup_info` that follow the no-direct-`context`-access rule.
- `stdlib.lua` provides `default_goal_kick_setup()` — reusable handler for `team_setup.goal_kick`: restarting team goes to `"goal kick own"` region (fallback: `run_to_start_position()`); defending team goes to `"goal kick opp"` region (fallback: `run_to_defence_position()`). Team preambles reference it directly: `goal_kick = default_goal_kick_setup`.
- `ynwa-football` exposes `SET_PIECE_KEYS` (16 mandatory set-piece keys every player must declare) and `ON_BALL_REQUIRED_KEYS` (8 own-keys where exactly one player must have `"on_ball"`). `create_football_world` validates both via `validate_set_piece_keys` and `validate_on_ball` before building the world.
- Determinism through fixed timestep (controlled by client)

**Statistics (`StatSet` in `game.rs`):**
- `StatSet` — named `f64` counters, game-specific keys (e.g. `"score"`)
- `GameState::team_stats: HashMap<Team, StatSet>` — per-team stats
- `GameState::player_stats: Vec<StatSet>` — per-player stats, parallel to `player_states`
- Populated by game-specific managers (e.g. `FootballGameManager`), not by core systems
- Not exposed to the decision engine (Lua scripts have no access to stats)

**Entity Model:**
- Separation of Config (immutable) / State (mutable per-frame)
- Entities: Player, Ball, Referee — separate types (not traits), as they are processed by different systems
- Indices: `config.players[i]` ↔ `state.player_states[i]` — O(1) access
- `PlayerDef::new(team, number, name, script, regions: HashMap<String, Region>)` — the last argument is a map of named regions; game-specific callers (e.g. `ynwa-football`) populate it; core only reads the key `REGION_START_POSITION` (`"start"`) to place the player at game start. `REGION_START_POSITION` is the contract between core and game-specific layers — core does not know any other region names. In `ynwa-football`, `REGION_START_POSITION` is aliased from `"kick off opp"` in `tactical.toml` (the positional region when the opponent kicks off, used as generic start placement until proper kick-off team selection is implemented in task 2.x).
- `PlayerDef::set_piece_roles: HashSet<String>` — set-piece types this player is the designated taker for (e.g. `"goal kick own"`). Populated by `ynwa-football` when a player has `"on_ball"` as the value in `set_piece_positions`. Core does not interpret this field.

**World & Systems (`world.rs`, `system.rs`):**
- World coordinates the game loop, contains Game and a list of systems
- System trait: `update(&mut self, game: &mut Game, timestamp: f32)` - common interface for all game systems
- Systems execute sequentially in the order they are added
- **World::step(delta_time)** - public API for running one game tick:
  - Calculates new timestamp = elapsed_time + delta_time
  - Calls update() for all systems with new timestamp
  - Updates game.state.elapsed_time
- Design decision: systems receive &mut Game instead of &mut World to avoid borrow checker issues during iteration over systems
- Design decision: systems receive absolute timestamp instead of delta_time so they can store last update time and calculate intervals themselves

**Football Crate (`ynwa-football`):**
- Main API for creating football world: `create_football_world(repo: &dyn TeamRepository, preambles_path)`
- `ynwa-football` does not depend on `ynwa-repository` — caller injects the repository implementation
- `preambles_path` points to the directory containing `core.lua` and `stdlib.lua` (default: `ynwa-scripts/preambles/`)
- GameConfig creation functions are private - clients work directly with World
- Design decision: field is created inside `ynwa-football`, external code has no direct access to field creation
- Key constants exposed as `pub(crate)` for use in tests: `GOAL_WIDTH`, `GOAL_DEPTH`, `FIELD_WIDTH` (field_builder), `BALL_RADIUS`, `GAME_DURATION` (events)
- Test fields use standard production orientation: Team A goal at z < 0, Team B goal at z > field_length
- Clients use ready-made world creation functions rather than manually constructing Game
- Design decision: extracted from `ynwa-core` so the core stays sport-agnostic; other sports would provide their own equivalent crate

**Game Systems:**
System execution order (important for correct operation):
1. **FootballGameManager** (`ynwa-football`) - manages game stage transitions (Setup → Play), manages football-specific game logic for determining events. Players are marked ready when their `current_decision` is `Stop` (arrival detected by DecisionSystem); game transitions to Play once all players are ready.
2. **PlayerReactionSystem** - determines when player is ready to accept new decision based on reaction_rate. During Setup stage: sets `needs_decision` when player has no decision yet, suppresses it otherwise (early filter; DecisionSystem is the final guard for arrived players). During Play: fires when reaction interval elapsed.
3. **BallPossessionSystem** - determines which player possesses the ball (see Ball Possession System section). Skipped entirely during Setup stage (ball is fixed, possession is meaningless).
4. **DecisionSystem** - creates decisions (Decision) for players using DecisionMaker trait. During Setup stage: (a) on every tick checks if the player has reached their Run target (within 0.5m); if so, overrides the decision with Stop without calling the script; (b) if the player's decision is already Stop, suppresses any re-poll regardless of `needs_decision`.
5. **ActionSystem** - transforms decisions into velocity (applies speed_rate)
6. **PhysicsSystem** - applies velocity to position using kinematics: position += velocity × delta_time

**Player Decisions (Decision):**
- `Decision::Run(DecisionTarget)` - run to target
  - `DecisionTarget::Point(Point3D)` - specific point
  - `DecisionTarget::GridCell(GridCell)` - center of grid cell
  - `DecisionTarget::Region(Region)` - center of region
  - `DecisionTarget::Ball` - current ball position (resolved live each tick for arrival check; direction set once at decision processing time)
- `Decision::Stop` - stop
- `Decision::Kick(Point3D)` - kick ball towards target point (only if player possesses ball)
- Each decision is processed exactly once (decision_processed flag)

**DecisionMaker trait:**
- Public interface for creating AI players
- `make_decision(&mut self, game: &Game, player_index: usize) -> Result<Decision, DecisionError>`
- DecisionSystem::with_decision_maker() for dependency injection
- Implementations:
  - PlaceholderDecisionMaker - fallback (random movement)
  - ScriptedDecisionMaker - Lua script execution per player via ynwa-decisions
- **Design principle:** Decision system is independent - uses ynwa-decisions crate for Lua support

**Scripted Decision System (`systems/decision/scripted_decision_maker.rs`):**
- Adapter between ynwa-core domain types and ynwa-decisions JSON API
- One isolated Lua VM per player via `DecisionEngine`
- Team B coordinates flipped on input; decisions flipped back on output (parser does NOT flip)
- `GAME_DATA` (static, set once per player): `zones` (field zones pre-transformed for the player's team perspective), `field.width`, `field.length`, `field.columns`, `field.rows`
- See module `//!` doc for JSON contract and Lua script return format

**Decision Engine Library (`ynwa-decisions` crate):**
- Game-agnostic Lua scripting, JSON in/out, no domain types
- Can be published separately and reused in other games
- `LuaExecutor::execute(script, fn, context)` — calls Lua function with no positional args
- `LuaExecutor::execute_with_args(script, fn, context, args)` — calls Lua function with positional args; `execute` delegates to this with `()`
- `DecisionEngine::get_setup_position` extracts `setup_reason` from context and passes it as the first positional argument to the Lua `get_setup_position(reason)` function
- See `ynwa-decisions/src/lib.rs` `//!` doc for sandbox, timeout, and architecture details

## Development Principles

### Code

1. **Type safety:** Use `uom` for physical quantities instead of raw `f32`
2. **Validation:** Runtime checks in constructors via `assert!`
3. **Data-driven:** Description as data where possible, not algorithms
4. **Idiomatic Rust:** `Option<T>` instead of sentinel values, Result for errors
5. **Performance:** O(1) operations are critical for game engine
6. **YAGNI (You Aren't Gonna Need It):** Implement ONLY what is explicitly requested. User specifies functionality increment at each step explicitly. Don't add "for the future"
7. **Code quality:** Regularly run `cargo clippy`, `cargo fmt`, and other analyzers. Fix all warnings

### Testing

1. **Don't test trivial:** If test only checks value assignment - delete it
2. **Test logic:** Validation, edge cases, boundary conditions, integration
3. **Meaningful tests:** Check real system behavior, not obvious facts
4. **Unit tests mandatory:** When adding new functionality, always create unit tests verifying correct operation
5. **No examples and integration tests by default:** Don't add example applications (examples/) and integration tests without explicit user request. Focus on unit tests inside modules
6. **Separate test files:** Unit tests live in `*_tests.rs` files next to the implementation. Connect via `#[cfg(test)] #[path = "foo_tests.rs"] mod tests;`. This keeps implementation files focused and reduces AI context window usage.

### Documentation

1. **Only non-obvious:** Don't duplicate information from function/variable/variable/constant names
2. **Design decisions:** Explain "why" decisions were made, not "what" the code does
3. **Public API:** Docstrings for public functions with purpose description
4. **Context.md:** Update when adding new components - record only design decisions and architectural solutions that cannot be extracted from code

### Lua Scripting Rules

1. **No direct `context`/`GAME_DATA` access** outside `core.lua`: team preambles and player scripts must use wrapper functions (`my_position()`, `get_own_goal()`, etc.). Direct access to raw JSON couples scripts to engine internals and breaks portability between teams.
2. **No `make_decision()` / `get_setup_position()` redefinition** in team or player scripts — these are owned by stdlib.
3. **No raw action tables in tactic scripts**: use stdlib functions instead of constructing `{action = "..."}` tables directly. When a stdlib function exists (`stop()`, `chase_ball()`, `run_to_start_position()`, etc.) — use it.
4. **Prefer point-free style for dispatch tables**: assign function references directly instead of wrapping them in lambdas. Write `team_has_ball = chase_ball` instead of `team_has_ball = function() return chase_ball() end`.

### Code Comments

1. **Minimalism:** Code should speak for itself through clear names and structure
2. **Don't comment obvious:** Avoid comments for methods, variables, and parameters whose meaning is clear from the name
3. **Don't comment future:** Don't add comments about possible future extensions or TODOs without explicit request
4. **Design decisions only:** Comment only architectural decisions and non-obvious reasons for choosing an approach
5. **Brevity:** Comments should be as concise as possible

## Technical Requirements

### Language and Approach

- **Language:** Rust
- **Data Architecture:** Hybrid approach
  - ECS-like structures (Structure of Arrays) for critical data: positions, velocities, commands
  - Regular structures for other data
  - Custom implementation without external ECS libraries

### Technology Stack

#### Core
- **Scripting:** Lua via `mlua` (implemented in `ynwa-decisions` crate)
- **Physics:** Custom PhysicsSystem (kinematics: position += velocity × dt)
- **Math:** `uom` for type-safe physical quantities (Length, Velocity, etc.)

#### Local Client
- **UI:** `egui` - immediate mode GUI for controls (buttons, text fields, selectors)
- **Field rendering:** `macroquad` - 2D pixel retro graphics
- **Graphics style:** Pixel graphics with top-down view (2D rendering of 3D positions)

**Graphics feature:**
- Field displayed in 2D (top-down view)
- Physics works in 3D (ball height, jumps)
- Projection of 3D coordinates onto 2D plane for rendering

#### Platform Support
- Native platforms (Linux, Windows, macOS)
- WebAssembly - same code compiles for browser

### Testability

- High test coverage
- Modular architecture facilitates testing of individual components
- Deterministic physics for reproducible tests

## Field System (`field/`)

Y-up right-handed coordinate system: X=width, Y=height, Z=length (Team A → Team B).
See `field/mod.rs` `//!` doc for types, zone geometry, and design decisions.

**Standard football field dimensions:** 68m × 104.6m, 26×40 grid (cell ≈ 2.615m).
Aspect ratio 1:1.54 matches FIFA proportions (68:105). Columns A–Z (26) fit the English alphabet.
See `ynwa-football/src/field_builder.rs` for constants.

## Orientation System (`orientation.rs`)

Coordinate transformations between team perspectives.
See `orientation.rs` `//!` doc for concept, functions, and usage.

## Region System (`region.rs`)

Grid-based field area addressing. Format: `"A1:B2"` (TopLeft:BottomRight), 1-based columns (A=1...).
See `region.rs` `//!` doc for types and indexing details.

Construction API:
- `GridDimensions::create_region(top_left, bottom_right)` — validated factory; use for user-supplied coordinates
- `Region::new(top_left, bottom_right)` — no validation; use when coordinates are internally generated (flip results, single-cell from `GridCell` target)

## Physics and Speed

Speed formula: `actual_speed = (speed_rate / 100.0) * 10.0 m/s` (max ~36 km/h at rate=100).
See `physics_util.rs` `//!` doc for coordinate types and utility functions.

## Ball Possession System

See `systems/ball_possession.rs` `//!` doc for parameters, possession logic, ball state fields, and design decisions.

## Game Stages System

See `ynwa-football/src/game_manager.rs` `//!` doc for `GameStage` enum, stage behavior, and transition logic.

