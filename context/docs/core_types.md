# `ynwa-core`: Types and Systems

Detailed description of `ynwa-core` internals: the types of each module, the decisions baked into
them, and the systems of the step pipeline. The purpose of each module is listed in
[`project_overview.md`](project_overview.md) §3; system behavior during a step is described in
[`game_step.md`](game_step.md).

Companion documents: [`project_overview.md`](project_overview.md) (general project context),
[`coordinate_system.md`](coordinate_system.md) (coordinate conventions),
[`serialization.md`](serialization.md) (recording and replay).

---

## 1. Main Types

Below is a map of the core modules: what lives where and what decisions are baked into each type.

### `world.rs` — `World`

Root simulation object. Owns `Game` and an ordered list of `Box<dyn System>`.
`step(delta_time)` computes the new timestamp, runs all systems **in insertion order**,
then commits `elapsed_time`. System order is part of the contract (see §2).

### `system.rs` — `System`

The interface of all systems: the single method `update(&mut self, game: &mut Game, timestamp: f32)`.

### `game.rs` — game model

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

### `team.rs`

`Team { A, B }` + `opposite()`.

### `field/` — `Field`, `FieldBuilder`, `Zone`, `ZoneGeometry`

Field geometry: dimensions, grid, and named zones. `Field` stores width, length, `GridDimensions`,
and zones in `HashMap<(name, Option<Team>), Zone>` — O(1) lookup by name+team pair
(`get_zone("goal", Some(Team::A))`). The name and team are duplicated inside `Zone` so a zone is
self-contained when passed to render/logic. `ZoneGeometry` — `Rectangle` | `Circle` | `Arc` |
`Point`; primitives validate input in the constructor via `assert!`. `cell_size()` computes the cell
size as `width / columns` (cells are assumed square).

### `region.rs` — `GridDimensions`, `GridCell`, `Region`, `RegionError`

Addressing of rectangular field areas in chess-like notation.

- `GridCell::from_notation("AA10")`, `GridCell::column_to_label(col)`.
- `Region::from_grid_notation("A1:B2", dims)`, `to_grid_notation()` (a single-cell region
  prints as `"M3"`).
- `Region::center(dims, field_width)` and `contains_point(...)` compute the metric from **square**
  cells of size `field_width / columns` (along both axes).
- Two creation paths — a deliberate decision: `GridDimensions::create_region(...)` validates
  bounds (for user data), `Region::new(...)` does not validate (for internally generated values —
  mirroring results, single-cell regions).

### `orientation.rs`

Coordinate mirroring between team perspectives (see [`coordinate_system.md`](coordinate_system.md)):
`flip_point_orientation`, `flip_grid_cell_orientation`, `flip_region_orientation`. When mirroring a
region, the corners are swapped to preserve the invariant `top_left <= bottom_right`.

### `rng.rs` — `RngConfig`, `RngManager`, `DefaultRngManager`

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
  [`serialization.md`](serialization.md)).

### `physics_util.rs`

Movement and kick formulas used by systems (spread and deviation are computed by `RngManager`):

- player speed: `speed_rate / 100 * 10 m/s` (max ≈ 36 km/h);
- `kick_speed(shot_power)` = `shot_power / 5` m/s — base kick speed;
- `max_kick_deviation(shot_accuracy)` — maximum direction deviation: ±5° at accuracy 100,
  ±45° at accuracy 10;
- `rotate_kick_direction(target, ball, deviation)` — rotation of the normalized direction by an angle;
- `distance`, `distance_2d` (without Y), `distance_length`.

### `repository.rs` — `TeamRepository` and DTOs

Team storage abstraction — the core defines the interface, the client injects the implementation:
`trait TeamRepository { fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> }`.
DTOs: `TeamRecord { players, preamble }`, `PlayerRecord { static_data, tactical, script:
Option<String> }`, `PlayerStatic` (name and 5 attributes), `PlayerTactical { number,
play_positions, set_piece_positions }`. Keys in both position maps are strings; the sport layer
assigns their meaning; the core does not interpret them.

### Recording and Replay — `journal.rs`, `record.rs`, `codec.rs`, `record_io.rs`, `replay.rs`

A played match can be recorded and later replayed. Why: to review the game,
to debug a specific run, and to write integration tests that verify which events
occurred during a match. Details — [`serialization.md`](serialization.md).

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

---

## 2. Systems (`systems/`)

The step pipeline consists of six systems; their order matters and is described in detail (along with
each system's behavior, flags, and error handling) in [`game_step.md`](game_step.md).

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

### `ScriptedDecisionMaker` (`systems/decision/scripted_decision_maker.rs`)

An adapter between the core's domain types and the `ynwa-decisions` JSON API (the protocol itself —
[`protocols.md`](protocols.md) §1). Responsibilities:

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

### `decision_parser.rs`

Converts a JSON decision into a domain `Decision` + `Option<reason>`. Grid bounds are not checked
here (the region is built via `Region::new`) — coordinate validity surfaces later.

### `systems/decision/util.rs`

`resolve_target_point(decision, field_width, grid_dims, ball_state)` — the single point where a `Run`
target resolves into a `Point3D`; for `Ball` it returns the ball's **current** position (used by the
arrival check).