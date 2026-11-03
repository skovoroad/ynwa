# Task

Implement full game-state snapshots in the record and their use during replay.

We already have partial snapshots for integration-testing purposes, implemented in the ynwa-integration-testing module. They allow adjusting the initial game state and checking the final game state when running a test scenario.

However, we want to extend their use:

1. Playing integration-testing scenarios in the player. The simulation must be recorded using the core's facilities (which exist) so it can later be replayed in the player. But the game record has no initial state, only the initial team lineup, and in a test scenario it may be adjusted.
2. Rewinding in the player.
3. Correcting numerical drift when playing a scenario (because physics is re-simulated).

Our goal — implement snapshot support for goal 1, so that goals 2 and 3 can be implemented in the future.

For goal 1 it is sufficient to:

- serialize the full game state (`GameState`) and place an initial-state snapshot in the record;
- apply this snapshot when creating the replay world, so replay starts from the same state as the scenario (stage, player/ball positions/velocities, possession, score, `restart_*`);
- make the `ynwa-integration-testing` scenario run write its result to a record-format file (JSON Lines), and have the player open it in Replay mode.

DOD:

- the snapshot and game-recording mechanism is extended so that integration-testing scenarios can be verified visually by running their records in the player;
- the result is checked automatically: a replay world starting from the initial-state snapshot is equivalent to the original scenario run (lockstep comparison, as in the existing replay tests).

Note for the architectural decision: `ynwa-core` already has the type `PartialStateSpec` (partial state specification for integration testing). The full state in the record is represented by `GameState` itself, so there is no name conflict.

---

# Architectural Decision

## 1. General Scheme

The full initial game state is added to the record (`Record`). The replay world builds the initial
state in the same two phases as the scenario: first a `Game` is assembled from `config`, then the
initial-state snapshot from the record is applied on top. `config` stays in the header (see below) —
`Game` is built from it, and static data for replay physics is taken from it. This closes goal 1
of the task: the `ynwa-integration-testing` scenario run (where the initial state is adjusted
by a partial `PartialStateSpec`) is written to a JSON Lines file, and the player opens it in `--replay`
and sees exactly the same scenario start.

The full state in the record is `GameState` itself — it gets serde. We do not introduce a separate
"record snapshot" type: the partial `PartialStateSpec` remains for integration-testing
scenarios, and the serialized [`GameState`](ynwa-core/src/game.rs:261) lies in the record. Thus
name conflict is ruled out, and for the future goals 2–3 (rewind, drift correction) the same `GameState`
can be placed in the record at an arbitrary step as well.

[`RecordHeader`](ynwa-core/src/record.rs:9) changes: `{ config, fixed_dt, initial_state: GameState }`
instead of `{ config, initial_stage, fixed_dt }`. The stage is contained in `initial_state.stage`, the separate
field `initial_stage` is removed — one source of truth, no possibility of divergence.

`config` remains in the header: the replay world builds `Game` from it and takes static data
(`PlayerDef`, `Field`, scripts), and `ReplayDriver` takes player attributes (`speed_rate`) for
physics calculation. The snapshot replaces only the *initial state* (`state`), not the match configuration.

Flow for a scenario:

1. `ScenarioLoader` assembles the world and applies the partial `PartialStateSpec` (as now).
2. [`run()`](ynwa-integration-testing/src/runner.rs:34) at the start captures
   `RecordHeader::from_game(world.game(), run.dt)` — that is the full initial-state snapshot
   of the scenario.
3. The run writes the journal to a `CollectJournalRecorder` (as now).
4. After the run, `run()` assembles a `Record` and returns it in `RunOutcome`.
5. `run_scenario()` serializes the `Record` to `~/.ynwa/scenarios/<scenario>.jsonl` (JSON Lines;
   next to `~/.ynwa/games`, where the player writes game records) and returns `ScenarioReport` (as now).
6. The player opens the file in `--replay`: `create_football_replay_world` applies `initial_state` and
   replays the scenario.

`ynwa-player` requires no changes: `--replay <file>` already reads `Record` and builds the replay world via
`create_football_replay_world`; after the Stage 1–2 changes it automatically starts from `initial_state`.
All that remains is to manually verify opening a scenario record.

## Stage 1. `ynwa-core`: state serialization and record model

Goal: make `GameState` serializable, place it in `RecordHeader`, be able to apply the full
state, and write a ready `Record` to a file.

Changes:

- [`game.rs`](ynwa-core/src/game.rs:1):
  - [`PlayerState`](ynwa-core/src/game.rs:154), [`BallState`](ynwa-core/src/game.rs:183),
    [`RefereeState`](ynwa-core/src/game.rs:207), and [`GameState`](ynwa-core/src/game.rs:261) get
    `Serialize, Deserialize` and `PartialEq` (all fields are already serde-compatible: `Point3D`, `Velocity3D`,
    `Decision`, `Team`, `GameStage`). `PartialEq` is needed so `RecordHeader`/`Record` remain
    `PartialEq`.
  - [`StatSet`](ynwa-core/src/game.rs:31) gets `Serialize, Deserialize`; its `values`
    serialize deterministically via `sorted_collections::map`.
  - `GameState.team_stats: HashMap<Team, StatSet>` serializes deterministically (sorted by
    `Team`).
  - `sorted_collections::map` is generalized to `K: Ord + Serialize` (currently the key is fixed to `String`),
    to cover `HashMap<Team, StatSet>`; `sorted_collections::set` stays as is.
  - New method `Game::apply_state(&mut self, state: GameState) -> Result<(), String>`: checks
    that the lengths of `player_states`/`player_stats`/`referee_states` match `config` and that
    `ball_state.possessed_by` is in bounds, then replaces `self.state`.
- [`record.rs`](ynwa-core/src/record.rs:9): `RecordHeader { config, fixed_dt, initial_state }`;
  `RecordHeader::from_game` captures `game.state().clone()`.
- [`record_io.rs`](ynwa-core/src/record_io.rs:1): new
  `write_json_record(path, &Record) -> Result<(), String>` — opens `JsonRecordWriter`, writes
  the header, all events, the footer.

DOD: `cargo test -p ynwa-core` green; round-trip tests for `GameState` and nested types,
`StatSet`/`team_stats` determinism, round-trip of `Record` through JSON Lines; tests for
`Game::apply_state` (valid state and index validation errors).

## Stage 2. `ynwa-football`: replay from a snapshot

Goal: `create_football_replay_world` applies `initial_state` from the record.

Changes:

- [`replay.rs::create_football_replay_world`](ynwa-football/src/replay.rs:42): builds
  `Game::new(record.header.config, rng)`, then `game.apply_state(record.header.initial_state)?`;
  the system set does not change (`ReplaySetupBallPlacer` → `ReplayDriver` → `PhysicsSystem`).
  This repeats the two-phase scenario setup: first the world is assembled from the configuration (the scenario —
  from team settings), then the snapshot is applied on top. The difference is that the scenario snapshot
  is partial and merged on top, while the record snapshot is full (`GameState`) — it is already the result of both
  scenario phases, so `apply_state` replaces the state entirely rather than merging individual fields.
- Update [`replay_tests.rs`](ynwa-football/src/tests/replay_tests.rs:1) and
  [`lockstep_tests.rs`](ynwa-football/src/tests/lockstep_tests.rs:1) for the new `RecordHeader`
  (the `initial_state` field instead of `initial_stage`).
- New lockstep test: a run with a deliberately non-standard initial state (ball in the corner with initial
  velocity, players with initial velocities) is recorded with `initial_state`; replay starts from
  it and matches the original step by step (`run_lockstep` + `assert_equivalent`).

DOD: `cargo test -p ynwa-football` green; replay tests pass; the lockstep test confirms
equivalence for a non-standard start.

## Stage 3. `ynwa-integration-testing`: writing the run to a file and verification

Goal: a scenario run leaves a JSON Lines record readable by the player; automatic "record → replay → equivalence" verification.

Changes:

- [`runner.rs`](ynwa-integration-testing/src/runner.rs:22): `RunOutcome` instead of
  `journal: Vec<JournalEntry>` contains `record: Record` (in it both the journal and the header with
  `initial_state`). At the start of `run()` `RecordHeader::from_game(world.game(), run.dt)` is captured;
  after the loop — `finish_journal` and `collection.take_record(header)`.
- [`compare.rs::compare_outcome`](ynwa-integration-testing/src/compare.rs:35): the journal is taken from
  `outcome.record.journal`.
- [`scenario.rs::run_scenario`](ynwa-integration-testing/src/scenario.rs:19): after `run(...)`
  writes `outcome.record` to `~/.ynwa/scenarios/<scenario>.jsonl` via `write_json_record` (the directory
  is created if necessary; the write happens before expectation comparison, so the file exists even for
  a failed scenario).
- Tests:
  - unit: `run()` returns a `Record` whose `header.initial_state` corresponds to the initial
    world state, and whose `journal`/`total_steps` correspond to the run;
  - unit: the record file is read back by `json_journal_file_reader` into a `Record` with the same
    `initial_state`/`journal`/`total_steps`;
  - integration: for a real scenario (e.g., `stdlib_kick_to_goal`), the record from
    `run_scenario` is replayed by `create_football_replay_world` and yields a state equivalent to
    the original run (comparison with tolerance `TOLERANCE` under the replay contract, a local helper
    in tests).

DOD: `cargo test -p ynwa-integration-testing` green; JSON Lines scenario files appear in `~/.ynwa/scenarios/`;
the record is read by the player in `--replay` and replay is equivalent to the run.

## DOD Verification

- the scenario is written to JSON Lines and opened in the player `--replay` — Stage 3 and the note in
  "General Scheme";
- automatic verification of replay-from-snapshot equivalence to the original — the Stage 2 lockstep test
  (core/football mechanism) and the Stage 3 integration test (the "scenario → record → replay" chain).

## Architectural Decision Review

The review is complete, the architectural decision is agreed. Comments are considered minor and
are taken into account during development:

- In addition to the listed changes, update tests referencing
  `RecordHeader.initial_stage`: `ynwa-core/src/tests/codec_tests.rs`,
  `ynwa-core/src/tests/journal_tests.rs`, `ynwa-core/src/tests/record_tests.rs`,
  `ynwa-football/src/tests/journal_integration_tests.rs`.
- Update `ynwa-integration-testing/src/tests/compare_tests.rs` for the replacement of
  `RunOutcome.journal` with `RunOutcome.record`.
- Make `write_json_record` public in `ynwa-core/src/record_io.rs` and re-export it
  from `ynwa-core/src/lib.rs`.
- Take the record path `~/.ynwa/scenarios/<scenario>.jsonl` from the home directory (`env_home`), as
  the player does for game records, rather than tying it to the current working directory or `CARGO_MANIFEST_DIR`.
- In `Game::apply_state`, additionally check the `team_stats` keys against the teams from
  `config` (or explicitly declare the caller's responsibility).
