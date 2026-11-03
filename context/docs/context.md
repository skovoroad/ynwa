# YNWA — Project Context

This document contains the development context: what the project is, how it is organized, what each
module consists of, and which contracts hold between layers. It describes the core concepts and
classes; details are moved to companion documents. Only production code is described; tests are not.
The actual sources of truth: the code itself and this documentation.

Companion documents in the context/docs folder:

- `context_game_step.md` — detailed structure of one simulation step (`World::step`): system
  order, decision generation, movement, determinism.
- `context_football.md` — the football layer: field and zones, match stages, set pieces,
  events.
- `protocols.md` — formats and protocols: the JSON contract between the core and the decision engine, the on-disk
  team data format, what the repository reads, the integration-testing scenario file formats.
- `scripting_developer.md` — Lua scripting developer guide: script levels
  (`core.lua`, `stdlib.lua`, team preamble, player script), authoring rules, the
  `make_decision()` dispatcher, examples.
- `scripting_player.md` — the player's Lua scripting reference: attributes and positions, standard-library
  functions, the `team_play`/`player_play` structure, field regions and zones.
- `context_serialization.md` — match recording and replay: the event journal, `Record`,
  the JSON Lines codec, the replay world, the equivalence contract.
- `integration_testing.md` — guide to writing integration tests: scenario structure,
  run plan, expectations, running.

---

## 1. About the Project

A football manager. The user:

1. specifies player attributes and their positions (regular and set-piece), and
   writes decision-making code for them in Lua;
2. runs the match and watches the game.

The simulation is deterministic by steps (the client supplies a fixed timestep), physics is custom,
and player AI is entirely delegated to user Lua scripts.

### Workspace crates

| Crate | Role |
|---|---|
| `ynwa-core` | Simulation core: world model, systems, field, regions, orientation, physics, decision execution. Knows nothing about football. |
| `ynwa-football` | Football rules: field construction, game stages, events (goal, out of play, end of match), ready-made `World` factory. Details — `context_football.md`. |
| `ynwa-decisions` | Game-independent Lua-script execution library. Communicates only via JSON (`protocols.md` §1). |
| `ynwa-repository` | File-based implementation of the core `TeamRepository` (reads `teams/`, see `protocols.md` §3). |
| `ynwa-integration-testing` | Scenario-model, loader, and runner library for automatic integration testing of the full "core — standard library — player scripts" chain. Details — `integration_testing.md`. |
| `ynwa-script-tests` | Script integration-testing infrastructure and visual scenarios. |
| `ynwa-player` | Local client (macroquad + egui). Not described. |
| `ynwa-scripts` | Library of Lua preambles (data, no Rust). Contracts are described in section 7 and in `scripting_developer.md`, `scripting_player.md`. |
| `teams/` | Team data (TOML + Lua). Format — `protocols.md` §2. |

Dependency graph (acyclic):

```
ynwa-player ──> ynwa-football ──> ynwa-core ──> ynwa-decisions
      │                                ▲
      └────────> ynwa-repository ──────┘  (implements the TeamRepository trait)

ynwa-integration-testing ──> ynwa-football ──> ynwa-core
            └──────────────> ynwa-repository
```

Key architectural decision: `ynwa-core` depends neither on football, nor on the storage mechanism,
nor on the client. Everything sport-specific lives in `ynwa-football`, everything storage-specific
in `ynwa-repository`.

---

## 2. Coordinate System and General Conventions

All project layers share one coordinate system and one key convention about "orientation" —
from whose perspective coordinates are written. Understanding this section is required for reading any
project code.

- Right-handed coordinate system, Y up:
  - **X** — field width,
  - **Y** — height above the pitch (3D physics, 2D top-down rendering),
  - **Z** — field length; Team A's goal at Z ≈ 0, Team B's goal at Z ≈ length.
- "Display orientation" (canonical) = Team A's perspective. All internal game state
  is stored only in it.
- Team B's perspective is obtained by mirroring: `x' = width - x`, `z' = length - z`, `y' = y`.
  Mirroring is applied **only at the decision-system boundaries**:
  the input (context/GAME_DATA/regions) is mirrored, the output (decision) is mirrored back.
  This allows both teams' scripts to be written identically.
- Physical quantities are `uom` (`Length`, `Velocity`, `Angle`), not bare `f32`.
- Field grid: columns along X, rows along Z, 1-based numbering, columns in Excel notation
  (A=1 … Z=26, AA=27). A cell is `"M22"`, a region is `"A1:B2"`.

---

## 3. `ynwa-core`

### Responsibility

Match simulation independent of the sport: the entity model, the set of systems, field and zones,
region addressing, orientation transforms, physics, and the bridge to Lua decisions. The core provides
the client with a single point for advancing time — `World::step(delta_time)`; what happens inside
one step is described in detail in `context_game_step.md`.

### Decisions Made

- **Hybrid ECS-like approach**: `config.players[i]` (immutable) ↔ `state.player_states[i]`
  (mutable) — parallel arrays, O(1) access. No external ECS libraries.
- **Config / State are separated** explicitly: `GameConfig` does not change during a match.
- **Single source of randomness**: all randomness goes through `RngManager`, injected into `Game`;
  systems do not keep their own RNG (see "`rng.rs`" below).
- **Player/Ball/Referee are separate types, not a trait**: different systems process them; a common trait
  would not pay off.
- **Poll model**: the game loop belongs to the client; the core provides `World::step(delta_time)`.
- Systems receive `&mut Game` (not `&mut World`) — otherwise the borrow checker would not allow iterating
  the system list; and they receive an **absolute timestamp** rather than a delta — so each system
  computes its own intervals.

### Main Types

Below is a map of the core modules: what lives where and what decisions are baked into each type. System behavior
details during a step — in `context_game_step.md`.

#### `world.rs` — `World`

Root simulation object. Owns `Game` and an ordered list of `Box<dyn System>`.
`step(delta_time)` computes the new timestamp, runs all systems **in insertion order**,
then commits `elapsed_time`. System order is part of the contract (see 3.1).

#### `system.rs` — `System`

The interface of all systems: the single method `update(&mut self, game: &mut Game, timestamp: f32)`.

#### `game.rs` — game model

The central model module: configuration, state, and decision types.

- `Game` — a container of `GameConfig` (private, accessed via `config()`) + public `state` +
  an owned randomness manager (`Box<dyn RngManager>`, accessed via `rng_manager()`).
  `Game::new` starts with the default stage `Setup("kick off")`; `Game::with_stage` sets
  the stage explicitly; both constructors take a randomness manager as a parameter. When starting in `Setup`,
  players are placed **outside the field** at
  `(-5, 0, length/2)`; when starting in `Play`/`GameOver` — at the center of their
  `REGION_START_POSITION` region.
- `GameConfig { field, players: Vec<PlayerDef>, ball: BallDef, referees, scripting }`.
- `GameState { elapsed_time, stage, player_states, ball_state, referee_states, team_stats,
  player_stats, restart_position, restart_team }`.
- `PlayerDef` — static player attributes (`reaction_rate`, `speed_rate`, `tackle_rate`,
  `shot_power`, `shot_accuracy`, all 10–100), their Lua script, a named-region map
  `HashMap<String, Region>`, and `set_piece_roles: HashSet<String>`. The core interprets from regions
  **only** the key `REGION_START_POSITION = "start"` — that is the contract between the core and
  the sport-specific layer; `set_piece_roles` is not interpreted by the core at all. Builder methods
  `with_*` set the attributes.
- `PlayerState` — position, velocity, `current_decision`, `decision_reason`, flags
  `needs_decision` / `decision_processed` / `is_ready`, `last_error`, `last_decision_time`.
- `BallState` — position, velocity, `possessed_by: Option<usize>`,
  `last_possessing_team: Option<Team>` (preserved during passes),
  `last_possession_change_time` (for debouncing).
- `Decision` — `Run(DecisionTarget)` | `Stop` | `Kick(Point3D)`.
  `DecisionTarget` — `Point` | `GridCell` | `Region` | `Ball`.
  The `Ball` variant resolves to the ball's current position: the direction is set once at decision
  processing time (`ActionSystem`), while the "reached" check runs every tick against the ball's current
  position.
- `GameStage` — `Play` | `Setup(String)` | `GameOver`. The string in `Setup` is the set-piece
  reason/type (`"kick off"`, `"throw in"`, `"corner"`, `"goal kick"`). The core itself does not know
  the meaning of these strings.
- `ScriptingConfig` — preamble texts: core, stdlib, Team A and B preambles.
- `StatSet` — named `f64` counters (`get`/`set`/`increment`). The game layer defines the keys
  (e.g., `"score"` is written by `FootballGameManager`). Stats are **not** passed to Lua.

#### `team.rs`

`Team { A, B }` + `opposite()`.

#### `field/` — `Field`, `FieldBuilder`, `Zone`, `ZoneGeometry`

Field geometry: dimensions, grid, and named zones. `Field` stores width, length, `GridDimensions`,
and zones in `HashMap<(name, Option<Team>), Zone>` — O(1) lookup by name+team pair
(`get_zone("goal", Some(Team::A))`). The name and team are duplicated inside `Zone` so a zone is
self-contained when passed to render/logic. `ZoneGeometry` — `Rectangle` | `Circle` | `Arc` |
`Point`; primitives validate input in the constructor via `assert!`. `cell_size()` computes the cell
size as `width / columns` (cells are assumed square).

#### `region.rs` — `GridDimensions`, `GridCell`, `Region`, `RegionError`

Addressing of rectangular field areas in chess-like notation.

- `GridCell::from_notation("AA10")`, `GridCell::column_to_label(col)`.
- `Region::from_grid_notation("A1:B2", dims)`, `to_grid_notation()` (a single-cell region
  prints as `"M3"`).
- `Region::center(dims, field_width)` and `contains_point(...)` compute the metric from **square**
  cells of size `field_width / columns` (along both axes).
- Two creation paths — a deliberate decision: `GridDimensions::create_region(...)` validates
  bounds (for user data), `Region::new(...)` does not validate (for internally generated values —
  mirroring results, single-cell regions).

#### `orientation.rs`

Coordinate mirroring between team perspectives (see section 2): `flip_point_orientation`,
`flip_grid_cell_orientation`, `flip_region_orientation`. When mirroring a region, the corners are
swapped to preserve the invariant `top_left <= bottom_right`.

#### `rng.rs` — `RngConfig`, `RngManager`, `DefaultRngManager`

The single source of randomness in the core — systems do not keep their own RNG.

- `RngManager` — a trait with three methods: `next()` (raw value `[0, 1]`),
  `randomize(base, variation_pct)` (base value with relative spread: `0.25` means ±25%)
  and `randomize_range(max)` (symmetric deviation `[-max, +max]`). The client does not know the
  randomization formulas — it passes a base value and a range. Methods take `&self`: the implementation
  provides interior mutability, so calls go through `game.rng_manager()`.
- **`temperature`** (0.0–1.0) — the "temperature" of non-determinism: `0.0` gives a fully
  reproducible result (`next()` always returns `0.5`), `1.0` full spread, and intermediate values
  scale it linearly.
- `RngConfig { temperature, seed }` — immutable configuration, created via `RngConfig::new`
  (validates temperature). `seed = Some(…)` — reproducible sequence (tests),
  `None` — entropy-based seed (game).
- `DefaultRngManager` (`StdRng`) — the production implementation. For replay a separate
  RNG manager is not needed: the outcomes of random branches are written to the journal (see
  "Recording and Replay").

#### `physics_util.rs`

Movement and kick formulas used by systems (spread and deviation are computed by `RngManager`):

- player speed: `speed_rate / 100 * 10 m/s` (max ≈ 36 km/h);
- `kick_speed(shot_power)` = `shot_power / 5` m/s — base kick speed;
- `max_kick_deviation(shot_accuracy)` — maximum direction deviation: ±5° at accuracy 100,
  ±45° at accuracy 10;
- `rotate_kick_direction(target, ball, deviation)` — rotation of the normalized direction by an angle;
- `distance`, `distance_2d` (without Y), `distance_length`.

#### `repository.rs` — `TeamRepository` and DTOs

Team storage abstraction — the core defines the interface, the client injects the implementation:
`trait TeamRepository { fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> }`.
DTOs: `TeamRecord { players, preamble }`, `PlayerRecord { static_data, tactical, script:
Option<String> }`, `PlayerStatic` (name and 5 attributes), `PlayerTactical { number,
play_positions, set_piece_positions }`. Keys in both position maps are strings; the sport layer
assigns their meaning; the core does not interpret them.

#### Recording and Replay — `journal.rs`, `record.rs`, `codec.rs`, `record_io.rs`, `replay.rs`

A played match can be recorded and later replayed. Why: to review the game,
to debug a specific run, and to write integration tests that verify which events
occurred during a match. Details — `context_serialization.md`.

**What is recorded.** Frames are not saved. The record contains, once, the match configuration (field,
players, scripts), the full initial game state (`GameState`), the time step (`fixed_dt`), and the **journal** — a sequence of significant events
with timestamps: which decisions players made, who took possession, at what speed
the ball actually flew after a kick, stage changes, goals and other football events. Together these
form a `Record`.

**How it is replayed.** Lua scripts and decision-making systems are not run during replay:
decisions are taken from the journal at the right moment, and only physics is recomputed —
player running and ball flight. Randomness is also not recomputed: its result is already recorded
in the journal (who won the ball contest, where the kick actually went). Therefore replay
reproduces the original while the record stays small. The `ReplayDriver` system is responsible for
feeding events.

**How recording is structured in code.** `Game` has a pluggable journal sink (`JournalSink`).
Systems report events by calling `game.record(...)` and do not know where the events go.
The default sink does nothing, so a normal game pays nothing for recording. For recording,
a sink is attached that accumulates events in memory (`CollectJournalRecorder`, convenient
in tests) or writes them straight to a file (`FileJournalRecorder`, JSON Lines format — one line
per event; even a truncated file remains readable).

**Serialization.** Core configuration types support serde, and their output is deterministic:
unordered collections are written sorted. The full game state (`GameState`) is also
serialized and placed in the record header as the initial-state snapshot
(`RecordHeader.initial_state`); the replay world applies it during construction to start from the
same state as the recorded run.

### 3.1 Systems (`systems/`)

The step pipeline consists of six systems; their order matters and is described in detail (along with
each system's behavior, flags, and error handling) in `context_game_step.md`. Here — only
the list and extension points:

1. **Game manager** (sport-specific; for football — `FootballGameManager`) — stages and events.
2. **`PlayerReactionSystem`** — determines whose turn it is to decide (reaction timer).
3. **`BallPossessionSystem`** — ball possession (contest, cooldown, debouncing); contest spread
   comes from `game.rng_manager()`.
4. **`DecisionSystem`** — decision-making coordination; strategy is injected via
   `trait DecisionMaker { make_decision(&mut self, game, player_index) ->
   Result<(Decision, Option<String>), DecisionError> }`. Implementations: `PlaceholderDecisionMaker`
   (stub — random cell) and `ScriptedDecisionMaker` (Lua).
   `DecisionError` — `ScriptError` | `Timeout` | `RuntimeError`.
5. **`ActionSystem`** — turns a decision into velocity/kick.
6. **`PhysicsSystem`** — integrates velocity into position.

#### `ScriptedDecisionMaker` (`systems/decision/scripted_decision_maker.rs`)

An adapter between the core's domain types and the `ynwa-decisions` JSON API (the protocol itself —
`protocols.md` §1). Responsibilities:

- `build_config()` — builds the engine's JSON configuration: team preambles and, for each player,
  their script, team, and `static_data` (field zones and field/grid dimensions), **already mirrored**
  to their team's perspective;
- `build_context()` — builds the JSON state for the current tick (positions, ball, player regions),
  also mirrored for Team B;
- delegates parsing to `decision_parser`, which does **not** mirror (that is done at the system
  boundary, in `DecisionSystem`).

Non-trivial:

- `Arc` zones get recomputed angles when mirrored (`180° − end`, `180° − start`);
- calling `make_decision` during `Setup` is a caller bug, and the method deliberately panics
  with an explicit message rather than silently returning a stub; in `GameOver` it returns `Stop`.

#### `decision_parser.rs`

Converts a JSON decision into a domain `Decision` + `Option<reason>`. Grid bounds are not checked
here (the region is built via `Region::new`) — coordinate validity surfaces later.

#### `systems/decision/util.rs`

`resolve_target_point(decision, field_width, grid_dims, ball_state)` — the single point where a `Run`
target resolves into a `Point3D`; for `Ball` it returns the ball's **current** position (used by the
arrival check).

---

## 4. `ynwa-decisions`

### Responsibility

Game-independent execution of Lua decision-making scripts. No domain types (no `Point3D`,
`Region`, `Team`) — only `serde_json::Value` as the lingua franca. The crate can be published and
reused in another game. The JSON protocol (configuration, context, decision format) is described in
`protocols.md` §1.

### Decisions Made

- **JSON contract** instead of a typed API: the game engine and the decision engine are versioned
  independently.
- **One isolated Lua VM per player**: scripts cannot communicate through global variables.
- **State between calls is not preserved**: the player script is reloaded (`load().exec()`) on
  every call; only the preambles and globals set at VM creation are preserved.
- **Sandbox**: `io`, `os`, `package`, `require`, `load`, `loadfile`, `dofile`,
  `debug`, `collectgarbage`, `_G` are cleared. `math`, `string`, `table`, and base functions remain.
- **Timeout** — an optional hook every 10,000 instructions (≈2× overhead);
  `DecisionEngine` enables it strictly at 100 ms.

### Types

- **`DecisionEngine`** — the facade. The constructor takes the JSON config and the core/stdlib preamble
  texts, creates a `LuaExecutor` per player with the concatenated preamble `core + stdlib + team`, and writes
  `GAME_DATA` from the player's `static_data`. `make_decision(player_index, context)` calls the Lua
  function `make_decision`, validates the result via `LuaDecision`, and returns raw JSON.
- **`LuaExecutor`** — a low-level wrapper over `mlua`: sandbox, preamble, timeout,
  `set_global`, `execute(script, fn, context)`, and `execute_with_args(...)` (the first delegates to
  the second with an empty argument list). The context is placed in the global variable `context`.
- **`LuaDecision`** (`lua_format.rs`) — the serde schema of allowed decisions (`Stop` / `Run` / `Kick`),
  used only for format validation, without translation into domain types.
- Errors: `ScriptError` (`SyntaxError`, `RuntimeError`, `SerializationError`,
  `DeserializationError`, `FunctionNotFound`, `Timeout`) and `DecisionEngineError`
  (`ScriptError`, `RuntimeError`, `Timeout`, `InvalidPlayerIndex`, `InvalidConfig`).

---

## 5. `ynwa-football`

Football rules on top of the core: field geometry with 19 zones, match stages and set pieces
(16 mandatory setup keys, the `"on_ball"` marker for the taker), event detection
(goal, out of play, end of match), and a ready-made `World` factory —
`create_football_world(repo, preambles_path)`. For replaying a record —
`create_football_replay_world(record)` (a reduced system set: `ReplaySetupBallPlacer` →
`ReplayDriver` → `PhysicsSystem`) and `decode_football_events` for extracting events from the journal
(see `context_serialization.md`). The crate depends on the `TeamRepository` trait, not on a
concrete store, and is the only place that knows the meaning of `Setup` stages.

Detailed description — in `context_football.md`.

---

## 6. `ynwa-repository`

File-based `TeamRepository` implementation: reads a team from the `teams/<team_id>/` directory
(`FsTeamRepository::new(base_path)`). What exactly lives in the directory and how it becomes the core's
DTOs — `protocols.md` §2–3.

Decisions made:

- Storage-mechanism isolation: replacing this crate with a DB-backed implementation requires no
  changes elsewhere — the core and the football layer know only the trait.
- Player directories are sorted by name — load order is deterministic (`01`, `02`, …), and the
  player's global index depends on it.

---

## 7. Lua Scripting Contract

The user's player AI is a Lua script running on top of two library preambles
(`ynwa-scripts/preambles/`) and a team preamble. The script environment consists of four levels:
`core.lua`, `stdlib.lua`, the team `preamble.lua`, and the player `script.lua`. What each level is
responsible for, the load order, authoring rules, and examples — in
[`scripting_developer.md`](scripting_developer.md); the function and structure reference for the player — in
[`scripting_player.md`](scripting_player.md).

In short:

- `core.lua` — elementary state access (`my_position()`, `ball_position()`, …);
- `stdlib.lua` — utilities, state queries, primitive and tactical actions, the dispatcher
  `make_decision()`;
- team `preamble.lua` — team tactics: the `team_play` table and common tables (e.g.,
  `goalkeeper_play`);
- player `script.lua` — optional partial override via `player_play`.

`core.lua` and `stdlib.lua` are project code; teams do not modify them; `preamble.lua` and `script.lua` are
team code.

The context format (`context`, `GAME_DATA`) and the returned decision format —
[`protocols.md`](protocols.md) §1; the on-disk team data format — [`protocols.md`](protocols.md) §2.

---

## 8. `ynwa-script-tests`

Infrastructure for verifying that Lua scripts yield the expected decisions through the full system pipeline.

- `src/lib.rs` — helpers for building mini-games: a game with one player and a given script, with
  preambles and without, at a given stage; fixture loading; `request_decisions_for_all` to bypass
  `PlayerReactionSystem` timings.
- `fixtures/` — minimal dispatch tables and a script spy.
- `scenarios/<name>/teams/` — self-contained team sets for **visual** verification of specific
  game situations; visual review is done from a record: `./replay.sh <path_to_record>` (e.g.,
  `~/.ynwa/scenarios/<scenario>.jsonl`).

---

## 9. Integration Testing (`ynwa-integration-testing`)

To protect the full "core — standard library — player scripts" chain against regression, the project
has the `ynwa-integration-testing` crate: it runs granular **scenarios** automatically, as part of
ordinary `cargo test`, without human involvement and without real time.

Each scenario is a directory with a partial initial state (`initial_state.toml`), a run plan and
expectations (`scenario.toml`), an optional expected final state
(`final_state.toml`), and a team set (`teams/`). The runner deterministically builds the football world
(fixed RNG seed, zero temperature), applies the initial snapshot, steps
`world.step(dt)` until the first triggered stop criterion, and compares the facts against the expectations: the event
journal, the stop reason and step count, and the final state (stage, positions, possession, score).
Discrepancies are printed as a human-readable diff in the `cargo test` output.

How to create your own scenario — in [`integration_testing.md`](integration_testing.md); the exact
scenario file formats — [`protocols.md`](protocols.md) §4.

---

## 10. Known Discrepancies and Unfinished Places

Verified against the code as of writing — useful to know so as not to trust old docs.
Football-specific discrepancies are listed in `context_football.md` §7.

- **Field dimensions.** Old docs mention 60 × 101.5 m and a 26 × 44 grid — these are test-field
  parameters. The production field: 68 × 104.615 m, grid 26 × 40 (`field_builder.rs`).
- **Grid bounds for Lua decisions are not validated** in the parser — an invalid region surfaces
  later, when computing the center.
- **The referee** (`RefereeDef`/`RefereeState`) exists as a stub; no system moves it.
- References in old docs to `ynwa-scripts/context.md` and `config.rs` are outdated: such files do not exist.
