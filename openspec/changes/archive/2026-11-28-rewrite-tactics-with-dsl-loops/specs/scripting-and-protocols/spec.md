## MODIFIED Requirements

### Requirement: Team and Player Script Authoring Rules (Layers 3 and 4)

Team-owned scripts (`teams/<id>/preamble.lua` and `teams/<id>/players/<NN>/script.lua`) SHALL obey 6 rules: (1) call only the user-facing Layer 1 and public `stdlib.lua` APIs without accessing `context` or `GAME_DATA`, (2) never override `make_decision()`, (3) never hand-craft `{action=...}` tables, (4) never call `get_teammates()` / `get_opponents()` or write nested loops, while non-nested DSL-aligned loops over explicit squad-number lists or `teammate_numbers()` / `opponent_numbers()` are permitted, (5) never read raw `.x`/`.y`/`.z` fields, and (6) prefer point-free handler assignment.

#### Scenario: Partial `player_play` override falling back to `team_play`
- **WHEN** `preamble.lua` defines `team_play = { ball_is_free = chase_ball, team_has_ball = run_to_attack_position, opponent_has_ball = run_to_defence_position }` and a forward's `script.lua` defines `player_play = { i_have_ball = kick_to_opponent_goal }`
- **THEN** `make_decision()` invokes `player_play.i_have_ball` when that forward owns the ball and falls back to `team_play[state]` for the other three possession states

#### Scenario: Bounded DSL-aligned loop over candidate or full-team number list
- **WHEN** a team preamble iterates (without nesting) over an explicit squad-number list, `teammate_numbers()`, or `opponent_numbers()` (with optional `table.sort` by a distance key and early `return` on decision) without calling `get_teammates()` or `get_opponents()`
- **THEN** the script complies with Rule (4) and remains compatible with future DSL transpilation

