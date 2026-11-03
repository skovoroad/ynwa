# Match Recording and Replay

Context on serializing game simulation results (task_3). Goals: match recording/replay
(play, debugging) and event-journal integration tests. The initial-state snapshot in the record
is implemented in `task_4` (goal 1). Extracted into separate tasks: rewind/drift correction (`task_4`,
goals 2–3), format versioning (`task_5`), cross-platform determinism (`task_6`),
player integration (`task_7`).

## 1. Model

**Event-driven physics re-simulation.** No per-frame recording is kept: only significant events with
timestamps are written to the journal (the absolute step time — the `System::update` argument).
During replay, Lua and the decision-making systems are not run — events are fed from the journal,
and only physics is recomputed.

- **RNG is recorded by outcomes**, not raw values: the possession change and the actual ball speed
  after a kick. Replay does not depend on RNG.
- **The step is fixed**: `fixed_dt` and `total_steps` are stored in the record.
- **The record model is separated from encoding**: `Record` — the in-memory model, bytes — behind the
  `RecordWriter`/`RecordReader` traits.
- Determinism is guaranteed within a single platform. Serde output is stable: `HashMap`/`HashSet`
  serialize in sorted order (`sorted_collections` in `game.rs`), `Field` zones — in
  deterministic order.

## 2. Serde

`Serialize`/`Deserialize` are present on the core configuration types (`GameConfig`, `PlayerDef`, `Field`,
zones, `Region`/`GridCell`, `Team`, `GameStage`, `Decision`, `Point3D`/`Velocity3D`, etc.), on
`JournalEntry`/`JournalEvent`, on `FootballEvent`, and on the full game state (`GameState` and
the nested `PlayerState`/`BallState`/`RefereeState`/`StatSet`). `GameState` is placed in the record as
the initial-state snapshot; `HashMap`/`HashSet` inside it (including `team_stats` by `Team`)
serialize deterministically via `sorted_collections`.

## 3. Journal (`ynwa-core/src/journal.rs`)

The journal is the only channel through which the game reports significant events. Systems only
report an event; what to do with it (drop, accumulate, write to a file) is decided by the attached
sink. Therefore system code does not depend on the storage mechanism.

- `JournalEntry { timestamp, event }`; `JournalEvent`: `DecisionAssigned`, `PossessionChange`,
  `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`,
  `External { kind, data }` (sport-specific events; the core does not interpret them, football places
  `FootballEvent` there with `kind = "football_event"`).
- `trait JournalSink { push, finish_step, finish }` — a pure sink; the implementation decides where events go.
  The sink counts steps itself via `finish_step`.
- Implementations: `NullJournalSink` (default), `CollectJournalRecorder` (writes to an external
  `Rc<RefCell<EventsCollection>>`; `EventsCollection::take_record(header)` assembles a `Record`),
  `FileJournalRecorder` (`codec.rs`, streaming write with flush per line; write errors accumulate
  and are returned from `finish`).
- `Game` owns the sink: `set_journal_sink`, `record(timestamp, event)`, `finish_step`,
  `finish_journal()` (detaches the sink and finalizes). `World::step` calls
  `Game::finish_step` at the end of each step.

Recording points: `DecisionSystem` (`DecisionAssigned`, in display orientation),
`BallPossessionSystem` (`PossessionChange`), `ActionSystem` (`KickOutcome`),
`FootballGameManager` (a football event as `External`, then its effects: `StageChange`,
`RestartSet`, `DecisionsReset`, `StatUpdate`; `Setup` positioning decisions — `DecisionAssigned`).

## 4. Record and Format

A record is a header (everything needed to rebuild the world) plus a journal and a step count.
The byte format is kept behind traits so a binary codec can be added later without changing the
model.

- `record.rs`: `RecordHeader { config, initial_state: GameState, fixed_dt }`
  (`RecordHeader::from_game` captures the configuration and current state), `Record { header,
  total_steps, journal }`. The stage is part of `initial_state.stage`; there is no separate `initial_stage`
  field.
- `codec.rs`: `RecordWriter`/`RecordReader`, the JSON Lines implementations `JsonRecordWriter`/
  `JsonRecordReader`. Lines: `header`, `event`…, `footer { total_steps }`. A truncated file
  (without a footer) reads as a valid prefix.
- `record_io.rs`: constructors `json_journal_file_writer/reader` and
  `json_journal_memory_writer/reader` (`SharedBytes` — an in-memory buffer), as well as
  `write_json_record(path, &Record)` — writes a ready `Record` in full (header, events, footer).

Typical usage:

```rust
let collection = Rc::new(RefCell::new(EventsCollection::default()));
let header = RecordHeader::from_game(world.game(), dt);
world.game_mut().set_journal_sink(Box::new(CollectJournalRecorder::new(collection.clone())));
// world.step(dt) × N
world.game_mut().finish_journal()?;
let record = collection.borrow_mut().take_record(header);
```

## 5. Replay

For replay, a separate world is built: it has no Lua and no decision-making systems, and decisions and
random-branch outcomes are taken from the journal. Physics is the same as in the original, so the
result matches the original run.

- `ynwa-core/src/replay.rs`: `ReplayDriver` — a system that applies journal entries with
  `timestamp <= current` by cursor. The velocity calculation from a decision is shared with `ActionSystem`
  (`systems/movement.rs`, `pub(crate)`) so physics matches.
- `ynwa-football/src/replay.rs`:
  - `create_football_replay_world(record)` — builds `Game` from `record.header.config`, then
    applies `record.header.initial_state` (`Game::apply_state`); system set:
    `ReplaySetupBallPlacer` (ball pin in `Setup`, as in the manager) → `ReplayDriver` →
    `PhysicsSystem`. The client calls `world.step(fixed_dt)` exactly `total_steps` times.
  - `decode_football_events(&[JournalEntry])` — extraction of `FootballEvent` from `External`.

**Replay equivalence contract**: player and ball positions/velocities match, `possessed_by`,
`last_possessing_team`, `last_possession_change_time`, `stage`, `team_stats`, `restart_*`,
`elapsed_time`. Player coordination flags (`needs_decision`, `is_ready`, `last_error`, …) are not
guaranteed.

## 6. Tests

Infrastructure in `ynwa-football/src/test_utils.rs` (`assert_equivalent`, lockstep run).
Lockstep tests compare the original and replay at each step; `check_events` is captured **before** the step
and only in the `Play` stage (the manager detects events as the first system, on the previous
tick's state). Covered: the end-to-end path through a file, RNG independence, ball contest.
