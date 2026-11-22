# Lua Scripts: Player Reference

Development-stage documentation: a scripting reference for those configuring
players and teams. Full data formats (`static.toml`, `tactical.toml`, `context`, `GAME_DATA`,
decision) — [`protocols.md`](protocols.md); general context — [`project_overview.md`](project_overview.md);
script internals — [`scripting_developer.md`](scripting_developer.md).

IMPORTANT: user-authored Lua scripts are an intermediate state. In the future a dedicated user language will be developed and transpiled to Lua. Therefore, at the development stage many capabilities of user Lua scripts (for example, direct calls to core.lua functions) are not restricted: they will not be available to the user in the real user programming language. Nevertheless, the separation of abstraction levels must be observed.

## 1. Field, Coordinates, and Grid

Briefly:

- Right-handed coordinate system: **X** — field width, **Z** — field length; the own goal is always "behind"
  (smaller Z), the opponent's goal "ahead" (larger Z).
- The script always sees the field **from its own team's side**; positions in files are written from its own
  perspective, and for Team B the engine mirrors them automatically. Therefore the same code works
  for both teams.
- Grid: columns along X in Excel notation (`A`…`Z`, then `AA`), rows along Z, 1-based numbering.
  A cell is `"M22"`, a region is `"A1:B2"`.

## 2. Player Attributes and Positions

**Static attributes** (`static.toml`):

| Field | Range |
|---|---|
| `name` | string |
| `reaction_rate` | 10–100 |
| `speed_rate` | 10–100 |
| `tackle_rate` | 10–100 |
| `shot_power` | 10–100 |
| `shot_accuracy` | 10–100 |

**Tactical data** (`tactical.toml`):

| Field | Meaning |
|---|---|
| `number` | tactical number (1…N) |
| `[play_positions]` | `attack`, `defence` — regular positions (cell/region notation) |
| `[set_piece_positions]` | 16 set-piece keys (see below) |

**Set-piece positions** — the 16 mandatory `[set_piece_positions]` keys:

```
"kick off own", "kick off opp",
"goal kick own", "goal kick opp",
"corner own left", "corner own right",
"corner opp left", "corner opp right",
"throw in own left own half", "throw in own left opp half",
"throw in own right own half", "throw in own right opp half",
"throw in opp left own half", "throw in opp left opp half",
"throw in opp right own half", "throw in opp right opp half"
```

A key's value is a cell/region notation or the `"on_ball"` marker (this player is the set-piece
taker; allowed only in `own` keys and exactly once per key). The
`"kick off opp"` key is additionally registered as the `start` region.

## 3. Available Functions

Only the functions listed in this section are part of the user-facing API. Everything else in
`core.lua` and `stdlib.lua` is internal to the engine preambles and must not be called from
team preambles or player scripts.

Squad numbers identify players for now; a dedicated DSL "player" type will replace them later.

### 3.1 State Access ([`core.lua`](../ynwa-scripts/preambles/core.lua))

| Function | Returns |
|---|---|
| `my_position()` | table `{x, y, z}` — my position |
| `ball_position()` | table `{x, y, z}` — ball position |
| `my_team_name()` | `"A"` or `"B"` |
| `get_ball_owner_team()` | `"A"`, `"B"`, or `"None"` |

### 3.2 Queries ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Function | Parameters | Returns |
|---|---|---|
| `distance(p1, p2)` | two positions | horizontal distance (without Y) |
| `teammate_position(n)` | squad number | teammate position (opaque `{x, y, z}`) |
| `opponent_position(n)` | squad number | opponent position (opaque `{x, y, z}`) |
| `ball_owner_number()` | — | squad number of the ball owner, or `nil` when nobody possesses the ball |
| `count_teammates_around(pos, r)` | position, radius | number of teammates (including me) strictly closer than `r` |
| `count_opponents_around(pos, r)` | position, radius | number of opponents strictly closer than `r` |

### 3.3 Condition Checks ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Function | Parameters | Returns |
|---|---|---|
| `is_in_region(from, to)` | corner notations | `true` if I am inside the region `from:to` |
| `is_in_opponent_penalty_area()` | — | `true` if I am in the opponent's penalty area |
| `is_teammate_free(n)` / `is_opponent_free(n)` | squad number | `true` if no opponent / teammate is strictly within 5 m |
| `is_teammate_ahead(n)` / `is_teammate_behind(n)` | squad number | teammate is more than 0.5 m closer to the opponent goal / own goal along Z |
| `is_teammate_left(n)` / `is_teammate_right(n)` | squad number | teammate is more than 0.5 m to the left / right along X |
| `is_opponent_ahead/behind/left/right(n)` | squad number | the same checks for an opponent |

### 3.4 Decision-Making Functions ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Function | Parameters | Action |
|---|---|---|
| `stop(reason?)` | reason | stop in place |
| `chase_ball()` | — | run after the ball |
| `run_to_region(from, to)` | corner notations | run to the center of the region |
| `run_to_start_position()` | — | run to the center of the `start` region |
| `run_to_attack_position()` | — | run to the center of the `attack` region |
| `run_to_defence_position()` | — | run to the center of the `defence` region |
| `run_to_opponent_penalty_area()` | — | run to the center of the opponent's penalty area |
| `kick_to_position(pos, reason?)` | opaque position, reason | kick to that position |
| `kick_to_opponent_goal()` | — | kick at the opponent's goal |
| `default_goalkeeper_cover_position()` | — | take a position on the goal line: Z from the `defence` region, X follows the ball clamped by the goal width |
| `default_goalkeeper_with_ball()` | — | pass to the nearest teammate at least 15 m away, otherwise kick at the opponent's goal |

## 4. Team Preamble and `team_play`

[`preamble.lua`](../teams/team_a/preamble.lua) — team tactics. Defines the `team_play` table
(default behavior for all players) and common tables (e.g., `goalkeeper_play`).

`team_play` is a table whose keys are ball-possession states:

| State | Meaning |
|---|---|
| `"i_have_ball"` | I have the ball |
| `"ball_is_free"` | the ball is neutral (`owner_team == "None"`) |
| `"team_has_ball"` | my team has the ball (not me) |
| `"opponent_has_ball"` | the opponent has the ball |

Brief example:

```lua
team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
```

The `make_decision()` dispatcher is defined in stdlib and calls the handler in the order
`player_play[state] → team_play[state] → error()`.

## 5. Player Script and `player_play`

[`script.lua`](../teams/team_a/players/10/script.lua) — the optional player script. Defines
the `player_play` table — a partial override: specified states replace `team_play`,
unspecified ones fall back to `team_play`.

Brief examples:

```lua
-- inherit a ready common table
player_play = goalkeeper_play

-- or override individual states
player_play = { i_have_ball = forward_with_ball }
```

## 6. Regions and Field Zones

Region and zone objects are internal to the engine preambles: team preambles and player
scripts never receive or inspect them directly. Use the public functions instead:

- tactical regions: `run_to_attack_position()`, `run_to_defence_position()`,
  `run_to_region(from, to)`;
- the opponent penalty area: `is_in_opponent_penalty_area()`,
  `run_to_opponent_penalty_area()`;
- positions: `my_position()`, `ball_position()`, `teammate_position(n)`,
  `opponent_position(n)`. Their `{x, y, z}` tables are opaque; compare them only through
  the public predicates and `distance()`.

Region names such as `start`, `attack`, `defence`, and the set-piece keys come from
`tactical.toml` (see section 2) and back the tactical actions above.
