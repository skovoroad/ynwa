# Football Rules Specification

## Purpose

Defines the football domain layer (`ynwa-football`): regulation pitch dimensions and all 19 named zones, mandatory set-piece tactical keys and `on_ball` validation, `FootballWorldBuilder` assembly, match event detection (`Goal`, `GameEnd`, `Touchline`, `GoalLine`), and `FootballGameManager` stage transitions and set-piece orchestration.

Key domain concepts:
- **Set-Piece Keys**: 16 string identifiers bridging `ynwa-core`'s sport-agnostic `GameStage::Setup(reason)` to concrete football restart formations (combining stoppage `reason`, restarting team `own`/`opp`, flank `left`/`right`, and field half `own half`/`opp half`) to look up each player's `Setup` assignment from `tactical.toml`.
- **`"on_ball"` marker**: special `tactical.toml` value used instead of a grid `Region` to designate the single taker of an `own` set piece (`PlayerDef::set_piece_roles`), who runs to the ball's restart point while teammates run to their assigned `Region`s.

## Requirements

### Requirement: Regulation Football Pitch Dimensions and Aspect Ratio

`create_football_field()` SHALL construct a `68.0` m × `104.615_38` m pitch with a `26 × 40` tactical grid (`cell_size ≈ 2.615385` m), `GOAL_WIDTH = 7.32` m, `GOAL_DEPTH = 2.5` m, and `GOAL_KICK_OFFSET = 5.5` m. Custom pitch construction via `create_football_field_with_dimensions(width, length, cols, rows)` MUST return `Err` whenever `|(width / cols) - (length / rows)| > 0.01` m so tactical grid cells remain square.

#### Scenario: Rejecting non-square custom pitch grid dimensions
- **WHEN** `create_football_field_with_dimensions` is called with `(width / cols)` and `(length / rows)` differing by more than `0.01` m
- **THEN** it returns `Err` without panicking

### Requirement: Complete 19-Zone Football Pitch Geometry

Every football `Field` SHALL register exactly **19** zones (`3` neutral `None` zones + `8` per-team zones for `Some(Team::A)` and `Some(Team::B)` in Display Orientation), and every `ArcZone` MUST satisfy `start_angle < end_angle` in radians. `BallDef::initial_position` MUST equal the `center_spot` point `(W / 2, 0.0, L / 2)`.

#### Scenario: Neutral and rectangular team zones
- **WHEN** a football pitch of width `W` and length `L` builds its rectangular, circular, and point zones
- **THEN** neutral zones are `field` (`Rect [0..W, 0..L]`), `center_circle` (`Circle (W/2, L/2), r = 9.15`), and `center_spot` (`Point (W/2, 0, L/2)`)
- **AND** per-team zones (`Team::A` at `Z = 0`, `Team::B` mirrored at `Z = L`) are `half` (`[0..L/2]` / `[L/2..L]`), `goal_area` (centered width `18.32`, length `5.5`), `penalty_area` (centered width `47.076_92`, length `18.307_69`), `penalty_spot` (`Point (W/2, 0, 11.0)` / `(W/2, 0, L - 11.0)`), and `goal` (centered width `7.32`, `A: z in [-2.5, 0.0]`, `B: z in [L, L + 2.5]`)

#### Scenario: Penalty arc and corner arc radian spans
- **WHEN** a football pitch of width `W` and length `L` builds its `ArcZone`s
- **THEN** `penalty_arc` (`r = 9.15` at `penalty_spot`, `dz = 18.307_69 - 11.0`, `dx = sqrt(9.15^2 - dz^2)`) spans `[atan2(dz, dx), atan2(dz, -dx)]` for `A` and `[atan2(-dz, -dx), atan2(-dz, dx)]` for `B`
- **AND** `corner_arc_bottom` (`r = 1.0` at `x = 0`) spans `A: (0, 0) [0, pi/2]` and `B: (0, L) [-pi/2, 0]`, while `corner_arc_top` (`r = 1.0` at `x = W`) spans `A: (W, 0) [pi/2, pi]` and `B: (W, L) [pi, 3*pi/2]`

### Requirement: Mandatory 16 Set-Piece Keys and `on_ball` Role Validation

`FootballWorldBuilder` SHALL require every player on both teams to declare all **16** `SET_PIECE_KEYS` in `tactical.set_piece_positions`. Each of the **8** `own` keys MUST have `count("on_ball") == 1` per team; all other keys (including all **8** `opp` keys) MUST have `count("on_ball") == 0`. `"on_ball"` populates `PlayerDef::set_piece_roles` (not `regions`), and `"kick off opp"` aliases `REGION_START_POSITION` (`"start"`, temporary workaround until kickoff team selection is implemented).

#### Scenario: Exact catalog of 16 mandatory set-piece keys
- **WHEN** team tactical configurations are validated at startup
- **THEN** the 8 `own` keys (`count("on_ball") == 1`) are `"kick off own"`, `"goal kick own"`, `"corner own left"`, `"corner own right"`, `"throw in own left own half"`, `"throw in own left opp half"`, `"throw in own right own half"`, and `"throw in own right opp half"`
- **AND** the 8 `opp` keys (`count("on_ball") == 0`) are `"kick off opp"`, `"goal kick opp"`, `"corner opp left"`, `"corner opp right"`, `"throw in opp left own half"`, `"throw in opp left opp half"`, `"throw in opp right own half"`, and `"throw in opp right opp half"`

### Requirement: FootballWorldBuilder vs `create_football_world` Error Policy

`FootballWorldBuilder::new(repo, preambles_path)` SHALL load `"team_a"` then `"team_b"`, flip `Team::B` regions to Display Orientation, load `core.lua` and `stdlib.lua`, default to `PLAYER_RNG_TEMPERATURE = 0.7` (unseeded) and `Setup("kick off")`, and wire the 6-system live football pipeline. `build()` MUST return `Err` on `ScriptedDecisionMaker` failure unless `.with_placeholder_fallback()` is enabled (as in `create_football_world`).

#### Scenario: Strict builder failure vs playable fallback
- **WHEN** `team_a/preamble.lua` contains a Lua syntax error
- **THEN** `FootballWorldBuilder::new(repo, preambles).build()` returns `Err`, whereas `create_football_world(repo, preambles)` logs a warning to `stderr` and returns `Ok(World)` with `PlaceholderDecisionMaker` (while missing TOML files or invalid set-piece keys still return `Err` in both)

### Requirement: Football Event Detection Priority and Boundary Conditions

During `GameStage::Play` ONLY, `check_events(game)` SHALL return at most **one** `FootballEvent` per step in priority order: (1) `Goal(conceding_team)` (scorer is `conceding_team.opposite()`), (2) `GameEnd` (`elapsed_time >= GAME_DURATION`, currently `120.0` s debug value), (3) `Touchline(ball_pos, last_team)`, (4) `GoalLine(ball_pos, last_team)`. Boundary checks MUST account for `BALL_RADIUS = 0.11` m, and `last_team` uses `last_possessing_team` (defaulting to `Team::A` when `None`).

#### Scenario: Exact geometric predicates for Goal, Touchline, and GoalLine
- **WHEN** `check_events` evaluates ball center `(bx, by, bz)` with `r = 0.11` m and goal X-span `[gx_min, gx_max]`
- **THEN** `Goal(team)` triggers iff `bz - r >= gz_min && bz + r <= gz_max && gx_min <= bx && bx <= gx_max`; `Touchline` triggers iff `bx + r < 0.0 || bx - r > W`; and `GoalLine` triggers iff `(bz + r < 0.0 || bz - r > L)` AND `!(gx_min <= bx && bx <= gx_max)`

### Requirement: FootballGameManager Event Transitions and Restart Assignment

When `check_events` yields an event in `Play`, `FootballGameManager` SHALL first emit `JournalEvent::External { kind: "football_event", data }` (`{"event": "goal"|"touchline"|"goal_line"|"game_end", "team"?: "A"|"B"}`) and then apply stage/restart transitions: `GameEnd` -> `GameOver`; `Goal`, `Touchline`, and `GoalLine` reset all players (`is_ready=false, current_decision=None, needs_decision=true`, emitting `DecisionsReset`), emit `RestartSet` and `StageChange`, and enter `Setup(reason)`.

#### Scenario: Restart parameters for Goal, Touchline, Goal Kick, and Corner
- **WHEN** `FootballGameManager` handles a boundary or scoring event on pitch `(W, L)`
- **THEN** `Goal(conceding)` increments `team_stats[conceding.opposite()]["score"]` by `1.0` (`StatUpdate`) and sets `Setup("kick off")` with `restart_position = None`, `restart_team = Some(conceding)`
- **AND** `Touchline(pos, last)` sets `Setup("throw in")` with `restart_position = Some(pos)`, `restart_team = Some(last.opposite())`
- **AND** `GoalLine(pos, last)` (with `attacking = B` if `pos.z < L/2` else `A`) sets `Setup("goal kick")` at `(W/2, 0, 5.5)` or `(W/2, 0, L - 5.5)` with `restart_team = Some(last.opposite())` when `last == attacking`, or `Setup("corner")` at nearest corner `{(0,0,0), (W,0,0), (0,0,L), (W,0,L)}` with `restart_team = Some(attacking)` when `last != attacking`
- **AND** `restart_position` and `restart_team` are latched before `Setup` ticks clear `ball_state.last_possessing_team = None`

### Requirement: FootballGameManager Setup Orchestration and Readiness Transition

On every `Setup(reason)` tick, `FootballGameManager` SHALL: (1) pin the ball to `restart_position.unwrap_or(ball.initial_position)` (`velocity = 0`, free ownership, `last_possessing_team = None`), (2) for players with `current_decision.is_none()`, resolve their set-piece key and assign `Run(Point(ball_target))` (`"on_ball"`) or `Run(Region)`, recording `last_error` on failure, and (3) mark `Stop` players `is_ready = true`, entering `Play` once all are ready.

#### Scenario: Perspective-aware set-piece key resolution and Play transition
- **WHEN** `FootballGameManager` assigns `Setup(reason)` targets on pitch `(W, L)`
- **THEN** key resolution computes `ownership = "own"` iff `restart_team == Some(player_team)` else `"opp"` (at initial kickoff `restart_team == None`, so both teams resolve `"kick off opp"`), `side = "left"` iff `(A: x < W/2, B: x >= W/2)` else `"right"`, and `half = "own half"` iff `(A: z < L/2, B: z >= L/2)` else `"opp half"`, forming `"{reason} {ownership}"`, `"corner {ownership} {side}"`, or `"throw in {ownership} {side} {half}"`
- **AND** valid targets assign `current_decision` (`decision_processed = false`) and emit `DecisionAssigned { reason: None }`, and once every player reaches `Stop` (`is_ready == true`), `StageChange { stage: Play }` is emitted

#### Scenario: Unknown setup reason or missing tactical region error handling
- **WHEN** `Setup(reason)` key resolution fails for an unknown `reason` or a non-taker lacks a region for the resolved key
- **THEN** `player_states[i].last_error` is populated, `current_decision` remains `None` so the player stays stationary, and `is_ready` remains `false`
