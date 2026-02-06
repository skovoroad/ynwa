# ynwa-scripts: Lua Scripting API Documentation

## 1. Project Purpose

`ynwa-scripts` is a project for developing Lua libraries that control player behavior in a football simulator. The project contains:

- **Preambles** - function libraries available to all scripts
- **Test scripts** - test scripts for functionality verification
- **Team libraries** - libraries for team strategies (in development)

Goal: provide a set of reusable functions for writing AI players in Lua without requiring knowledge of the game engine internals.

## 2. Core Mechanisms: API Contract

### 2.1 Main Contract

Each player script must define a `make_decision()` function that:
- Receives an implicit `context` parameter (global variable) with game state
- Returns a Lua table with a decision

```lua
function make_decision()
    -- Analyze context and make decision here
    return {action = "stop"}
end
```

### 2.2 Input Data: `context` Structure

The global variable `context` contains a JSON representation of the current game state:

```lua
context = {
    -- Current player information
    me = {
        team = "A",           -- Player's team: "A" or "B"
        number = 10,          -- Player number in the team (jersey number, 1-99)
        index = 5,            -- Global player index (0-21, position in players array)
        position = {
            x = 15.5,         -- X coordinate (meters)
            y = 0.0,          -- Height (meters)
            z = 25.0          -- Z coordinate (meters)
        }
    },
    
    -- Array of teammate positions
    teammates = {
        {
            index = 0,        -- Global player index
            number = 1,       -- Player number
            position = {x = 10.0, y = 0.0, z = 20.0}
        },
        {
            index = 2,
            number = 3,
            position = {x = 20.0, y = 0.0, z = 30.0}
        }
        -- ... other teammates (total 10, excluding current player)
    },
    
    -- Array of opponent positions
    opponents = {
        {
            index = 11,       -- Global player index
            number = 1,       -- Opponent number
            position = {x = 50.0, y = 0.0, z = 20.0}
        }
        -- ... other opponents (total 11)
    },
    
    -- Ball position
    ball = {
        position = {
            x = 52.5,         -- Field center by default
            y = 0.0,
            z = 34.0
        },
        owner_index = 5       -- Global player index possessing the ball, or null if ball is free
    },
    
    -- Game time
    game = {
        elapsed_time = 125.5  -- Seconds since game start
    }
}
```

#### Important Coordinate Details:

1. **Field Orientation**: Coordinates are always presented from the player's team perspective
   - Team A: attacks towards increasing X
   - Team B: sees the field mirrored (core automatically transforms coordinates)

2. **Field Dimensions** (standard football):
   - Length (X): 105 meters
   - Width (Z): 68 meters
   - Field center: (52.5, 0, 34)

3. **Coordinate System**:
   - X: along the field (length)
   - Y: height above field
   - Z: across the field (width)

#### Player Identification: Two Methods

There are two ways to identify players in the game:

1. **Global Index (`index`)** - Technical identifier (0-21)
   - Used internally by the game engine
   - Position in the players array (Team A: 0-10, Team B: 11-21)
   - **Use for:** Comparing with `context.ball.owner_index`, matching with `teammates[i].index`
   - **Example:** Check if I own the ball: `context.me.index == context.ball.owner_index`

2. **Team + Number (`team`, `number`)** - Domain identifier
   - `team`: "A" or "B"
   - `number`: Jersey number (1-99, arbitrary, can be non-unique)
   - **Use for:** Human-readable player identification, game configuration
   - **Example:** "Player #10 from Team A"

**Important:** 
- For internal comparisons (ball ownership, finding specific player) → use `index`
- For display and configuration → use `team` + `number`
- `context.ball.owner_index` contains global index, not number

### 2.3 Output Data: Decision Formats

The `make_decision()` function must return a Lua table in one of the following formats:

#### 2.3.1 Stop

```lua
return {action = "stop"}
```

Player stops and remains at current position.

#### 2.3.2 Run to Point

```lua
return {
    action = "run",
    target_type = "point",
    target = {
        x = 30.0,    -- X coordinate (meters)
        z = 40.0,    -- Z coordinate (meters)
        y = 0.0      -- Height (optional, default 0)
    }
}
```

Player runs to the specified point with given coordinates.

#### 2.3.3 Run to Cell

```lua
return {
    action = "run",
    target_type = "cell",
    target = "C7"    -- String with cell designation
}
```

Player runs to the center of the specified field grid cell.

**Cell Format**: `[COLUMN][ROW]`
- Columns: letters A, B, C, ..., Z, AA, AB, ... (like Excel)
- Rows: numbers 1, 2, 3, ...
- Examples: `"A1"`, `"B5"`, `"Z10"`, `"AA1"`
- Case insensitive: `"a1"` = `"A1"`

**Standard football field grid**: 21 columns (A-U) × 13 rows
- Cell size: ~5m × ~5m
- Central cell: K7

#### 2.3.4 Run to Region

```lua
return {
    action = "run",
    target_type = "region",
    target = {
        from = "A5",    -- Starting cell of region
        to = "C7"       -- Ending cell of region
    }
}
```

Player runs to the center of a rectangular region defined by two corner cells.

**Region Format**: two cells define opposite corners of a rectangle
- `from` and `to` can be specified in any order
- Region center is calculated automatically

#### 2.3.5 Kick

```lua
return {
    action = "kick",
    target = {
        x = 75.0,    -- X coordinate to kick towards (meters)
        z = 30.0,    -- Z coordinate to kick towards (meters)
        y = 0.0      -- Height (optional, default 0)
    }
}
```

Player kicks the ball towards the specified point.

**Notes**:
- Target specifies direction
- Ball physics determines actual trajectory

#### 2.3.6 Decision Validation

The core validates:
- Presence of required `action` field
- Correctness of `target_type` for `run` action
- `target` format depending on `target_type` or `action`
- Existence of cells within field grid bounds

On error:
- Decision is rejected
- Player receives "stop" decision
- Error is logged (in development mode)

## 3. Preamble Structure (Three-Level System)

Before executing user script, three preamble levels are loaded in the following order:

### 3.1 Core Preamble

**File**: `ynwa-scripts/preambles/core.lua`

**Purpose**: Elementary functions for reading game state and creating decisions.

**Responsibilities**:
- Parsing global `context` variable
- Helper functions for accessing game data
- Factories for creating correct decision objects
- Minimal utilities (no business logic)

**Example Functions** (planned):
```lua
-- Get own position
function my_position()
    return context.me.position
end

-- Get ball position
function ball_position()
    return context.ball.position
end

-- Create "stop" decision
function stop()
    return {action = "stop"}
end

-- Create "run to point" decision
function run_to_point(x, z, y)
    return {
        action = "run",
        target_type = "point",
        target = {x = x, z = z, y = y or 0}
    }
end

-- Create "run to cell" decision
function run_to_cell(cell)
    return {
        action = "run",
        target_type = "cell",
        target = cell
    }
end
```

**Rule**: Core preamble should not depend on game specifics (football, hockey, etc.)

### 3.2 Stdlib Preamble (Standard Library)

**File**: `ynwa-scripts/preambles/stdlib.lua`

**Purpose**: Common utilities used by all games and teams.

**Responsibilities**:
- Geometric functions (distances, vectors, angles)
- Search algorithms (nearest player, nearest opponent)
- Mathematical utilities
- Functions for working with regions and grids
- General tactical functions

**Example Functions** (planned):
```lua
-- Calculate distance between two points
function distance(pos1, pos2)
    local dx = pos1.x - pos2.x
    local dz = pos1.z - pos2.z
    return math.sqrt(dx*dx + dz*dz)
end

-- Find nearest teammate
function nearest_teammate()
    local my_pos = my_position()
    local nearest = nil
    local min_dist = math.huge
    
    for _, tm in ipairs(context.teammates) do
        local dist = distance(my_pos, tm.position)
        if dist < min_dist then
            min_dist = dist
            nearest = tm
        end
    end
    
    return nearest, min_dist
end

-- Check if I'm closest to ball
function am_i_closest_to_ball()
    local ball_pos = ball_position()
    local my_dist = distance(my_position(), ball_pos)
    
    for _, tm in ipairs(context.teammates) do
        if distance(tm.position, ball_pos) < my_dist then
            return false
        end
    end
    
    return true
end
```

**Rule**: Stdlib contains no team strategies, only reusable utilities.

### 3.3 Team Preamble

**Files**: Not yet (defined in TOML config, possibly in `ynwa-scripts/team-libs/` in the future)

**Purpose**: Functions specific to a particular team's strategy.

**Responsibilities**:
- Team tactics (formation, zones of responsibility)
- Role functions (defender, forward, goalkeeper)
- Coordination between players of the same team
- Team play style

**Example Functions** (planned):
```lua
-- Determine my role in the team
function my_role()
    if context.me.number == 1 then
        return "goalkeeper"
    elseif context.me.number <= 5 then
        return "defender"
    elseif context.me.number <= 8 then
        return "midfielder"
    else
        return "forward"
    end
end

-- Get my zone of responsibility
function my_defensive_zone()
    local role = my_role()
    if role == "goalkeeper" then
        return {from = "A6", to = "A8"}
    elseif role == "defender" then
        return {from = "B1", to = "E13"}
    end
    -- ... etc.
end

-- Team strategy: should I chase the ball?
function should_i_chase_ball()
    if my_role() == "goalkeeper" then
        return false  -- Goalkeeper doesn't leave goal
    end
    
    if am_i_closest_to_ball() then
        return true
    end
    
    -- Additional team coordination logic
    return false
end
```

**Rule**: Team preamble can use functions from core and stdlib, but not vice versa. Stdlib can use core, but not vice versa.

### 3.4 Implemented Functions (Current Status)

**Core Preamble** (`preambles/core.lua`):

*Constants:*
- `FIELD_LENGTH`, `FIELD_WIDTH` - field dimensions in meters (TODO: move to config)

*Context Access:*
- `my_position()` - returns current player's position `{x, y, z}`
- `my_index()` - returns global player index (0-21)
- `my_number()` - returns jersey number (1-99)
- `ball_position()` - returns ball position `{x, y, z}`
- `ball_owner()` - returns global player index who owns ball, or nil if free
- `get_teammates()` - returns array of teammates (each has `index`, `number`, `position`)
- `get_opponents()` - returns array of opponents (each has `index`, `number`, `position`)

*Decision Factories:*
- `stop()` - creates "stop" decision
- `run_to_point(x, z, y)` - creates "run to point" decision (y optional, default 0)
- `run_to_random_position()` - creates "run to random position" decision
- `kick_to(x, z, y)` - creates "kick" decision (y optional, default 0)

**Stdlib Preamble** (`preambles/stdlib.lua`):

*Ball Ownership:*
- `am_i_ball_owner()` - returns true if current player owns the ball
- `is_ball_owned_by_my_team()` - returns true if ball owned by any teammate (or me)

*Geometric Utilities:*
- `distance(pos1, pos2)` - calculates 2D distance between two positions (ignoring Y)

*Search Functions:*
- `find_nearest_opponent()` - returns `{opponent = <opponent_data>, distance = <number>}` (or `{opponent = nil, distance = math.huge}`)
- `am_i_closest_teammate_to_ball()` - returns true if current player is closest teammate to ball

### 3.5 User Script

**Files**: In game configuration (TOML), `script` field for each player

**Purpose**: Individual behavior of a specific player.

**Responsibilities**:
- Implementation of `make_decision()` function
- Using functions from all three preamble levels
- Player-specific behavior

**Example**:
```lua
function make_decision()
    -- Use functions from core
    local ball_pos = ball_position()
    local my_pos = my_position()
    
    -- Use functions from stdlib
    if am_i_closest_to_ball() then
        -- Use functions from team preamble
        if should_i_chase_ball() then
            return run_to_point(ball_pos.x, ball_pos.z)
        end
    end
    
    -- Otherwise go to zone of responsibility
    local zone = my_defensive_zone()
    return run_to_region(zone.from, zone.to)
end
```

### 3.5 Loading Order and Isolation

**Concatenation Order**:
```
[Core Preamble] + [Stdlib Preamble] + [Team Preamble] + [User Script]
```

**Isolation**:
- Each player has an **isolated Lua VM** (virtual machine)
- Players cannot communicate directly through global variables
- State between `make_decision()` calls is **not preserved** (VM is reset)
- Functions from preambles are available on each call

**Sandboxing** (security):
- Dangerous functions disabled: `os.execute`, `io.open`, `require`, `dofile`, `loadfile`
- Available: math (`math`), strings (`string`), tables (`table`), basic functions

## 4. Isolated Development

### 4.1 What You DON'T Need to Know

When developing Lua libraries in `ynwa-scripts`, you **don't need** to know:
- How the game engine is structured (ynwa-core)
- How the coordinate system works internally (orientation.rs)
- How physics and systems are implemented (systems/)
- Serialization and deserialization details (config.rs)

### 4.2 What You Need to Know

It's sufficient to know:
1. `context` format (described in section 2.2)
2. Decision formats (described in section 2.3)
3. Preamble structure (described in section 3)

### 4.3 Development Workflow

1. **Editing Preambles**:
   - `preambles/core.lua` - for elementary functions
   - `preambles/stdlib.lua` - for utilities
   - TOML config - for team preambles

2. **Testing**:
   - `ynwa-script-tests` project contains integration tests
   - Tests run full game cycle with Lua scripts
   - Run: `cargo test --package ynwa-script-tests`

3. **Debugging**:
   - Use `print()` in Lua (output not yet implemented)
   - Return debug information in decisions (non-standard fields are ignored by core)
   - Check error logs for invalid decisions

### 4.4 Example Test Scripts

See files in `ynwa-scripts/test-scripts/`:
- `kick_if_ball_owner.lua` - kick ball randomly if I own it, otherwise stop

## 5. Testing Infrastructure (ynwa-script-tests)

### 5.1 Project Structure

**Directory Layout:**
- `src/lib.rs` - helper functions for creating test games
- `tests/` - integration tests:
  - `basic_scripts.rs` - tests decision format validation
  - `stdlib_functions.rs` - tests preamble function correctness
  - `team_orientation.rs` - tests coordinate transformation for Team B

### 5.2 Running Tests

```bash
# Run all script tests
cargo test --package ynwa-script-tests

# Run specific test file
cargo test --package ynwa-script-tests --test basic_scripts
cargo test --package ynwa-script-tests --test stdlib_functions
```

### 5.3 Helper Functions

**`create_test_game_with_script(script: &str)`**
- Creates minimal game with one player running the given script
- No preambles loaded - use only for testing raw decision formats

**`create_test_game_with_preambles(script: &str)`**
- Creates game with core and stdlib preambles loaded
- Use for testing preamble functions and realistic scripts

**`load_test_script(name: &str)`**
- Loads test script from `ynwa-scripts/test-scripts/` directory
- Returns script content as String

### 5.4 Writing New Tests

To test a preamble function:

```rust
use ynwa_script_tests::create_test_game_with_preambles;

#[test]
fn test_my_function() {
    let script = r#"
        function test_function()
            local result = my_function()
            if not result then
                error("my_function() failed")
            end
        end
        test_function()
        
        function make_decision()
            return {action = "stop"}
        end
    "#;
    
    let mut game = create_test_game_with_preambles(script);
    // ... trigger systems and check for errors
}
```

## 6. Future Extensions

### 6.1 Planned Contract Features

- Ball possession information
- Player and ball velocities
- Field zones - named regions
- Events - goals, fouls, etc.
- Game phases - attack, defense, set pieces

### 6.2 Planned Preamble Capabilities

- Finite State Machines (FSM) for players
- Behavior trees
- Group coordination utilities
- Trajectory prediction

### 6.3 Tooling

- Context visualizer (web interface for debugging)
- REPL for interactive Lua function testing
- Script performance profiler
- Library of ready-made tactics and strategies

---

## Useful Links

- Lua 5.4 Reference: https://www.lua.org/manual/5.4/
- mlua project (Rust-Lua integration): https://github.com/mlua-rs/mlua
- Field coordinate system: see `ynwa-core/src/orientation.rs` (if details needed)

## Support

When developing in `ynwa-scripts`:
1. Create issues for contract ambiguities
2. Propose new functions via PRs to preambles
3. Document all public functions in comments

**Principle**: Lua scripts should be readable and understandable without knowing Rust code.
