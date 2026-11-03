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

### 3.1 State Access ([`core.lua`](../ynwa-scripts/preambles/core.lua))

| Function | Returns |
|---|---|
| `ball_owner()` | global index of the ball owner or `nil` if the ball is free |
| `my_position()` | table `{x, y, z}` — my position |
| `my_regions()` | table `{name = region}` — my named regions |
| `ball_position()` | table `{x, y, z}` — ball position |
| `get_teammates()` | array of teammates `{index, number, position}` (excluding self) |
| `my_index()` | my global index |
| `my_team_name()` | `"A"` or `"B"` |
| `get_ball_owner_team()` | `"A"`, `"B"`, or `"None"` |
| `get_opponent_goal()` | the opponent's goal zone object |
| `get_own_goal()` | the own goal zone object |
| `get_opponent_penalty_area()` | the opponent's penalty-area zone object |

### 3.2 Utilities ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Function | Parameters | Returns |
|---|---|---|
| `distance(p1, p2)` | two positions | horizontal distance (without Y) |
| `parse_col(s)` | column string | column number (`A`→1, `Z`→26, `AA`→27) |
| `parse_notation(n)` | string `"M22"` | `col, row` |

### 3.3 Condition Checks ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

| Function | Parameters | Returns |
|---|---|---|
| `am_i_ball_owner()` | — | `true` if I have the ball |
| `is_in_region_obj(region)` | region | `true` if I am inside the region |
| `is_in_region(from, to)` | corner notations | `true` if I am inside the region `from:to` |
| `is_in_opponent_penalty_area()` | — | `true` if I am in the opponent's penalty area |
| `get_teammate_by_number(n)` | tactical number | teammate `{index, number, position}` or `nil` |

### 3.4 Decision-Making Functions ([`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua))

Primitive actions:

| Function | Parameters | Action |
|---|---|---|
| `stop(reason)` | reason | stop in place |
| `chase_ball()` | — | run after the ball |
| `run_to_region_obj(r, reason)` | region, reason | run to the center of the region object |
| `run_to_region(from, to)` | corner notations | run to the center of the region |
| `kick_to_cell(notation)` | cell | kick to the center of the cell |
| `kick_to_region(from, to)` | corner notations | kick to the center of the region |
| `kick_to_opponent_goal()` | — | kick at the opponent's goal |
| `pass_to_teammate(tm)` | teammate | pass to a teammate |
| `pass_to_players_by_numbers(numbers)` | array of numbers | pass to the nearest of the listed; if none — kick at goal |

Tactical actions (use the player's named regions):

| Function | Parameters | Action |
|---|---|---|
| `run_to_start_position()` | — | run to the center of the `start` region |
| `run_to_attack_position()` | — | run to the center of the `attack` region |
| `run_to_defence_position()` | — | run to the center of the `defence` region |
| `run_to_opponent_penalty_area()` | — | run to the center of the opponent's penalty area |
| `default_goalkeeper_cover_position()` | — | take a position on the goal line: Z from the `defence` region, X follows the ball clamped by the goal width |

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

**A player's region** in `my_regions()` is an object `{min_x, max_x, min_z, max_z, display_notation}`.
Keys: `start`, `attack`, `defence`, and the set-piece keys from `[set_piece_positions]`.
`display_notation` is the notation shown in the UI/logs.

**Field zones** live in `GAME_DATA.zones`; access them through the functions
`get_own_goal()`, `get_opponent_goal()`, `get_opponent_penalty_area()`. The zone name = name +
`_a`/`_b` suffix for team zones (neutral zones — without suffix).

Zone geometries:

- `rectangle`: `{type, min_x, max_x, min_z, max_z}`;
- `circle`: `{type, center_x, center_z, radius}`;
- `arc`: `{type, center_x, center_z, radius, start_angle, end_angle}` (degrees);
- `point`: `{type, x, z, tolerance}`.

19 field zones: `field`, `half` (A/B), `goal_area` (A/B), `penalty_area` (A/B), `center_circle`,
`penalty_arc` (A/B), `corner_arc_bottom`/`corner_arc_top` (A/B), `center_spot`,
`penalty_spot` (A/B), `goal` (A/B).
