# Recording and Replay Specification

## Purpose

Provides deterministic match recording and physics-only replay for three use cases: (1) user playback of played matches in `ynwa-player`, (2) debugging match runs (including crashed or interrupted sessions via prefix-recoverable JSON Lines), and (3) automated integration-scenario verification with on-demand visual inspection of runs in `ynwa-player`.

Instead of per-frame state dumps, a `Record` stores `RecordHeader { config, initial_state, fixed_dt }` plus a timestamped `JournalEntry` stream of non-recomputable events. `config` defines invariant world rules and entities (sufficient alone for a fresh kickoff), while `initial_state` overlays the exact starting `GameState` snapshot so recordings and test scenarios beginning from arbitrary mid-match states reconstruct accurately.

## Requirements

### Requirement: Event-Sourced Record Model and JournalSink Lifecycle

Match recording SHALL store `Record { header: RecordHeader { config, initial_state, fixed_dt }, total_steps: u64, journal: Vec<JournalEntry> }` instead of per-frame dumps. Pluggable `JournalSink` recording MUST default to zero overhead when disabled, count steps once per `World::step`, and defer streaming I/O errors until journal finalization without aborting simulation steps.

#### Scenario: Zero-overhead default, pre-step header snapshot, and deferred I/O errors
- **WHEN** `RecordHeader::from_game(game, fixed_dt)` is captured before the first `World::step` and `N` steps execute
- **THEN** unrecorded matches incur zero allocation overhead, in-memory collection assembles a `Record` with `total_steps == N` on finalization, and streaming file recording flushes the header immediately and surfaces any write failure on `finish_journal()` rather than panicking mid-step

### Requirement: Complete 8-Variant JournalEvent Wire Contract

`JournalEvent` SHALL record all non-recomputable decisions (in Display Orientation), stochastic outcomes, stage/restart changes, stat deltas, and sport markers across 8 variants: `DecisionAssigned`, `PossessionChange`, `KickOutcome`, `StageChange`, `RestartSet`, `DecisionsReset`, `StatUpdate`, and `External`. `ynwa-core` MUST treat `External { kind, data }` as opaque; sport layers decode their own `kind` (e.g., `"football_event"`).

#### Scenario: Separating player kick intent from post-RNG ball trajectory and opaque sport events
- **WHEN** a player kicks the ball or a sport-specific event occurs during a recorded run
- **THEN** `DecisionAssigned` records the player's Display-Orientation `Kick` decision and `reason` (for UI/telemetry) while `KickOutcome` separately records the post-RNG `ball_velocity` so replay reproduces exact ball flight without RNG, and `External` stores sport events for UI/test decoding without affecting replay kinematics

### Requirement: Deterministic Serialization, Codec Traits, and Crash-Resilient JSON Lines

`GameConfig` and `GameState` SHALL serialize unordered maps, sets, and `Field` zones in deterministic sorted key order. `RecordWriter` and `RecordReader` MUST decouple `Record` from byte transport (supporting streaming files, in-memory buffers, and whole-`Record` dumps). `JsonRecordWriter` flushes newline-terminated JSON Lines (`header`, `event`..., `footer`); `JsonRecordReader` rejects missing/duplicate headers and ignores trailing lines without `'\n'`.

#### Scenario: Recovering valid prefix from crashed recording
- **WHEN** a `.jsonl` recording contains 1 `"header"` line (`config`, `initial_state`, `fixed_dt`), `K` newline-terminated `"event"` lines, and a truncated final line without `'\n'` (no `"footer"`)
- **THEN** `JsonRecordReader::read()` returns `Ok(Record)` containing the deterministically serialized header, all `K` complete events, and `total_steps == 0`

### Requirement: Replay Pipeline and Observable State Equivalence Contract

Replay world construction SHALL initialize `Game` from `record.header.config`, overlay `record.header.initial_state`, and run physics-only re-simulation driven by due journal entries (`entry.timestamp <= current_timestamp`, plus `Setup` ball pinning) without Lua, decision systems, sport manager, or RNG. Stepping `total_steps` times at `fixed_dt` on the same platform MUST reproduce all observable `GameState` fields within `1e-4`.

#### Scenario: Observable state equivalence vs excluded internal coordination flags
- **WHEN** a match recorded on a platform with `fixed_dt` is replayed step-by-step with `fixed_dt` on the same platform
- **THEN** at every step, `elapsed_time`, `stage`, `restart_position`, `restart_team`, `team_stats`, `player_stats`, all `BallState` fields, and `player_states[i].{position, velocity, current_decision, decision_reason}` match the recorded run within `1e-4`, and `check_events` evaluated before each `Play` step matches `decode_football_events(&record.journal)`
- **AND** internal live-pipeline coordination flags `player_states[i].{needs_decision, is_ready, last_error, decision_processed, last_decision_time}` are excluded from replay equivalence guarantees
