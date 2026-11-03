# YNWA — Guide to Writing Integration Tests

This document explains how to write an integration test of the whole "core — standard library —
player scripts" chain as a granular scenario. The exact file formats and all allowed fields —
in [`protocols.md`](protocols.md) §4; here — how a scenario is structured, how to run it, and by which
rules it is executed.

General project context — in [`context.md`](context.md).

---

## 1. What It Is and Why

`ynwa-integration-testing` runs a football match in deterministic mode and compares the facts with
expectations. Unlike `ynwa-script-tests`, the check is automatic: there is no need to watch the
player visually — the scenario either passes or `cargo test` prints a readable diff (expected/actual).

A scenario checks three things:

- **the event journal** — which decisions players made, who took possession, at what speed the
  ball flew after a kick, stage changes, and football events (`Goal`, `Touchline`, `GoalLine`, `GameEnd`);
- **the stop criterion** — for what reason the run ended and how many steps it took;
- **the final state** — stage, player positions, ball position and possession, score,
  `restart_position`/`restart_team`.

The run is fully reproducible: the runner uses a fixed RNG seed and zero temperature
(`temperature = 0.0`), so the result does not change from run to run.

---

## 2. Running

```sh
cargo test -p ynwa-integration-testing
```

The harness `tests/scenarios.rs` walks the `ynwa-integration-testing/scenarios/` directory and runs
each subdirectory containing a `scenario.toml`. To debug one scenario:

```sh
YNWA_SCENARIO=goal_into_b cargo test -p ynwa-integration-testing
```

An unknown name — an error listing the available scenarios.

---

## 3. Scenario Directory Structure

```
ynwa-integration-testing/scenarios/<name>/
  initial_state.toml   # partial initial state (input)
  scenario.toml        # run plan [run] + expectations [expect]
  final_state.toml     # expected final state (optional)
  teams/
    team_a/
      preamble.lua
      players/NN/{static.toml, tactical.toml, script.lua}
    team_b/
      ...
```

- `teams/` — ordinary teams in the [`protocols.md`](protocols.md) §2–3 format; they are read by
  the existing `FsTeamRepository`. Roster, attributes, regions, and tactics are set by the same
  mechanisms, without duplication.
- the core and standard-library preambles (`core.lua`, `stdlib.lua`) are taken from
  `ynwa-scripts/preambles`, as in a normal world build.

A scenario specifies not the full state but a **partial** one: only the needed fields are listed, the rest
are filled with deterministic defaults (missing fields are added as
needed).

---

## 4. How to Create a Scenario: Step by Step

### 4.1 Teams (`teams/`)

Create minimal `team_a` and `team_b`: each with a `preamble.lua` and at least one player with
`static.toml` and `tactical.toml`. For most scenarios one or two players per
team suffice; `script.lua` is optional.

### 4.2 Initial State (`initial_state.toml`)

Set the starting stage and ball/player positions sufficient to provoke the needed
situation:

```toml
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 99.0 }
velocity = { x = 0.0, y = 0.0, z = 20.0 }
last_possessing_team = "A"
```

Rules that matter when writing:

- `stage` may be `"Play"`, `"Setup"` (with an optional `setup_reason`), or `"GameOver"`; if
  the stage is absent, the world starts in `Setup("kick off")`.
- when starting in `Setup`, the `[ball]` fields must **not** be set (a load error): the first
  `FootballGameManager` tick unconditionally overwrites the ball. Ball control in `Setup` — only via
  `[setup]` (`restart_position`, `restart_team`).
- `[[players]]` sets exact positions in meters and overrides only the player's initial position;
  the tactical regions, set-piece keys, and roles from `tactical.toml` remain untouched
  and still visible to scripts. A player is addressed by the pair `{team, number}` (number — from
  `tactical.toml`); players not listed get the factory position.
- randomness is not configurable: the runner always uses a deterministic RNG.

### 4.3 Run Plan (`scenario.toml`, `[run]` section)

```toml
[run]
dt = 0.1                # step length, required

[[run.stop]]
when = "event"          # stop on an event
event = "Goal"
team = "B"

[[run.stop]]
when = "steps"          # safety step limit
steps = 1000
```

Stop criteria — a typed list; the run stops at the **first** triggered one (in
declaration order):

- `when = "stage"` — stage change to the target (`stage`, optional `setup_reason`);
- `when = "event"` — occurrence of a football event (`event`, optional `team`);
- `when = "steps"` — step limit reached;
- `when = "time"` — model time reached.

At least one safety criterion `steps` or `time` is required — protection against infinite loops. Criteria
are checked after each `world.step(dt)`.

### 4.4 Expectations (`scenario.toml`, `[expect]` section)

Each item is optional; only the listed ones are checked.

```toml
[expect]
journal_match = "subsequence"   # "exact" (default) | "subsequence"

[expect.stop]
when = "event"
event = "Goal"
team = "B"

[[expect.journal]]
type = "kick_outcome"
player = { team = "A", number = 1 }

[[expect.journal]]
type = "football_event"
event = "Goal"
team = "B"

[[expect.journal]]
type = "stage_change"
stage = "Setup"
setup_reason = "kick off"
```

- `[expect.stop]` — by which criterion the run ended; for `stage`/`event`/`time` one can
  additionally specify the exact step count `steps`.
- `[[expect.journal]]` — expected journal entries; the type and order are compared, and for each entry
  only the specified fields are checked. Players are addressed by `{team, number}`.
- `journal_match` specifies the comparison mode: `"exact"` — length and order match fully;
  `"subsequence"` — the listed entries occur in the actual journal in the specified order,
  the rest are ignored. The `"subsequence"` mode is convenient for scenarios starting in `Setup` — there is no need
  to list the service positioning entries.

Comparison with tolerances: coordinates and velocities — with tolerance `TOLERANCE` (`1e-4`), `timestamp` — with
tolerance `max(dt, 1e-4)`; integers, enums, and strings — exact.

### 4.5 Expected Final State (`final_state.toml`)

The file is optional. The format is the same partial snapshot, but with the "**missing field = do not
check**" semantics; a present field is compared (coordinates — with tolerance). Additionally, the score
is specified:

```toml
stage = "Setup"
setup_reason = "kick off"
score = { A = 1, B = 0 }
```

---

## 5. What Else Is Checked Automatically

- **Strict build.** Any error initializing the decision engine, reading a script or preamble —
  a scenario load error, not a silent fallback to a stub. The test world is built by the strict
  `FootballWorldBuilder::build()`, without a placeholder fallback.
- **Script errors.** After the run, the runner checks that no player has
  `player_state.last_error` set; if any do, the scenario is considered failed.
- **Reference resolution.** Each `{team, number}` pair resolves to a global index by searching
  `config.players`; a duplicate or a missing player — a load error.

---

## 6. Ready Examples

- `scenarios/goal_into_b` — the ball in Team B's goal during the play stage: a goal and transition to
  `Setup("kick off")`, score `A = 1, B = 0` are checked.
- `scenarios/goal_kick` — the ball goes out over the goal line: the `GoalLine` event and transition to
  `Setup("goal kick")` with `restart_team = "A"` are checked.
- `scenarios/throw_in` — the ball goes out over the touchline: the `Touchline` event and transition to
  `Setup("throw in")` are checked.
- `scenarios/stdlib_kick_to_goal` — standard-library check: a Team A player shoots at
  goal (`kick_outcome`), a goal into B's goal; exact player positions are set in meters.
