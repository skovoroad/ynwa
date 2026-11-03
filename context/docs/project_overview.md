# YNWA — Project Context

This document contains the development context: what the project is, how it is organized, what each
module consists of, and which contracts hold between layers. It describes the core concepts and
classes; details are moved to companion documents. Only production code is described; tests are not.
The actual sources of truth: the code itself and this documentation.

Companion documents in the context/docs folder:

- `coordinate_system.md` — the coordinate system and general conventions shared by all layers:
  axes, display orientation, mirroring, grid notation.
- `core_types.md` — `ynwa-core` internals: detailed description of types and the decisions baked
  into them, the systems of the step pipeline.
- `game_step.md` — detailed structure of one simulation step (`World::step`): system
  order, decision generation, movement, determinism.
- `football.md` — the football layer: field and zones, match stages, set pieces,
  events.
- `protocols.md` — formats and protocols: the JSON contract between the core and the decision engine, the on-disk
  team data format, what the repository reads, the integration-testing scenario file formats.
- `scripting_developer.md` — Lua scripting developer guide: script levels
  (`core.lua`, `stdlib.lua`, team preamble, player script), authoring rules, the
  `make_decision()` dispatcher, examples.
- `scripting_player.md` — the player's Lua scripting reference: attributes and positions, standard-library
  functions, the `team_play`/`player_play` structure, field regions and zones.
- `serialization.md` — match recording and replay: the event journal, `Record`,
  the JSON Lines codec, the replay world, the equivalence contract.
- `integration_testing.md` — guide to writing integration tests: scenario structure,
  run plan, expectations, running.
- `player_rendering.md` — field-to-screen mapping used by the client (rendering coordinate system).

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
| `ynwa-core` | Simulation core: world model, systems, field, regions, orientation, physics, decision execution. Knows nothing about football. Details — `core_types.md`. |
| `ynwa-football` | Football rules: field construction, game stages, events (goal, out of play, end of match), ready-made `World` factory. Details — `football.md`. |
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

All project layers share one right-handed coordinate system (Y up) and one convention about
"display orientation": the internal state is stored from Team A's perspective, and Team B's data is
mirrored only at the decision-system boundaries. Details — [`coordinate_system.md`](coordinate_system.md).

---

## 3. `ynwa-core`

### Responsibility

Match simulation independent of the sport: the entity model, the set of systems, field and zones,
region addressing, orientation transforms, physics, and the bridge to Lua decisions. The core provides
the client with a single point for advancing time — `World::step(delta_time)`; what happens inside
one step is described in detail in `game_step.md`.

### Decisions Made

- **Hybrid ECS-like approach**: `config.players[i]` (immutable) ↔ `state.player_states[i]`
  (mutable) — parallel arrays, O(1) access. No external ECS libraries.
- **Config / State are separated** explicitly: `GameConfig` does not change during a match.
- **Single source of randomness**: all randomness goes through `RngManager`, injected into `Game`;
  systems do not keep their own RNG.
- **Player/Ball/Referee are separate types, not a trait**: different systems process them; a common trait
  would not pay off.
- **Poll model**: the game loop belongs to the client; the core provides `World::step(delta_time)`.
- Systems receive `&mut Game` (not `&mut World`) — otherwise the borrow checker would not allow iterating
  the system list; and they receive an **absolute timestamp** rather than a delta — so each system
  computes its own intervals.

### Modules

Below is a map of the core modules and their purpose. The detailed description of the types and the
decisions baked into them — in [`core_types.md`](core_types.md); system behavior during a step —
in [`game_step.md`](game_step.md).

| Module | Purpose |
|---|---|
| `world.rs` | Root simulation object `World`: owns `Game` and the ordered system list, advances time. |
| `system.rs` | The `System` interface. |
| `game.rs` | Game model: configuration, state, and decision types. |
| `team.rs` | The `Team` enum and `opposite()`. |
| `field/` | Field geometry: dimensions, grid, and named zones. |
| `region.rs` | Addressing of rectangular field areas in chess-like notation. |
| `orientation.rs` | Coordinate mirroring between team perspectives. |
| `rng.rs` | The single source of randomness (`RngManager`). |
| `physics_util.rs` | Movement and kick formulas. |
| `repository.rs` | `TeamRepository` abstraction and its DTOs. |
| `journal.rs`, `record.rs`, `codec.rs`, `record_io.rs`, `replay.rs` | Match recording and replay. |

### 3.1 Systems (`systems/`)

The step pipeline consists of six systems; their order matters and is described in detail (along with
each system's behavior, flags, and error handling) in [`game_step.md`](game_step.md). Extension
points and the details of the decision adapters — in [`core_types.md`](core_types.md) §2.

1. **Game manager** (sport-specific; for football — `FootballGameManager`) — stages and events.
2. **`PlayerReactionSystem`** — determines whose turn it is to decide (reaction timer).
3. **`BallPossessionSystem`** — ball possession (contest, cooldown, debouncing).
4. **`DecisionSystem`** — decision-making coordination; the strategy is injected.
5. **`ActionSystem`** — turns a decision into velocity/kick.
6. **`PhysicsSystem`** — integrates velocity into position.

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
(see `serialization.md`). The crate depends on the `TeamRepository` trait, not on a
concrete store, and is the only place that knows the meaning of `Setup` stages.

Detailed description — in [`football.md`](football.md).

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
Football-specific discrepancies are listed in [`football.md`](football.md) §7.

- **Field dimensions.** Old docs mention 60 × 101.5 m and a 26 × 44 grid — these are test-field
  parameters. The production field: 68 × 104.615 m, grid 26 × 40 (`field_builder.rs`).
- **Grid bounds for Lua decisions are not validated** in the parser — an invalid region surfaces
  later, when computing the center.
- **The referee** (`RefereeDef`/`RefereeState`) exists as a stub; no system moves it.
- References in old docs to `ynwa-scripts/context.md` and `config.rs` are outdated: such files do not exist.
