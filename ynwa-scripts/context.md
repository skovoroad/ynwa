# ynwa-scripts: Lua Scripting API Documentation

## 1. Project Purpose

`ynwa-scripts` is a project for developing Lua libraries that control player behavior in a football simulator. The project contains:

- **Preambles** - function libraries available to all scripts
- **Test scripts** - test scripts for functionality verification
- **Team libraries** - libraries for team strategies (in development)

Goal: provide a set of reusable functions for writing AI players in Lua without requiring knowledge of the game engine internals.

## 1.1 Quick Start Summary

**What you need to know**:
1. **Input**: Global `context` variable with game state (positions, ball, regions)
2. **Output**: Return decision table from `make_decision()` function
3. **Grid system**: Field divided into cells (A1, B2, ..., AA1, ...)
4. **Regions**: Named rectangular areas assigned to each player (`context.me.regions`)
5. **Preambles**: Three-level library system (core → stdlib → team → your script)

**Coordinate system**:
- X axis: field length 
- Y axis: height above field
- Z axis: field width 
- Team B sees flipped coordinates automatically

**Grid notation**:
- Columns: A=1, B=2, ..., Z=26, AA=27, ...
- Rows: 1, 2, 3, ...
- Example: `"K7"` is center cell, `"A1:B2"` is region from A1 to B2

**See section 7 (end of document) for quick reference tables**.

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

Each player script may define a `get_setup_position(reason)` function that:
- Receives `reason` string (e.g. `"start"`, `"after_goal"`) via `context.game.setup_reason`
- Returns a `"run"` decision towards the player's start position
- **Stopping is handled by the engine automatically** — once the player is within 0.5m of the target, the engine replaces the decision with Stop; the script does not need to check distance

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
        },
        regions = {
            -- Named regions assigned to this player (e.g., start position, zone of responsibility)
            -- Coordinates are already transformed for Team B (see section 2.2.2)
            ["start position"] = {
                min_x = 10.0,     -- Minimum X coordinate of region (meters)
                max_x = 15.0,     -- Maximum X coordinate of region (meters)
                min_z = 20.0,     -- Minimum Z coordinate of region (meters)
                max_z = 25.0      -- Maximum Z coordinate of region (meters)
            }
            -- Can have multiple named regions
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
        owner_index = 5,      -- Global player index possessing the ball, or null if ball is free
        owner_team = "A"      -- Team that last possessed the ball: "A", "B", or "None"
                              -- Persists during passes (when owner_index is null)
                              -- "None" at game start and after Setup stages
    },
    
    -- Game time
    game = {
        elapsed_time = 125.5,  -- Seconds since game start
        setup_reason = "start" -- Present only during Setup stage: reason for setup
                               -- Values: "start", "after_goal", "throw_in", "set_piece"
    }
}
```

#### Important Coordinate Details:

1. **Field Orientation**: Coordinates are always presented from the player's team perspective
   - Team A: attacks towards increasing X (sees field in "canonical" orientation)
   - Team B: sees the field mirrored - core automatically transforms coordinates
   - **Transformation formula for Team B**: 
     - `x' = field_width - x` (flip along width)
     - `z' = field_length - z` (flip along length)
     - `y' = y` (height unchanged)
   - This transformation is **automatic** - scripts don't need to handle it manually

2. **Coordinate System**:
   - X: along the field (length)
   - Y: height above field
   - Z: across the field (width)

#### Player Identification: Two Methods

There is a way to identify players in the game:

**Global Index (`index`)** - Technical identifier (0-21)
 - Used internally by the game engine
 - Position in the players array (Team A: 0-10, Team B: 11-21)
 - **Use for:** Comparing with `context.ball.owner_index`, matching with `teammates[i].index`
 - **Example:** Check if I own the ball: `context.me.index == context.ball.owner_index`

**Important:** 
- `context.ball.owner_index` contains global index
- To check team ownership during passes, use `context.ball.owner_team`
- this low level info must be handled with functions in core preample. High level code must use this functions, not directly low-level abstarctions
  
#### 2.2.1 Player Regions (`context.me.regions`)

**What are regions?**

Regions in `context.me.regions` are named rectangular areas assigned to the current player. Each region has exact metric boundaries calculated from grid cell notation.

**Structure**:
```lua
context.me.regions = {
    ["region_name"] = {
        min_x = 10.0,   -- meters
        max_x = 15.0,   -- meters
        min_z = 20.0,   -- meters
        max_z = 25.0    -- meters
    },
    ["another_region"] = { ... }
}
```

**Key points**:
1. **Coordinate perspective**: Regions are already transformed for Team B's perspective
   - Team A sees regions in canonical coordinates
   - Team B sees regions flipped (same transformation as positions)
   - Scripts work identically for both teams

2. **Metric coordinates**: Boundaries are in meters, not grid cells
   - Calculated from grid notation in config (e.g., "A1:B2")
   - Use directly with `my_position()` for distance checks

3. **Access via `my_regions()`**:
   ```lua
   local regions = my_regions()
   local start = regions["start position"]
   if start then
       local center_x = (start.min_x + start.max_x) / 2
       local center_z = (start.min_z + start.max_z) / 2
   end
   ```

4. **Common uses**:
   - Check if player is in assigned zone: `is_point_in_rectangle(my_pos.x, my_pos.z, region)`
   - Navigate to region center during setup
   - Define zones of responsibility for tactics

See section 3.4.1 for detailed usage examples.

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

**Grid Notation System**:

The field is divided into a grid using Excel-style column naming:
- **Columns**: A, B, C, ..., Z, AA, AB, AC, ... (1-based indexing)
  - A = column 1
  - Z = column 26
  - AA = column 27 (26 + 1)
  - AB = column 28 (26 + 2)
  - Formula: For multi-letter columns, each letter contributes: position × 26^power
  - Example: "AB" = A×26¹ + B×26⁰ = 1×26 + 2×1 = 28

- **Rows**: 1, 2, 3, ... (1-based indexing)
  - Simple numeric indexing starting from 1

**Grid coordinate system orientation**:
- Columns (letters) run along the **Z axis** (field width)
- Rows (numbers) run along the **X axis** (field length)
- Cell "A1" is at the corner (min X, min Z) from the player's perspective
- Cell size calculation: `cell_width = field_width / num_columns`

**Examples**:
- `"A1"` → top-left corner (col=1, row=1)
- `"K7"` → center of standard football field (col=11, row=7)
- `"U13"` → bottom-right corner of standard field (col=21, row=13)
- `"AA1"` → column 27, row 1

**Important**: Grid cells are 1-based (not 0-based). Column A = 1, Row 1 = 1.

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

### 2.4 Static Game Data: `GAME_DATA`

The global variable `GAME_DATA` contains static information about the field that doesn't change during the game:

```lua
GAME_DATA = {
    field = {
        width = 60.0,   -- field width in meters (X axis)
        length = 101.5  -- field length in meters (Z axis)
    },
    zones = {
        -- Field zones with geometry (Team A coordinates)
        field = {
            type = "rectangle",
            min_x = 0.0, max_x = 101.5,
            min_z = 0.0, max_z = 60.0
        },
        penalty_area_a = { ... },
        center_circle = { ... },
        -- ... other zones
    }
}
```

**`GAME_DATA.field`**: actual field dimensions from the engine.

**Zone Types**:
- `rectangle`: defined by `min_x`, `max_x`, `min_z`, `max_z`
- `circle`: defined by `center_x`, `center_z`, `radius`
- `arc`: defined by `center_x`, `center_z`, `radius`, `start_angle`, `end_angle` (in degrees)
- `point`: defined by `x`, `z`, `tolerance`

**Zone Naming**:
- Team-specific zones have suffix `_a` or `_b` (e.g., `penalty_area_a`, `goal_b`)
- Neutral zones have no suffix (e.g., `field`, `center_circle`)

**Note**: `GAME_DATA` is initialized once when DecisionEngine is created and remains constant during the game.

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

**Rule**: Core preamble should not depend on game specifics (football, hockey, etc.), but provides geometric primitives that can be used for any sport.

### 3.2 Stdlib Preamble (Standard Library)

**File**: `ynwa-scripts/preambles/stdlib.lua`

**Purpose**: Common utilities used by all games and teams.

**Responsibilities**:
- Geometric functions (distances, vectors, angles)
- Search algorithms (nearest player, nearest opponent)
- Mathematical utilities
- Functions for working with regions and grids
- General tactical functions

**Rule**: Stdlib contains no team strategies, only reusable utilities.

### 3.3 Team Preamble

**Purpose**: Functions specific to a particular team's strategy.

**Responsibilities**:
- Team tactics (formation, zones of responsibility)
- Role functions (defender, forward, goalkeeper)
- Coordination between players of the same team
- Team play style

**Rule**: Team preamble can use functions from core and stdlib, but not vice versa. Stdlib can use core, but not vice versa.

## 3.4 Player Regions: Detailed Description

**What are regions?**

Regions are named rectangular areas on the field assigned to specific players. They are used to define:
- **Start positions** - where player should be at the beginning of the game
- **Zones of responsibility** - areas player should defend or patrol
- **Tactical positions** - formation-specific locations

**Region Structure**:

Each region in `context.me.regions` is a table with exact boundaries in meters:

```lua
{
    ["region_name"] = {
        min_x = 10.0,   -- Minimum X coordinate (meters)
        max_x = 15.0,   -- Maximum X coordinate (meters)
        min_z = 20.0,   -- Minimum Z coordinate (meters)
        max_z = 25.0    -- Maximum Z coordinate (meters)
    }
}
```

**How regions are defined**:

In game configuration (TOML), regions are defined using grid notation:

```toml
# Player definition in config
[[players]]
team = "A"
number = 5
start_position = "D3:E4"  # Region from cell D3 to cell E4
```

The core automatically converts grid notation to metric boundaries based on:
1. Field dimensions (width, length)
2. Grid dimensions (number of columns and rows)
3. Cell size = field_width / num_columns

**Coordinate transformation for Team B**:

Important: Team B sees the field from the opposite side. The core automatically transforms region coordinates so that Team B scripts see regions in their own perspective: this means Team B scripts can use regions naturally without thinking about coordinate systems.

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

### 3.7 Loading Order and Isolation

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

## 5. Testing Infrastructure (ynwa-script-tests)

### 5.1 Project Structure

**Directory Layout:**
- `src/lib.rs` - helper functions for creating test games
- `tests/` - integration tests

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

## Support

When developing in `ynwa-scripts`:
1. Create issues for contract ambiguities
2. Propose new functions via PRs to preambles
3. Document all public functions in comments

**Principle**: Lua scripts should be readable and understandable without knowing Rust code.