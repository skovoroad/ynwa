# Tasks

Every scenario below uses the flat `<group>_<case>_<combination>` naming with the physical combination suffix from `design.md`, and the replay convention from `design.md`: `run.timestep = 0.02`. `*_kick_*` and `decision_*` scenarios stop on `when = "time"` (≈2.5–3.0 s and ≈2.0–3.0 s respectively) and assert the outcome via `[[expect.journal]]`; `*_awarded_*` and `goal_scored_*` stop on their football event. Verify each task with `cargo test -p ynwa-integration-testing` (use `YNWA_SCENARIO=<name>` to run one scenario while authoring); the suite must stay green including the legacy scenarios.

## 1. Kick-off

- [x] 1.1 Author `kick_off_placement_team_a` and `kick_off_placement_team_b` (Setup("kick off") with the named `restart_team`, stop on `Play`); assert all four placement checks: ball at center, stationary and free in Setup; `on_ball` player at the ball's exact position (`position`, not `region`) with others in their regions; no decision script invoked during Setup (`reason = "none"` on every `DecisionAssigned`); `Play` reached only after all players stop. Verify both pass
- [x] 1.2 Author `kick_off_kick_team_a` and `kick_off_kick_team_b` (Play, ball free with the taker standing on it at center, stop on `time` ≈2.5–3.0 s, assert `PossessionChange` then `KickOutcome` with the kicking team's `last_possessing_team` via journal); verify both pass
 
## 2. Corner

- [x] 2.1 Author `corner_awarded_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (stop on `GoalLine`, assert `DecisionsReset` + `RestartSet` at nearest corner + `Setup("corner")`; ball starts with a preset velocity toward the goal line — no scripted kick); verify all 4 pass
- [x] 2.2 Author `corner_placement_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (Setup("corner"), stop on `Play`, assert placement via regions); verify all 4 pass
- [x] 2.3 Author `corner_kick_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (Play, ball free with the taker standing on it at the corner, stop on `time` ≈2.5–3.0 s, assert `PossessionChange` + `KickOutcome`); verify all 4 pass

## 3. Goal kick

- [x] 3.1 Author `goal_kick_awarded_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (stop on `GoalLine`, assert `RestartSet` at the 5.5 m line + `Setup("goal kick")`; ball starts with a preset velocity toward the goal line — no scripted kick); verify all 4 pass
- [x] 3.2 Author `goal_kick_placement_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (Setup("goal kick"), stop on `Play`, assert placement via regions); verify all 4 pass
- [x] 3.3 Author `goal_kick_kick_{goal_a_x0,goal_a_xw,goal_b_x0,goal_b_xw}` (Play, ball free with the taker standing on it at the goal-kick point, stop on `time` ≈2.5–3.0 s, assert `PossessionChange` + `KickOutcome`); verify all 4 pass

## 4. Throw-in

- [x] 4.1 Author `throw_in_awarded_{x0_half_a_by_a,x0_half_a_by_b,x0_half_b_by_a,x0_half_b_by_b,xw_half_a_by_a,xw_half_a_by_b,xw_half_b_by_a,xw_half_b_by_b}` (stop on `Touchline`, assert `RestartSet` at the crossing point + `Setup("throw in")`; ball starts with a preset velocity toward the touchline — no scripted kick); verify all 8 pass
- [x] 4.2 Author `throw_in_placement_` with the same 8 suffixes (Setup("throw in"), stop on `Play`, assert placement via regions); verify all 8 pass
- [x] 4.3 Author `throw_in_kick_` with the same 8 suffixes (Play, ball free with the taker standing on it at the throw-in point, stop on `time` ≈2.5–3.0 s, assert `PossessionChange` + `KickOutcome`); verify all 8 pass

## 5. Goal

- [x] 5.1 Author `goal_scored_a_by_b`, `goal_scored_a_own`, `goal_scored_b_by_a`, `goal_scored_b_own` (stop on `Goal`, assert `StatUpdate`/score + `RestartSet` center with conceding team + `Setup("kick off")`; ball starts with a preset velocity toward goal — no scripted kick); verify all 4 pass
- [x] 5.2 Author `kick_off_after_goal_placement_team_a` and `kick_off_after_goal_placement_team_b` (Setup("kick off") with conceding `restart_team`, players at in-field positions not described by any tactical region — distinct from the match-start `kick_off_placement_*` — stop on `Play`); verify both pass
- [x] 5.3 Author `kick_off_after_goal_kick_team_a` and `kick_off_after_goal_kick_team_b` (Play, ball free with the conceding team's taker standing on it at center, stop on `time` ≈2.5–3.0 s, assert `PossessionChange` + `KickOutcome`); verify both pass

## 6. Pass and interception

- [x] 6.1 Author `pass_received_team_a` and `pass_received_team_b` (stop on `PossessionChange` to the partner, assert `kick_outcome` then `possession_change` with unchanged `last_possessing_team`; receiver must get the ball more than 1.0 s after the kick per `POSSESSION_COOLDOWN`); verify both pass
- [x] 6.2 Author `pass_intercepted_team_a` and `pass_intercepted_team_b` (opponent on the pass lane, stop on `PossessionChange` to the opponent, assert `last_possessing_team` flips; interceptor must get the ball more than 1.0 s after the kick); verify both pass

## 7. Decision functions

- [x] 7.1 Author `decision_targets_*` — one directory per decision kind, target and team (12): `decision_targets_stop_team_{a,b}`, `decision_targets_run_point_team_{a,b}`, `decision_targets_run_cell_team_{a,b}`, `decision_targets_run_region_team_{a,b}`, `decision_targets_run_ball_team_{a,b}`, `decision_targets_kick_point_team_{a,b}`; assert `DecisionAssigned` kind, target and reason; stop on `time` ≈2.0–3.0 s and assert via journal. Verify each passes
- [x] 7.2 Author `decision_symmetry_team_a` and `decision_symmetry_team_b` (mirrored situations; assert the assigned coordinate-dependent targets `point`, `cell`, `region` and `kick` are symmetric across teams); verify both pass

## 8. Final verification

- [x] 8.1 Run `cargo test -p ynwa-integration-testing` and confirm every scenario passes; run `cargo clippy` and `cargo fmt -- --check` and confirm no warnings
- [x] 8.2 Add/adjust code comments documenting the two engine pitfalls for scenario authors: final-state regions are compared in Display Orientation (`ynwa-integration-testing/src/compare.rs`), and `POSSESSION_COOLDOWN` affects pass reception (`ynwa-core/src/systems/ball_possession.rs`)

## 9. Review fix: awarded/goal scenario duration

- [x] 9.1 Lengthen `corner_awarded_*`, `goal_kick_awarded_*`, `throw_in_awarded_*`, and `goal_scored_*` by starting the ball ≈33 m from the line/goal at `15 m/s` and repositioning players clear of the longer ball path, updating the throw-in crossing positions in `scenario.toml`/`final_state.toml`/`description.md`; verify `cargo test -p ynwa-integration-testing` passes

## 10. Review fix: corner kick delivery and stay-in-play guard

- [x] 10.1 Change `corner_kick_*` to cross the ball into the penalty area instead of `kick_to_opponent_goal()`, add `final_state.toml` (`stage = "Play"`) to each so an immediate out-of-bounds restart fails, and update the descriptions; verify `cargo test -p ynwa-integration-testing` passes

## 11. Review fix: `*_kick_*` final-state guard

- [x] 11.1 Add `final_state.toml` (`stage = "Play"`, `ball.possessed_by = "none"`, `ball.last_possessing_team = <kicking team>`) to all 16 remaining `*_kick_*` scenarios (`kick_off_kick_*`, `goal_kick_kick_*`, `throw_in_kick_*`, `kick_off_after_goal_kick_*`); verify `cargo test -p ynwa-integration-testing` passes

## 12. Review fix: observable placement scenarios with exact journal

- [x] 12.1 Make `kick_off_placement_*`, `corner_placement_*`, `goal_kick_placement_*`, and `throw_in_placement_*` start players off-field (movement), set equal `speed_rate`, assert the taker via `final_state.ball.possessed_by`, and use `journal_match = "exact"` built from the deterministic arrival order; update descriptions and `design.md` decision 10/14; verify `cargo test -p ynwa-integration-testing` passes
