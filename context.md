# YNWA - Football Manager

## Project Description

A football manager where the player sets characteristics and instructions for their team's players, launches the game, and observes its flow, adjusting instructions in real-time. The main focus is observing the game flow.

## Architecture

### Modularity

The project is divided into independent modules using Rust workspace:

- **Core (`core`)** - a library that simulates the game
  - Knows only game simulation logic
  - Receives all parameters (characteristics, commands, instructions) from outside via API
  - Does not depend on specific client implementations
  
- **Clients** - applications using the core:
  - Local client - simulates the game locally and interacts with the player
  - Game server (future) - simulates multiple games, transmits data over network
  - Test applications

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
- API design: `step()` returns events, `state()` provides access to state
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
  - Returns list of GameEvent events
- Design decision: systems receive &mut Game instead of &mut World to avoid borrow checker issues during iteration over systems
- Design decision: systems receive absolute timestamp instead of delta_time so they can store last update time and calculate intervals themselves

**Football Module (`football/mod.rs`):**
- Main API for creating football world: `create_football_world()`, `create_football_world_from_file()`, `create_football_world_from_toml()`
- GameConfig creation functions are made private - clients work directly with World
- Design decision: field is created inside the football module, external code has no direct access to field creation
- Clients use ready-made world creation functions rather than manually constructing Game

**Game Systems:**
System execution order (important for correct operation):
1. **PlayerReactionSystem** - determines when player is ready to accept new decision based on reaction_rate
2. **DecisionSystem** - creates decisions (Decision) for players using DecisionMaker trait
3. **ActionSystem** - transforms decisions into velocity (applies speed_rate)
4. **PhysicsSystem** - applies velocity to position using kinematics: position += velocity × delta_time

**Player Decisions (Decision):**
- `Decision::Run(DecisionTarget)` - run to target
  - `DecisionTarget::Point(Point3D)` - specific point
  - `DecisionTarget::GridCell(GridCell)` - center of grid cell
  - `DecisionTarget::Region(Region)` - center of region
- `Decision::Stop` - stop
- Each decision is processed exactly once (decision_processed flag)

**DecisionMaker trait:**
- Public interface for creating AI players
- `make_decision(&mut self, game: &Game, player_index: usize) -> Decision`
- DecisionSystem::with_decision_maker() for dependency injection
- PlaceholderDecisionMaker - basic implementation (generates random run decisions to grid cells)

**Scripting System (`scripting/`):**
- Game-agnostic Lua-based scripting for decision making
- **LuaExecutor** - executes Lua scripts with context data
  - Uses mlua with vendored Lua 5.4
  - Serialization via serde (Rust structs → Lua tables)
  - Returns ScriptResult with JSON value for game-specific parsing
  - `execute(script, function_name, context)` - loads script and calls specified function
  - State behavior: script code reloads on each execute(), preamble persists
  - Preamble: optional Lua code string injected once at creation
  - **Sandbox:** dangerous libraries disabled (io, os, package, debug, loadfile, dofile)
- **ScriptError** - structured error handling (SyntaxError, RuntimeError, SerializationError, DeserializationError, FunctionNotFound)
- User scripts contract:
  - Must implement function with specified name (e.g., `make_decision()`)
  - Access context via global `context` variable
  - Return Lua table (structure depends on game)
- Design decisions:
  - Lua chosen over Python for better embedding support (control, isolation, performance)
  - serde used instead of JSON for direct Rust ↔ Lua table conversion (faster, type-safe)
  - ScriptResult uses JSON internally to maintain game-agnostic design
  - Function name passed as parameter for flexibility (can call different functions from same script)
  - Preamble as plain string (no builder) - game integration decides how to construct it
  - Sandbox hardcoded (no configuration) - security by default, simplifies API
  - Timeout deferred - can be added later via mlua interrupt hooks when needed
- Test coverage: 33 unit tests covering all error cases, state behavior, data types, and sandbox

## TODO

- [ ] Render game entities: players, ball, referees
- [ ] Integrate Lua scripting with DecisionMaker trait
- [ ] Add timeout control for script execution (via mlua interrupt hooks when needed)
- [ ] Add memory limits for scripts

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
- **Physics engine:** `rapier3d` - 3D physics for movements, distance calculations, collisions, ball flight
- **Scripting (future):**
  - Lua - via `mlua`
  - Python - via `PyO3`
  - Scripts receive and return data structures from core

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
- Deterministic physics (Rapier mode) for reproducible tests

## Region System (region.rs)

**Purpose:** Addressing field areas through a grid coordinate system.

**Key types:**
- `GridDimensions { columns: u32, rows: u32 }` - grid dimensions
- `GridCell { col: u32, row: u32 }` - one cell (1-based indexing)
- `Region { team: Team, top_left: GridCell, bottom_right: GridCell }` - rectangular area

**Indexing:** 1-based for columns and rows
- Columns: A=1, B=2, ..., Z=26, AA=27, AB=28, ...
- Rows: 1, 2, 3, ...

**Grid notation:** Human-readable format for regions
- Format: "A1:B2" (TopLeft:BottomRight)

## Game Configuration (config.rs)

**Purpose:** TOML-based configuration system for initial game parameters.

## Physics and Speed

**Coordinate and velocity types:**
- `Point3D { x: Length, y: Length, z: Length }` - position in meters
- `Velocity3D { x: Velocity, y: Velocity, z: Velocity }` - velocity in m/s
- Using `uom` for type safety of physical quantities

**Physics utilities (physics_util.rs):**
- `distance(a: &Point3D, b: &Point3D) -> f32` - calculates Euclidean distance between points in meters
- `distance_length(a: &Point3D, b: &Point3D) -> Length` - same but returns Length for type safety

**Player speed:**
- `speed_rate`: 10-100 (player configuration)
- `MAX_SPEED_METERS_PER_SECOND = 10.0` (~36 km/h at speed_rate=100)
- Formula: `actual_speed = (speed_rate / 100.0) * MAX_SPEED_METERS_PER_SECOND`
- Linear dependency on `speed_rate`

