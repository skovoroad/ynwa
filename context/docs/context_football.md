# Football Layer (`ynwa-football`)

This document describes everything football-specific in the project: field geometry, match stages and
set pieces, event detection (goal, out of play, end of match), and ready-made world assembly.
The core (`ynwa-core`) knows nothing about football — all this logic lives in the separate crate
`ynwa-football` and hooks into the core at two points: as the `World` factory and as the first system
in the step pipeline (`FootballGameManager`, see `context_game_step.md`).

Companion documents: `context.md` (general context), `context_game_step.md` (the structure of one
simulation step), `protocols.md` (team data formats).

---

## 1. Responsibility and Decisions Made

The crate is responsible for football rules on top of the neutral core: it builds the field with zones, manages
match stages (setup / play / end), detects events and reacts to them, and assembles a
ready-made `World` from team data.

Decisions made:

- The core must not know about football — so this whole layer is moved to a separate crate; for another
  sport an analogous crate is written.
- Field and `GameConfig` creation are internal details; only `World` is exposed.
- The crate depends on the **trait** `TeamRepository`, not on `ynwa-repository`: the client injects the
  concrete implementation.
- All 16 set-piece keys are mandatory for every player and validated at startup:
  it is better to fail at load time than to get a player without a position mid-match.

---

## 2. Core Concepts and Terminology

- **Teams `"team_a"` and `"team_b"`** — text identifiers of the two competing teams,
  whose configuration directories (`preamble.lua`, `players/`) are loaded from the repository.
- **Set-Piece Keys** — 16 string identifiers encoding situations of putting the ball into play. They
  select tactical regions from players' `tactical.toml` files for positioning in the `Setup` phase. Every player
  must specify a position for all 16 keys.
- **The `"on_ball"` marker** — a special position value in `tactical.toml` indicating that this
  player is the taker (kicker) of the given set piece. Assigned instead of a region.
- **`on_ball` validation** — a startup check guaranteeing that for each "own"
  set piece (`own`) exactly one taker with `"on_ball"` is assigned, and that for the opponent's set pieces
  (`opp`) there are no such takers in our team.
- **"reason + state" → key mapping** — a function translating an abstract match-stoppage
  reason (e.g., a throw-in) and the current situation (who takes it, from which flank and half) into one
  of the 16 concrete set-piece keys for player positioning.

---

## 3. `lib.rs` — World Assembly

The crate's entry point is a factory that turns the team repository and the preamble directory into a
simulation-ready `World`:

`create_football_world(repo: &dyn TeamRepository, preambles_path: &Path) -> Result<World, String>`

Assembly order: builds the field, loads `"team_a"` and `"team_b"`, validates the key sets and
`on_ball` markings, converts records into `PlayerDef`, reads `core.lua`/`stdlib.lua`, creates
`Game` in the `Setup("kick off")` stage, and adds systems in the order described in
`context_game_step.md` §2.

Significant details:

- `build_player_defs` parses position notations and **mirrors Team B's regions** into absolute
  field coordinates (in TOML, Team B writes positions from its own perspective).
- A value of `"on_ball"` instead of a notation means the player is the taker of that set piece; the key
  goes into `set_piece_roles`, and no region is created for it.
- The `"kick off opp"` key is additionally registered as `REGION_START_POSITION` (`"start"`) —
  a temporary solution: all players wait there before the match starts, until the kicking-off
  team is chosen.
- `SET_PIECE_KEYS` — the 16 mandatory keys; `ON_BALL_REQUIRED_KEYS` — 8 `own` keys, in each
  of which exactly one player on the team must have `"on_ball"`; in `opp` keys `"on_ball"`
  is forbidden (the ball is with the opponent).
- If `ScriptedDecisionMaker` fails to be created, the decision system silently degrades to
  `PlaceholderDecisionMaker` with a warning to stderr.
- The initial ball position is taken from the `center_spot` zone.

---

## 4. `field_builder.rs` — Field and Zones

The field is dimensions, an addressing grid, and a set of named zones used by both the rules
(goal detection) and the scripts (via `GAME_DATA.zones`).

`create_football_field()` — a 68 × 104.615 m field, 26 × 40 grid (cell ≈ 2.615 m).
`create_football_field_with_dimensions(...)` allows other dimensions but **requires square
cells** (`width/columns == length/rows`), otherwise it returns an error — all region metrics in the core
are computed from a square cell.

19 zones are created: `field`, `half` (A/B), `goal_area` (A/B), `penalty_area` (A/B), `center_circle`,
`center_spot`, `penalty_arc` (A/B), `penalty_spot` (A/B), `corner_arc_bottom`/`corner_arc_top`
(A/B), `goal` (A/B). Goals are placed **outside** the field line (A: z ∈ [−2.5, 0], B: z ∈
[length, length+2.5]) — that is how a goal is detected. The penalty-arc angles are computed as the
intersection of a circle of radius 9.15 m around the penalty spot with the penalty-area line.

---

## 5. `game_manager.rs` — `FootballGameManager`

The only system aware of stages: it switches `GameStage`, hands out decisions during
setup, and processes events. It runs first in the step pipeline, so an event takes
effect before players start making decisions (more in `context_game_step.md` §3).

Behavior by stage:

- **`Setup(reason)`**: every tick the ball is forced into `restart_position` (or into the initial
  ball position if none is set), velocity and possession are cleared, `last_possessing_team`
  is reset. Then:
  - `assign_setup_decisions` — every player without a decision gets a `Run`: the set-piece taker —
    to the ball position, the rest — to their region for the corresponding key. If there is no region,
    `last_error` is written and the player stays in place. Lua is **not** called.
  - `check_player_readiness` — a player is considered ready (`is_ready`) when their current decision
    has become `Stop` (that is, `DecisionSystem` recorded arrival).
  - When everyone is ready — the stage switches to `Play`.
- **`Play`**: calls `check_events` and processes the first found event.
- **`GameOver`**: does nothing.

### Event Handling (`handle_event`)

| Event | Consequence |
|---|---|
| `Goal(team)` | `team` — the owner of the goal that was scored into, so the score is incremented for `team.opposite()`. Transition to `Setup("kick off")`, `restart_position = None` (ball at center), `restart_team` = the conceding team. |
| `Touchline(pos, last)` | `Setup("throw in")`, ball at the intersection point, `restart_team = last.opposite()`. |
| `GoalLine(pos, last)` | If the attacking team put the ball out — `Setup("goal kick")`, ball 5.5 m from the line at the center, taken by the defending team; otherwise — `Setup("corner")`, ball at the nearest corner flag, taken by the attacking team. |
| `GameEnd` | `GameOver`. |

On any transition to `Setup`, all players have `is_ready` and `current_decision` reset and
`needs_decision` raised.

Important decision: `restart_position`/`restart_team` are set in `handle_event`, before the `Setup`
ticks begin — otherwise they would be wiped by the `last_possessing_team` reset that happens every `Setup` tick.

### Set-Piece Key Selection

`resolve_set_piece_key(reason, player_team, restart_team, restart_position, w, l)` — the
"reason + state" → one-of-16-keys mapping. The distinguishing logic:

- `own`/`opp` — whether `restart_team` matches the player's team;
- `left`/`right` — from the player's perspective (Team A's left side — small X, Team B's — large X);
- `own half`/`opp half` — in whose half the ball is (A defends `z < length/2`, B — `z > length/2`).

An unknown reason yields `None`, which the caller treats as an error.

---

## 6. `events.rs` — Event Detection

Events are the only way the rules "notice" that the game must stop.
The `FootballEvent` type — `Goal(Team)` | `Touchline(Point3D, Team)` | `GoalLine(Point3D, Team)` |
`GameEnd`. `check_events` checks in priority order: goal → end of match → touchline out →
goal-line out.

Rule details:

- goal: the ball has **fully** crossed the line along Z (the 0.11 m radius is accounted for), while along X only the
  ball's center is considered — touching the post from inside counts as a goal, from outside it does not;
- `check_goal_line` does not fire if the ball is between the posts (that case is handled by `check_goal`);
- outs require full line crossing accounting for the radius;
- the team that last touched the ball is taken from `last_possessing_team`, and when `None` — `Team::A`
  (a crude stub);
- `GAME_DURATION = 120.0` seconds — a temporary debug value, not a full match.

---

## 7. Known Football Discrepancies and Unfinished Places

- **Initial positioning in `Setup`.** A comment in `game_manager.rs` says
  `(width/2, 0, −5)`, but actually `Game::with_stage` places players at `(−5, 0, length/2)` — beside
  the field.
- **`GAME_DURATION = 120 s`** despite a "1 minute" comment — a debug value.
- **`REGION_START_POSITION` = the `"kick off opp"` alias** — a temporary solution; choosing the kicking-off
  team at match start is not yet implemented.
- **Last ball touch** is determined via `last_possessing_team` (possession), not via
  an actual touch; when `None`, `Team::A` is substituted.
