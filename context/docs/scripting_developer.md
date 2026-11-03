# Lua Scripts: Developer Guide

Documentation for those developing the scripting part of the project: the core functions and the standard
library ([`ynwa-scripts/preambles/`](../ynwa-scripts/preambles)), team preambles, and player
scripts. How scripts are executed (VM, systems, timers) is not described here — see
[`context.md`](context.md) §4 and [`context_game_step.md`](context_game_step.md). The core ↔ decision-engine
exchange protocol — [`protocols.md`](protocols.md); here only references to it. The on-disk team data
format — [`protocols.md`](protocols.md) §2.

## 1. Script Levels and Their Responsibilities

| Level | File | Author | Responsibility |
|---|---|---|---|
| Core functions | [`core.lua`](../ynwa-scripts/preambles/core.lua) | project | Thin state access: wrappers over `context`/`GAME_DATA`. Hide the team binding (`own`/`opponent`). No game logic. |
| Standard library | [`stdlib.lua`](../ynwa-scripts/preambles/stdlib.lua) | project | Utilities, state queries, primitive and tactical actions, the [`make_decision()`](../ynwa-scripts/preambles/stdlib.lua:229) dispatcher. |
| Team preamble | `teams/<id>/preamble.lua` | team | Team tactics: the `team_play` table and common tactical tables (e.g., `goalkeeper_play`). |
| Player script | `teams/<id>/players/NN/script.lua` | team | Individual tactics: partial override via `player_play`. |

Teams do not modify `core.lua` and `stdlib.lua`; the team writes `preamble.lua` and `script.lua`.

IMPORTANT: user-authored Lua scripts (team preambles, player scripts) are an intermediate state. In the future a dedicated user language will be developed and transpiled to Lua. Therefore, at the development stage many capabilities of user Lua scripts (for example, direct calls to core.lua functions) are not restricted: they will not be available to the user in the real user programming language. Nevertheless, the separation of abstraction levels must be observed.

## 2. Loading, Perspective, State

Briefly, without derivation:

- Load order: `core.lua` + `stdlib.lua` + team `preamble.lua` + player `script.lua`.
- Preambles are executed once when the player's VM is created; the player script is reloaded on every
  call.
- Perspective: both teams' scripts are written identically — the input (`context`, `GAME_DATA`, regions)
  is mirrored to the player's team, the output (decision) is mirrored back. Team-specific data is accessible
  only through the `own`/`opponent` functions.
- State: each player has their own VM. Globals set by the preamble are preserved between
  calls within that player's VM; player-script globals are not.

## 3. Script Authoring Rules

1. Do not access `context` and `GAME_DATA` directly outside `core.lua` — only through wrapper functions;
   direct access by raw keys (`GAME_DATA.zones.goal_a`) is not portable between teams.
2. Do not override `make_decision()`.
3. Do not assemble `{action = ...}` tables by hand if a ready stdlib function exists.
4. Prefer point-free style: `team_has_ball = chase_ball`, not a lambda wrapper.

## 4. The `make_decision()` Dispatcher

[`make_decision()`](../ynwa-scripts/preambles/stdlib.lua:229) is defined in `stdlib.lua`;
it cannot be overridden. It determines the ball-possession state and calls the handler:

```
player_play[state] → team_play[state] → error()
```

States: `"i_have_ball"`, `"ball_is_free"`, `"team_has_ball"`, `"opponent_has_ball"`.

`team_play` is set in the team preamble and defines the default behavior for all players.
`player_play` — an optional partial override in the player script: unspecified states
fall back to `team_play`.

Subtlety: `"ball_is_free"` occurs only when `owner_team == "None"`; after a pass, team possession
is preserved and the state becomes `"team_has_ball"`.

## 5. Level Examples

Brief examples of each level with the "allowed/forbidden" boundary.

`core.lua` — only reads `context`/`GAME_DATA`, without game logic:

```lua
function ball_owner()
    return context.ball.owner_index
end
```

`stdlib.lua` — ready actions; `make_decision()` must not be overridden:

```lua
function stop(reason)
    return {action = "stop", reason = reason}
end
```

`preamble.lua` — `team_play` and common tactical tables:

```lua
team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
```

`script.lua` — partial override; unspecified states fall back to `team_play`:

```lua
player_play = { i_have_ball = forward_with_ball }
```

## 6. Decision Format and Data

The format of the returned decision, the context (`context`), and the static data (`GAME_DATA`) —
[`protocols.md`](protocols.md) §1. Not repeated here.
