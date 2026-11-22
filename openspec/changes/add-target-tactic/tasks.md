# Tasks

## 1. Team C Setup

- [x] 1.1 Create `teams/team_c/meta.toml` and player directories `teams/team_c/players/01..11` with `static.toml`, `tactical.toml`, and `script.lua` role assignments

## 2. Target Tactic Implementation

- [x] 2.1 Implement `team_play` fallback in `teams/team_c/preamble.lua` (attack position when team has ball; loop-free top-3 press `count_teammates_around(ball_position(), distance(my_position(), ball_position())) < 3` or defence position when opponent has ball / ball is free)
- [x] 2.2 Implement `goalkeeper_play` using `default_goalkeeper_with_ball` and `default_goalkeeper_cover_position` (for `team_has_ball`, `opponent_has_ball`, and `ball_is_free`)
- [x] 2.3 Implement `fullback_with_ball` (free teammate or nearest non-free fallback from the same list), `center_back_with_ball` (manual `if` distance/free/ahead comparison), `wide_midfielder_with_ball`, `central_midfielder_with_ball`, and `forward_with_ball` using only public `stdlib.lua` functions

## 3. Verification

- [x] 3.1 Verify `teams/team_c` loads via `FsTeamRepository` without validation errors and obeys all Layer 3–4 authoring rules
- [x] 3.2 Run `ynwa-simulator` matches with `team_c` against `team_a` / `team_b` and verify zero Lua runtime errors (tactical behavior analysis is performed manually by the developer)

