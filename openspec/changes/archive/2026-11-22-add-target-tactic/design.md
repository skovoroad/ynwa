# Design

## Context

`context/vision/tactics.md` defines the first-stage target tactic with distinct roles: Goalkeeper, Fullbacks (wide defenders), Center Backs, Wide Midfielders, Central Midfielders, and Forwards, plus a shared team fallback.

## Goals / Non-Goals

**Goals:**
- Implement the target tactic as `teams/team_c` (`meta.toml`, `preamble.lua`, and 11 players `01`..`11` with `static.toml`, `tactical.toml`, `script.lua`).
- Strictly use only the public `stdlib.lua` API (no loops, no coordinate reads, no manual decision tables).
- Ensure `teams/team_c` loads cleanly via `FsTeamRepository` and can play full matches against `team_a` / `team_b` in `ynwa-simulator` and `ynwa-player`.

**Non-Goals:**
- Adding new `stdlib.lua` functions (all required primitives come from `update-lua-stdlib`).
- Designing set-piece-specific AI beyond standard `tactical.toml` `[set_piece_positions]`.

## Decisions

1. **Squad formation & numbering (`teams/team_c/players/01..11`):**
   - `01`: Goalkeeper (`goalkeeper_play`)
   - `02`, `03`: Fullbacks (`fullback_with_ball`)
   - `04`, `05`: Center Backs (`center_back_with_ball`)
   - `06`, `08`: Central Midfielders (`central_midfielder_with_ball`)
   - `07`, `11`: Wide Midfielders (`wide_midfielder_with_ball`)
   - `09`, `10`: Forwards (`forward_with_ball`)
   - Base `tactical.toml` and `static.toml` templates can be adapted from `teams/team_a` with appropriate `attack`/`defence` regions per role.

2. **Target tactic logic mapping to `stdlib.lua`:**
   - **Team fallback (`team_play`)**:
     - `team_has_ball`: `run_to_attack_position`
     - `opponent_has_ball` / `ball_is_free`: if `count_teammates_around(ball_position(), distance(my_position(), ball_position())) < 3`, `chase_ball()`, else `run_to_defence_position()`.
   - **Goalkeeper (`01`)**: `i_have_ball = default_goalkeeper_with_ball`, `team_has_ball = default_goalkeeper_cover_position`, `opponent_has_ball = default_goalkeeper_cover_position`, `ball_is_free = default_goalkeeper_cover_position`.
   - **Fullbacks (`02`, `03`)**: if in opponent penalty area (or flank attacking region `is_in_region(...)`), pass to a free teammate from a designated list (`9, 10, 7, 11`) via `kick_to_position(teammate_position(n), "pass_to_#" .. n)`, or to the nearest non-free teammate from the same list if none is free; else `run_to_opponent_penalty_area()` (dribbling with ball).
   - **Center Backs (`04`, `05`)**: check candidate midfielders/fullbacks (`6, 8, 7, 11`) using `is_teammate_free(n) and is_teammate_ahead(n)` and compare `distance(my_position(), teammate_position(n))` manually in `if` branches to pick the nearest free teammate ahead; fallback to `run_to_attack_position()`. *(Note for plan step 7: manual `if` comparisons over 4 candidates produce verbose code — collect as feedback for DSL design.)*
   - **Wide Midfielders (`07`, `11`)**: if a forward (`9`, `10`) is `is_teammate_free(n) and is_teammate_ahead(n)`, pass via `kick_to_position(teammate_position(n), "pass_to_#" .. n)`; else if `is_in_opponent_penalty_area()` then `kick_to_opponent_goal()`, else `run_to_opponent_penalty_area()`.
   - **Central Midfielders (`06`, `08`)**: check primary list (`9, 10`) for `is_teammate_free(n) and is_teammate_ahead(n)`, then secondary list (`7, 11`), else advance to penalty area and shoot.
   - **Forwards (`09`, `10`)**: if `is_in_opponent_penalty_area()` then `kick_to_opponent_goal()`, else `run_to_opponent_penalty_area()`.

