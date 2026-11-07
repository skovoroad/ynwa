# Design

## Context

`discover_scenarios()` (in `tests/scenarios.rs`) reads only the immediate subdirectories of `ynwa-integration-testing/scenarios/` that contain a `scenario.toml`, sorts them by name, and runs each via `run_scenario`. `YNWA_SCENARIO` filters by exact directory name. A scenario directory holds `initial_state.toml`, `scenario.toml` (`[run]` timestep + `[[run.stop]]`, `[expect]` journal/stop), an optional `final_state.toml`, and a `teams/` tree (`team_a`, `team_b`, each with `preamble.lua` and `players/N/{static,tactical}.toml`). Stop criteria are stage/event/journal/steps/time; journal patterns include `decision_assigned` (with target/reason), `possession_change`, `kick_outcome`, `stage_change`, `restart_set`, `decisions_reset`, `stat_update`, and `football_event`; final state supports player position/region and score. See `proposal.md` for motivation.

The runner evaluates stop criteria in declaration order after every step and stops on the first satisfied one. Systems run in the order `FootballGameManager → PlayerReaction → BallPossession → Decision → Action → Physics`, so a `Play` start with the ball free and a taker standing on it emits `PossessionChange` then `KickOutcome` within the first step.

## Goals / Non-Goals

**Goals:**

- Author the full target set from `context/test_scenarios.md` as flat directories.
- Make every run replayable in the player (small timestep, several seconds of model time).
- Keep the 6 legacy scenarios untouched.

**Non-Goals:**

- No framework or core code changes.
- No spec changes (`skip_specs: true`).
- No scenario parameterization; one directory per combinatoric combination.
- No retirement of the legacy scenarios (handled separately and manually).

## Decisions

1. **Flat `<group>_<case>_<combination>` naming with physical combination suffix.** Alphabetical discovery reproduces the document's grouping for free. The `<combination>` suffix uses absolute field coordinates in Display Orientation (`Team::A` perspective), independent of team perspective:
   - Corner & goal kick: `goal_a`/`goal_b` (goal line: `goal_a` at `Z = 0`, `goal_b` at `Z = length`) × `x0`/`xw` (side line: `X = 0` / `X = width`) — e.g. `corner_awarded_goal_a_x0`. A corner/goal-kick at `goal_a_*` is in favour of / taken by the team defending goal `A`; `goal_b_*` mirrors it for goal `B`.
   - Throw-in: `x0`/`xw` (side line) × `half_a`/`half_b` (half: `Z ∈ [0, length/2)` / `[length/2, length]`) × `by_a`/`by_b` (last-touch team) — e.g. `throw_in_awarded_x0_half_a_by_a`. The throw is awarded to the opposite of the `by_*` team.
   - Kick-off, pass and decision scenarios: `team_a`/`team_b` (kicking/passing/deciding team).

2. **Visual replay via time-based stop.** `run.timestep = 0.02` (50 Hz). Scenarios whose natural outcome is a single journal entry emitted on the first step (`*_kick_*`, `decision_*`) must not stop on that entry — they stop on `when = "time"` and assert the outcome via `[[expect.journal]]`, with the `steps` safety criterion kept well above the duration. Target model durations: `*_kick_*` ≈ 2.5–3.0 s, `decision_*` ≈ 2.0–3.0 s. `*_awarded_*` and `goal_scored_*` stop on their football event (`GoalLine`/`Touchline`/`Goal`), which already spans multiple steps as the ball travels.

3. **One directory per combination**, with the physical suffix from decision 1. The full enumeration (per group, with counts and names) is in `tasks.md`.

4. **Free-ball start for `*_kick_*`.** Each `*_kick_*` initial state sets the ball free (`possessed_by = "none"`, `last_possessing_team = "none"`) with the taker standing on the ball, so `BallPossessionSystem` picks it up on the first `Play` step. This exercises the "receives possession" part of the check as a real `PossessionChange` before `KickOutcome`.

5. **No scripted player kick for `*_awarded_*` and `goal_scored_*`.** These scenarios start the ball with a preset velocity toward the line/goal; the player action does not change the award/outcome under test. The `context/test_scenarios.md` wording ("после действия/удара игрока") will be aligned to this.

6. **`kick_off_after_goal_*` differs from `kick_off_*` by initial player positions.** The "after goal" scenarios place players at in-field positions that are not described by any tactical region (random post-goal spots), so they are not byte-identical to the match-start scenarios. The initial `score` is irrelevant to these checks and is not added to the DTO.

7. **Team composition per group follows the document** (corner/goal-kick: defender = goalkeeper + one outfield, attacker = two outfield; kick-off/throw-in: at least two outfield per team). Each scenario carries its own `teams/` tree with the required set-piece keys; `FootballWorldBuilder` validates the 16 keys and the `on_ball` count.

8. **`journal_match = "subsequence"`** for multi-event sequences (awarded/placement/goal and time-stopped `*_kick_*`); `"exact"` only where the full ordered journal is fixed and small.

9. **Granular decision scenarios.** `decision_targets_*` has one directory per (decision kind × target × team): `Stop`, `Run → point`, `Run → cell`, `Run → region`, `Run → ball`, `Kick → point`, each for `team_a` and `team_b` (12 directories). `decision_symmetry_*` checks mirrored situations for the coordinate-dependent targets `point`, `cell`, `region` and `kick` per team (2 directories).

10. **Full `kick_off_placement_*` checks.** Restore all four checks from the document: (a) ball at center, stationary, free in `Setup`; (b) `on_ball` player at the ball's exact position (asserted by `position` — `on_ball` produces no region), others in their regions; (c) decision scripts not invoked during `Setup` (assert `reason = "none"` on every `DecisionAssigned`, or use an error-throwing script); (d) `Play` reached only after all players have stopped.

11. **Engine pitfalls for scenario authors.** Two behaviours must be documented in this file and in code comments (the code comments are an apply-phase task):
    - `final_state.toml` player regions are compared in **Display Orientation** (`Team::A` perspective) without a `Team::B` flip (`ynwa-integration-testing/src/compare.rs`), whereas `team_b/players/*/tactical.toml` is written in `Team::B` local orientation and flipped on load. Final-state regions must be written in field coordinates.
    - `POSSESSION_COOLDOWN = 1.0 s` (`ynwa-core/src/systems/ball_possession.rs`): in `pass_received_*`/`pass_intercepted_*` the receiver or interceptor must receive the ball more than `1.0 s` after the kick, or `BallPossessionSystem` ignores the reception.

## Risks / Trade-offs

- [~78 directories each carry a full `teams/` tree → duplication] → accepted; team trees differ per group by player count and roles, and shared-team refactoring is out of scope (YAGNI).
- [Small timestep increases step count, slowing the suite] → ~78 scenarios × ~200 steps is still cheap; keep the `steps` safety criterion well above the expected duration.
- [Legacy scenarios overlap with the new ones → duplicated coverage] → acceptable; legacy retirement is handled manually later.
