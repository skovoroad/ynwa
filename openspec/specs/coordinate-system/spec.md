# Coordinate System Specification

## Purpose

Defines the right-handed 3D spatial coordinate system, canonical Display Orientation versus egocentric Team Orientation, Team B 180-degree point-reflection transformations across all geometric primitives, and 2D tactical grid notation and square-cell geometry.

Spatial concepts and their roles:
- **Metric coordinates (`Point3D`)**: continuous 3D positions in meters used by simulation kinematics, physics, and ball/zone boundary checks.
- **Field `Zone`s**: continuous regulation pitch markings (rectangles, circles, arcs, spots such as goals and penalty areas) defined on `Field` for sport rule detection and exposed to scripts via `GAME_DATA.zones`.
- **Tactical grid (`GridCell` and `Region`)**: a discrete 2D spreadsheet-like overlay of square cells (`GridCell` = single square cell; `Region` = inclusive rectangular cell range `top_left..=bottom_right`) used in `tactical.toml` and `Decision::Run` so tactical formations and movement areas are expressed symbolically rather than in raw meters.
- **Display Orientation vs Team Orientation**: **Display Orientation** is the engine's single canonical coordinate frame (`Team::A` perspective: `Team::A` defends `Z <= 0` and attacks `+Z`), used exclusively across `GameConfig`, `GameState`, systems, and journals; **Team Orientation** is each team's egocentric frame (own goal near `Z = 0`, opponent goal near `Z = length`, left wing near `X = 0`) presented to team TOML files and Lua scripts so identical code works for both teams.

## Requirements

### Requirement: Right-Handed 3D Field Coordinate System

All spatial positions and velocities SHALL use a right-handed 3D Cartesian coordinate system (`Point3D`, `Velocity3D`) in meters and meters/second:
- `X` (`0.0..=field.width`): touchline-to-touchline width (`X = 0` left, `X = width` right in Display Orientation).
- `Y` (`>= 0.0`): elevation above turf (`Y = 0.0` on ground).
- `Z` (`0.0..=field.length`): goal-line-to-goal-line length (`Team::A` defends `Z <= 0` and attacks `+Z`; `Team::B` defends `Z >= length` and attacks `-Z`).

#### Scenario: Off-pitch goal depth, staging, and horizontal angle convention
- **WHEN** zones, staging coordinates, or `ArcZone` angles are evaluated in Display Orientation
- **THEN** `Team::A` goal spans `Z in [-goal_depth, 0.0]`, `Team::B` goal spans `Z in [field.length, field.length + goal_depth]`, initial `Setup` off-field staging places players at `(-5.0, 0.0, field.length / 2.0)`, and `ArcZone` angles are measured counter-clockwise from `+X` (`0 rad` = `+X`, `pi/2 rad` = `+Z`)

### Requirement: Canonical Display Orientation and Transformation Boundaries

`GameConfig`, `GameState`, `JournalEvent`, and all core simulation systems SHALL store and process coordinates exclusively in **Display Orientation** (`Team::A` perspective). `Team::B` orientation transforms MUST occur ONLY at three boundaries: team loading into `PlayerDef`, Lua JSON input (`GAME_DATA` and `context`), and Lua decision output (`DecisionSystem`).

#### Scenario: Exact transformation responsibilities at each boundary
- **WHEN** `Team::B` data crosses an engine boundary
- **THEN** team loading flips `Team::B` TOML regions into Display Orientation once at startup, script input assembly flips `GAME_DATA` and per-tick `context` into `Team::B` perspective, and `DecisionSystem` flips returned `Team::B` `Decision` targets back into Display Orientation

#### Scenario: Symmetric script portability across teams
- **WHEN** an identical Lua script and tactical profile are assigned to both `Team::A` and `Team::B`
- **THEN** each team's script observes its own defended goal near `Z = 0`, its opponent's goal near `Z = field.length`, and its left wing near `X = 0`

#### Scenario: Orientation-invariant decision variants
- **WHEN** a `Team::B` player script returns `Decision::Stop` or `Decision::Run(DecisionTarget::Ball)`
- **THEN** `DecisionSystem` stores the decision unchanged without coordinate transformation

### Requirement: Involutory 180-Degree Reflection Formulas for Team B

Flipping between Display Orientation and `Team::B` Orientation SHALL apply a 180-degree point reflection around `(field_width / 2, field_length / 2)` preserving `Y` and ordering invariants (`min <= max`, `top_left <= bottom_right`, `start_angle < end_angle`). Applying any flip operation twice MUST return the original value.

#### Scenario: Point, circle, point-zone, and rectangle reflection
- **WHEN** spatial primitives are flipped on a pitch of width `W` and length `L`
- **THEN** `Point3D(x, y, z)` maps to `(W - x, y, L - z)`, `Circle` and `PointZone` centers `(cx, cz)` map to `(W - cx, L - cz)`, and `Rectangle { min_x, max_x, min_z, max_z }` maps to `{ min_x: W - max_x, max_x: W - min_x, min_z: L - max_z, max_z: L - min_z }`

#### Scenario: GridCell and Region reflection with corner swap
- **WHEN** `GridCell(col, row)` or `Region { top_left, bottom_right }` is flipped on a `(Cols, Rows)` grid
- **THEN** each cell maps to `(Cols - col + 1, Rows - row + 1)` and `Region` swaps flipped corners `(flip(bottom_right), flip(top_left))` so `top_left <= bottom_right` holds (e.g., `"A1:B2"` on `26x40` maps to `"Y39:Z40"`)

#### Scenario: Arc angle reflection in Team B static data
- **WHEN** an `ArcZone` with Display Orientation degree span `[start_deg, end_deg]` is serialized into `GAME_DATA.zones` for `Team::B`
- **THEN** `center` maps to `(W - cx, L - cz)` and degree angles currently map to `start_angle = 180.0 - end_deg`, `end_angle = 180.0 - start_deg` (known latent defect: X-only angle reflection instead of 180-degree rotation; tracked in `fix-team-b-arc-angle-flip`)

### Requirement: Spreadsheet Grid Notation and Region Validation

`GridCell` (single grid square) and `Region` (rectangular range `top_left..=bottom_right`) SHALL address `GridDimensions { columns, rows }` using 1-based base-26 column labels along `X` (`A=1..Z=26, AA=27..`, case-insensitive input, uppercase output) and 1-based row digits along `Z` (`"M22"`). User/TOML region parsing MUST enforce `1 <= top_left <= bottom_right <= (columns, rows)`, whereas Lua `Decision` parsing validates `>= 1` cell syntax without grid upper-bound or corner-order checks.

#### Scenario: Rejecting malformed, inverted, or out-of-bounds TOML grid notation
- **WHEN** grid or region notation parsing receives empty/malformed labels (`""`, `"0"`, `"A0"`, `"1A"`), an inverted range (`"C3:A1"`), or coordinates exceeding `GridDimensions`
- **THEN** TOML/validated region parsing returns an error, while Lua `run` region decision parsing rejects malformed or zero-indexed cell strings but defers range/bound checks until metric resolution

### Requirement: Square-Cell Metric Conversion and Half-Open Region Containment

All grid-to-meter conversions SHALL compute a uniform square `cell_size = field_width / columns as f32` along both `X` and `Z` (enforced on regulation football fields within `0.01` m tolerance). `Region` containment MUST evaluate half-open 2D intervals `x in [min_x, max_x)` and `z in [min_z, max_z)`, ignoring `y`, so adjacent cells and regions partition the pitch without overlap on shared borders.

#### Scenario: Region metric bounds, center, and shared-border disjointness
- **WHEN** a `Region` spans `top_left = (c1, r1)` to `bottom_right = (c2, r2)` with `cell_size = W / columns`
- **THEN** `min_x = (c1 - 1) * cell_size`, `max_x = c2 * cell_size`, `min_z = (r1 - 1) * cell_size`, `max_z = r2 * cell_size`, `center = ((min_x + max_x) / 2, 0.0, (min_z + max_z) / 2)`, and a point lying on border `x = cell_size` belongs to column `B` (`[cell_size, 2 * cell_size)`) and NOT column `A` (`[0, cell_size)`)
