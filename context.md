# YNWA - Football Manager

## Project Description

A football manager where the player sets characteristics and instructions for their team's players, launches the game, and observes its flow, adjusting instructions in real-time. The main focus is observing the game flow.

## Architecture

### Modularity

The project is divided into independent modules using Rust workspace:

- **Core (`ynwa-core`)** - a library that simulates the game
  - Knows only game simulation logic
  - Receives all parameters (characteristics, commands, instructions) from outside via API
  - Does not depend on specific client implementations
  - Uses `ynwa-decisions` for Lua scripting support
  - No external physics engine - custom physics in PhysicsSystem
  
- **Decision Engine (`ynwa-decisions`)** - game-agnostic decision-making library
  - Independent crate with Lua scripting support
  - No dependencies on game domain types (no GridCell, Region, Point3D)
  - JSON-based contract: config → context → decision
  - Can be published separately and reused in other games
  
- **Scripts (`ynwa-scripts`)** - Lua scripts library (data only, no Rust code)
  - `preambles/` - core.lua (elementary functions), stdlib.lua (utilities)
  - `test-scripts/` - integration test scripts
  - Three-level preamble system: core → stdlib → team → user script
  - See `ynwa-scripts/context.md` for full scripting API documentation
  
- **Clients** - applications using the core:
  - `ynwa-player` - local client, simulates the game locally and interacts with the player
  - Game server (future) - simulates multiple games, transmits data over network
  - `ynwa-simulator` - local client, simulates the game locally and write the game to the file
  
- **Test suites:**
  - `ynwa-script-tests` - integration tests for Lua scripts
  - Verifies that scripts produce correct decisions through the full system pipeline

### Universality (optional requirement)

Goal - ability to use the core for other team sports (American football, basketball, hockey).

Common traits of target games:

- Team game on a playing field
- Game object must be placed into a goal

### Deferred Aspects

The following aspects are considered in the design but implementation is postponed:
- Data storage logic
- Network play
- Server architecture for multiple games

### Implemented Components

**Game API (`game.rs`):**
- Poll-based model: client owns the game loop
- API design: `state()` provides access to state
- Determinism through fixed timestep (controlled by client)

**Entity Model:**
- Separation of Config (immutable) / State (mutable per-frame)
- Entities: Player, Ball, Referee — separate types (not traits), as they are processed by different systems
- Indices: `config.players[i]` ↔ `state.player_states[i]` — O(1) access

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

**Football Module (`football/mod.rs`):**
- Main API for creating football world: `create_football_world_from_file()`
- GameConfig creation functions are made private - clients work directly with World
- Design decision: field is created inside the football module, external code has no direct access to field creation
- Clients use ready-made world creation functions rather than manually constructing Game

**Game Systems:**
System execution order (important for correct operation):
1. **FootballGameManager** - manages game stage transitions (Setup → Play), manages football-specific game logic for determining events (future)
2. **PlayerReactionSystem** - determines when player is ready to accept new decision based on reaction_rate. During Setup stage: requests a decision once (when player has none); arrival is handled by DecisionSystem, not by re-polling.
3. **BallPossessionSystem** - determines which player possesses the ball (see Ball Possession System section)
4. **DecisionSystem** - creates decisions (Decision) for players using DecisionMaker trait. During Setup stage: on every tick checks if the player has reached their Run target (within 0.5m); if so, overrides the decision with Stop without calling the script.
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
- `GAME_DATA` (static, set once per player): `zones` (field zones pre-transformed for the player's team perspective), `field.width`, `field.length`
- See module `//!` doc for JSON contract and Lua script return format

**Decision Engine Library (`ynwa-decisions` crate):**
- Game-agnostic Lua scripting, JSON in/out, no domain types
- Can be published separately and reused in other games
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

1. **Only non-obvious:** Don't duplicate information from function/variable/constant names
2. **Design decisions:** Explain "why" decisions were made, not "what" the code does
3. **Public API:** Docstrings for public functions with purpose description
4. **Context.md:** Update when adding new components - record only design decisions and architectural solutions that cannot be extracted from code

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

## Orientation System (`orientation.rs`)

Coordinate transformations between team perspectives.
See `orientation.rs` `//!` doc for concept, functions, and usage.

## Region System (`region.rs`)

Grid-based field area addressing. Format: `"A1:B2"` (TopLeft:BottomRight), 1-based columns (A=1...).
See `region.rs` `//!` doc for types and indexing details.

Construction API:
- `GridDimensions::create_region(top_left, bottom_right)` — validated factory; use for user-supplied coordinates
- `Region::new(top_left, bottom_right)` — no validation; use when coordinates are internally generated (flip results, single-cell from `GridCell` target)

## Game Configuration (`config.rs`)

TOML-based configuration for initial game parameters.

## Physics and Speed

Speed formula: `actual_speed = (speed_rate / 100.0) * 10.0 m/s` (max ~36 km/h at rate=100).
See `physics_util.rs` `//!` doc for coordinate types and utility functions.

## Ball Possession System

See `systems/ball_possession.rs` `//!` doc for parameters, possession logic, ball state fields, and design decisions.

## Game Stages System

See `football/game_manager.rs` `//!` doc for `GameStage` enum, stage behavior, and transition logic.

