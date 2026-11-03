# YNWA — Data Formats and Protocols

This document gathers in one place all the "data contracts": the JSON protocol between the game core and the
decision engine, the on-disk team storage format and what the repository actually reads, and the
integration-testing scenario file formats (§4).
General project context — in `context.md`.

---

## 1. JSON Protocol `ynwa-core` ↔ `ynwa-decisions`

`ynwa-decisions` is a game-independent library: it does not know the core's domain types and communicates with it
only via `serde_json::Value`. The protocol consists of three parts: configuration (once at
engine creation), context (per call), and decision (the return from Lua). On the core side the protocol
is implemented by `ScriptedDecisionMaker` (`ynwa-core/src/systems/decision/scripted_decision_maker.rs`),
on the engine side — by `DecisionEngine` (`ynwa-decisions/src/decision_engine.rs`).

### 1.1 Configuration (at engine creation)

```jsonc
{
  "team_preambles": { "team_a": "<lua>", "team_b": "<lua>" },
  "players": [
    {
      "script": "<lua or empty string>",
      "team": "team_a",              // key in team_preambles
      "static_data": {               // becomes the GAME_DATA global in the player's VM
        "zones": { "goal_a": {...}, "penalty_area_b": {...}, "center_circle": {...} },
        "field": { "width": 68.0, "length": 104.6, "columns": 26, "rows": 40 }
      }
    }
  ]
}
```

*Note:* the `static_data` field is deliberately duplicated for each player. This allows
transforming (mirroring along the X and Z axes) zone coordinates individually for each team. Thus,
both teams' scripts see the field from their own "home" perspective (where their own
goal is always behind), while keeping a single codebase.

Zone geometry is one of the variants:
`{"type":"rectangle","min_x","max_x","min_z","max_z"}`,
`{"type":"circle","center_x","center_z","radius"}`,
`{"type":"arc","center_x","center_z","radius","start_angle","end_angle"}` (degrees),
`{"type":"point","x","z","tolerance"}`.
The zone name in JSON = `name` + `_a`/`_b` suffix for team zones (neutral zones — without a suffix).

### 1.2 Context (every call, the Lua global `context`)

```jsonc
{
  "me": {
    "team": "A",                 // "A" | "B"
    "number": 10,                // tactical number within the team
    "index": 5,                  // global index in the players array (A: 0..10, B: 11..21)
    "position": {"x":..,"y":..,"z":..},
    "regions": {
      "start":   {"min_x":..,"max_x":..,"min_z":..,"max_z":.., "display_notation":"M3"},
      "attack":  {...}, "defence": {...}, "goal kick own": {...}, ...
    }
  },
  "teammates": [ {"index":0,"number":1,"position":{...}}, ... ],   // excluding self
  "opponents": [ {"index":11,"number":1,"position":{...}}, ... ],
  "ball": {
    "position": {"x":..,"y":..,"z":..},
    "owner_index": 5,            // null if the ball is free
    "owner_team": "A"            // "A" | "B" | "None"; preserved during passes
  },
  "game": { "elapsed_time": 125.5 }
}
```

All coordinates and regions are already brought into the player's team perspective.

The region's `display_notation` field: for Team A — ordinary notation (`"M3"`), for Team B —
`"display (team)"`, e.g. `"R42 (M3)"`, so logs/UI show both the absolute and the "own"
coordinate.

*Performance note:* although a player's regions and metadata are stable during a match,
they are passed on every tick as part of `context` to simplify the Lua-script API.
In the future this may be optimized (splitting into static and dynamic contexts)
if profiling reveals serialization problems.

### 1.3 Decision (the return from Lua)

```lua
{action = "stop"}
{action = "run", target_type = "point",  target = {x = .., z = .., y = 0}}
{action = "run", target_type = "cell",   target = "C7"}
{action = "run", target_type = "region", target = {from = "A5", to = "C7"}}
{action = "run", target_type = "ball"}                 -- the target field is not needed
{action = "kick", target = {x = .., z = .., y = 0}}
```

Any decision may contain an optional string field `reason` — it is stored in
`player_state.decision_reason` and shown in the UI/logs. It is the main behavior-debugging tool,
and should always be filled in.

The format is validated in `ynwa-decisions` by the `LuaDecision` schema (`lua_format.rs`), and translation into the core's domain
types is done by `decision_parser.rs` in `ynwa-core`. Grid bounds are not checked in the parser —
an invalid region surfaces later, when computing the center.

---

## 2. On-Disk Team Data Format (`teams/<team_id>/`)

A team is a directory with Lua tactics and player subdirectories. This is exactly the structure that
`FsTeamRepository` reads (see §3). Player directories are sorted by name; the order determines the global
player index.

```
meta.toml        # the team's display name (currently not read by the repository)
preamble.lua     # team tactics: the team_play table (and common tables like goalkeeper_play)
players/NN/
  static.toml    # name, reaction_rate, speed_rate, tackle_rate, shot_power, shot_accuracy (10..100)
  tactical.toml  # number, [play_positions], [set_piece_positions]
  script.lua     # optional: player_play
```

### `tactical.toml`

```toml
number = 1

[play_positions]
attack  = "N9"
defence = "N1"

[set_piece_positions]
"kick off own"                = "N3"
"kick off opp"                = "N3"     # additionally registered as "start"
"goal kick own"               = "on_ball"
"goal kick opp"               = "N5"
"corner own left"             = "N34"
"corner own right"            = "N34"
"corner opp left"             = "N3"
"corner opp right"            = "N3"
"throw in own left own half"  = "N3"
"throw in own left opp half"  = "N3"
"throw in own right own half" = "N3"
"throw in own right opp half" = "N3"
"throw in opp left own half"  = "N3"
"throw in opp left opp half"  = "N3"
"throw in opp right own half" = "N3"
"throw in opp right opp half" = "N3"
```

Rules:

- The value is a cell/region notation (`"K7"`, `"A1:B2"`) or the `"on_ball"` marker
  (the player is the set-piece taker; see `context_football.md`).
- All 16 set-piece keys are mandatory; `"on_ball"` is allowed only in `own` keys
  and for exactly one player per key. Validation is performed at startup (`ynwa-football`).
- Positions are written in the team's own perspective; for Team B the engine mirrors them at load.
- `number` — the tactical number (1…N, no gaps within a team). Shirt numbers are
  a separate future entity.
- The `[play_positions]` keys (`attack`, `defence`) are used only by scripts through
  `my_regions()`.

---

## 3. What `ynwa-repository` Reads

`FsTeamRepository::load_team(team_id)` reads the `<base>/<team_id>/` directory and turns it into the core's
DTOs (`TeamRecord` / `PlayerRecord`, defined in `ynwa-core/src/repository.rs`):

- `preamble.lua` — required → `TeamRecord.preamble`;
- `players/NN/static.toml` — required → `PlayerStatic` (name and 5 attributes);
- `players/NN/tactical.toml` — required → `PlayerTactical { number, play_positions,
  set_piece_positions }`; keys of both position maps are strings, the sport layer assigns their meaning;
- `players/NN/script.lua` — optional (`None` ≡ an empty script; the player plays entirely by the team's
  tactics);
- an empty player list — an error;
- `meta.toml` is currently not read.

---

## 4. Integration-Testing Scenario File Formats

A `ynwa-integration-testing` scenario is a directory with three TOML files and a team set. All files
are parsed by serde structs with `deny_unknown_fields`: an unknown key is a load error. Value encodings:
`Point3D` — `{x, y, z}` in meters, `Velocity3D` — `{x, y, z}` in m/s, `Team` — `"A"`/`"B"`.
The initial and final states use **the same partial format**: a missing field
on input means "do not set", on output — "do not check".

How to assemble a scenario end to end — in [`integration_testing.md`](integration_testing.md).

### 4.1 `initial_state.toml` — partial initial state

```toml
stage = "Play"                  # optional: "Play" | "Setup" | "GameOver"; default Setup("kick off")
setup_reason = "kick off"       # meaningful only with stage = "Setup" (or without stage)

[ball]                          # optional
position = { x = 34.0, y = 0.0, z = 99.0 }
velocity = { x = 0.0, y = 0.0, z = 20.0 }
possessed_by = { team = "A", number = 9 }   # or "none" — the ball is free
last_possessing_team = "A"                  # or "none"

[setup]                         # meaningful only with stage = "Setup"
restart_position = { x = 34.0, y = 0.0, z = 5.5 }
restart_team = "A"              # or "none"

[[players]]                     # optional list of exact positions in meters
team = "A"
number = 9
position = { x = 34.0, y = 0.0, z = 10.0 }
```

Rules:

- if `stage` is absent — the world starts in `Setup("kick off")`; `setup_reason` is honored both with an explicit
  `stage = "Setup"` and without `stage`.
- when starting in `Setup` (`stage` absent or `"Setup"`), setting `[ball]` is a **load error**:
  the first `FootballGameManager` tick unconditionally overwrites the ball. Ball control in `Setup` —
  only via `[setup]`.
- randomness is not configurable: the runner strictly uses a deterministic RNG (`temperature = 0.0`,
  fixed `seed`).
- `[[players]]` overrides only the initial position `PlayerState.position`; the tactical
  regions, the `set_piece_positions` keys, and the taker roles from `tactical.toml` do not change and remain visible
  to scripts. A player is addressed by `{team, number}` (number — from `tactical.toml`); the number must be
  positive. Players not listed get the factory position.

### 4.2 `scenario.toml` — run plan `[run]`

```toml
[run]
dt = 0.1                # step length, required, a positive finite number

[[run.stop]]            # typed list; ANY criterion stops the run
when = "stage"          # stage change to the target
stage = "Setup"         # "Play" | "Setup" | "GameOver"
setup_reason = "kick off"   # optional: the exact Setup reason

[[run.stop]]
when = "event"          # event occurrence
event = "Goal"          # Goal | Touchline | GoalLine | GameEnd
team = "B"              # optional; forbidden for GameEnd

[[run.stop]]
when = "steps"          # step limit — a safety criterion
steps = 1000

# [[run.stop]]
# when = "time"         # model-time limit — a safety criterion
# time = 60.0
```

Criterion types (`when`): `"stage"`, `"event"`, `"steps"`, `"time"`.

- `stage` — `stage` + optional `setup_reason`; `Setup` without `setup_reason` matches any
  reason.
- `event` — `event` + optional `team`; `GameEnd` carries no team, the pair `GameEnd` + `team` —
  a load error.
- `steps` — step limit; `time` — model-time limit.

At least one safety criterion `steps` or `time` is required (otherwise a load error) — protection
against infinite loops. Criteria are checked after each step in declaration order; the first triggered
one goes into `StopReason`.

### 4.3 `scenario.toml` — expectations `[expect]`

Each item is optional; only the listed fields are checked.

```toml
[expect]
journal_match = "exact"     # "exact" (default) | "subsequence"

[expect.stop]               # optional
when = "event"
event = "Goal"
team = "B"
steps = 15                  # optional: the exact step count (for stage/event/time)

[[expect.journal]]          # optional; one entry per line
type = "kick_outcome"
player = { team = "A", number = 1 }
at = 1.5                    # optional: absolute timestamp (with tolerance)
```

`[expect.stop]` — by which criterion the run finished (`when` and the fields are the same as for `[[run.stop]]`),
plus an optional exact step count `steps` for `stage`/`event`/`time`; for `when = "steps"`
the `steps` value is both the limit and the expected step count.

`[[expect.journal]]` — one expected journal entry, discriminator `type`:

| `type` | Optional fields |
|---|---|
| `decision_assigned` | `player = {team, number}`, `decision`, `reason`, `at` |
| `possession_change` | `possessed_by = {team, number} \| "none"`, `last_possessing_team`, `at` |
| `kick_outcome` | `player = {team, number}`, `ball_velocity`, `at` |
| `stage_change` | `stage` (+ `setup_reason` for `Setup`), `at` |
| `restart_set` | `position`, `team`, `at` |
| `decisions_reset` | `at` |
| `stat_update` | `team`, `key`, `delta`, `at` |
| `football_event` | `event` (required: `Goal`/`Touchline`/`GoalLine`/`GameEnd`), `team`, `at` |

`decision` in `decision_assigned` is specified by the decision kind (`"Stop"` / `"Run"` / `"Kick"`); the decision
target is not compared. Players are everywhere addressed by `{team, number}` (number — `tactical.toml.number`);
the runner resolves the pair to a global index by searching `config.players`, a duplicate or absence —
a load error. Possession/team fields (`possessed_by`, `last_possessing_team`, `restart_set.team`)
accept `{team, number}` / `"A"` / `"B"` or `"none"`.

`journal_match` specifies how `[[expect.journal]]` is compared:

- `"exact"` (default) — full match of length and order;
- `"subsequence"` — the listed entries appear in the actual journal in the specified order,
  the rest are ignored.

Comparison: types and order — exact; integers, enums, strings — exact; float fields (`Point3D`,
`Velocity3D`, `delta`) — with tolerance `TOLERANCE` (`1e-4`); `timestamp` (`at`) — with tolerance
`max(dt, 1e-4)`. `at` is the absolute timestamp from the run start (the run always starts at zero
model time).

### 4.4 `final_state.toml` — expected final state

The same partial format as `initial_state.toml`, but with the "**missing field = do not check**" semantics;
a present field is compared (coordinates — with tolerance). Additionally, the score is specified. The file
is optional: if absent, the final state is not checked.

```toml
stage = "Setup"                # optional
setup_reason = "kick off"      # optional; with stage = "Setup" or without
score = { A = 1, B = 0 }       # optional: score by team

[ball]                         # optional
position = { x = 34.0, y = 0.0, z = 10.0 }
velocity = { x = 0.0, y = 0.0, z = 0.0 }
possessed_by = { team = "A", number = 9 }   # or "none"
last_possessing_team = "A"                  # or "none"

[setup]                        # optional
restart_position = { x = 34.0, y = 0.0, z = 5.5 }
restart_team = "B"             # or "none"

[[players]]                    # optional
team = "A"
number = 9
position = { x = 34.0, y = 0.0, z = 10.0 }
```

`stage = "Setup"` without `setup_reason` — only the `Setup` stage is checked (any reason); with
`setup_reason` — the exact reason. `score` is compared against the `"score"` value in each
team's `team_stats` (with tolerance). Player and ball positions — with tolerance `TOLERANCE`; ball velocity — also with
tolerance.
