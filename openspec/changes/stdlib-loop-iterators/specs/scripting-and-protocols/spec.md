# Spec Delta

## ADDED Requirements

### Requirement: Bounded Loop Iterators in `stdlib.lua`

`stdlib.lua` SHALL expose two `for ... in` iterator helpers so team preambles express bounded loops without Lua iteration or sorting mechanics: `each(numbers)` yields each number in list order, and `order_by_distance_asc(numbers)` yields numbers ordered by ascending `distance` to me, preserving list order on equal distances. Both yield the provisional squad-number identifier (one value per step) and terminate when the source is empty or exhausted.

#### Scenario: `each` yields the list in order
- **WHEN** a team preamble iterates `for player in each({10, 11})`
- **THEN** the loop body runs twice, first with `player == 10` then with `player == 11`, and no numeric key is exposed to the loop body

#### Scenario: `order_by_distance_asc` sorts by ascending distance with stable tie-break
- **WHEN** a team preamble iterates `for player in order_by_distance_asc({6, 8, 7, 11})`
- **THEN** the numbers are yielded ordered by ascending `distance(my_position(), teammate_position(number))`, and numbers at equal distance keep their order in the input list

#### Scenario: Empty source yields no iterations
- **WHEN** a team preamble iterates over an empty list (`each({})` or `order_by_distance_asc({})`)
- **THEN** the loop body never executes

#### Scenario: Iterators compose with full-team number lists
- **WHEN** a team preamble iterates `for player in order_by_distance_asc(teammate_numbers())` or `for player in each(opponent_numbers())`
- **THEN** the loop yields the numbers returned by `teammate_numbers()` (teammates excluding me) or `opponent_numbers()` in the iterator's order, without calling `get_teammates()` / `get_opponents()`

## MODIFIED Requirements

### Requirement: Team and Player Script Authoring Rules (Layers 3 and 4)

Team-owned scripts — Layer 3 `teams/<id>/preamble.lua` (`team_play` default handlers and shared role tables like `goalkeeper_play`) and Layer 4 `teams/<id>/players/<NN>/script.lua` (optional partial `player_play` override) — SHALL obey 6 rules: (1) call only functions of the user-facing API (the user-facing Layer 1 functions and the public `stdlib.lua` API) and never access `context` or `GAME_DATA` directly, (2) never override `make_decision()`, (3) never hand-craft `{action=...}` tables, (4) never call `get_teammates()` / `get_opponents()` or write nested loops, while non-nested loops over explicit squad-number lists or `teammate_numbers()` / `opponent_numbers()` using the `each` / `order_by_distance_asc` iterators are permitted, (5) never read raw coordinate fields (`.x`, `.y`, `.z`) of positions, and (6) prefer point-free handler assignment (`ball_is_free = chase_ball`).

#### Scenario: Partial `player_play` override falling back to `team_play`
- **WHEN** `preamble.lua` defines `team_play = { ball_is_free = chase_ball, team_has_ball = run_to_attack_position, opponent_has_ball = run_to_defence_position }` and a forward's `script.lua` defines `player_play = { i_have_ball = kick_to_opponent_goal }`
- **THEN** `make_decision()` invokes `player_play.i_have_ball` when that forward owns the ball and falls back to `team_play[state]` for the other three possession states

#### Scenario: DSL-aligned loop via iterator helpers
- **WHEN** a team preamble iterates (without nesting) with `each(...)` or `order_by_distance_asc(...)` over an explicit squad-number list or `teammate_numbers()` / `opponent_numbers()`, and does not call `get_teammates()` or `get_opponents()`
- **THEN** the script complies with Rule (4) and remains compatible with future DSL transpilation
