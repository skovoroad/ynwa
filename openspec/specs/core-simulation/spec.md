# Core Simulation Specification

## Purpose

Defines `World` step execution across arbitrary system pipelines, `Game` initialization and partial state merging, temperature-controlled randomness (`RngManager`), and the 6-system **live match** pipeline (`SportGameManager -> PlayerReactionSystem -> BallPossessionSystem -> DecisionSystem -> ActionSystem -> PhysicsSystem`).

Live-match step dataflow (`rules -> intent -> action -> physics`) and coordination flags (replay and test worlds register different system subsets on `World`; see `recording-and-replay`):
1. **Sport Game Manager**: evaluates match events first, switches `GameStage` (`Setup(reason)` / `Play` / `GameOver`), and assigns non-scripted positioning decisions in `Setup` — ensuring events take effect before player decisions in the same tick without one-frame lag.
2. **`PlayerReactionSystem`**: raises `needs_decision = true` when a player's `reaction_rate` timer elapses in `Play` (or when `current_decision` is `None` in `Setup`).
3. **`BallPossessionSystem`**: resolves `1.0` m ball contests before scripts run and sets `needs_decision = true` on **all** players upon possession change (interrupting reaction timers).
4. **`DecisionSystem`**: unconditionally checks 2D arrival (`< 0.5` m -> `Decision::Stop`, `decision_processed = false`) every tick; in `Play`, invokes `DecisionMaker` for `needs_decision == true`, flips `Team::B` output to Display Orientation, sets `decision_processed = false`, and clears `needs_decision`.
5. **`ActionSystem`**: consumes unprocessed decisions (`!decision_processed`) once into straight-line velocity (`Stop`/`Run`) or a ball kick (`Kick`), then marks `decision_processed = true`.
6. **`PhysicsSystem`**: integrates `position += velocity * dt` and free-ball friction last so all step mutations apply consistently. In `Setup`, reaching a target triggers `Decision::Stop` (step 4), which the Sport Game Manager observes on the next tick to mark `is_ready = true` until all players are ready to enter `Play`.

Key domain concepts:
- **`temperature` (`RngConfig::temperature`, `0.0..=1.0`)**: continuous stochasticity scale controlling random spread across tackle contests and kicks — `0.0` disables all randomness for deterministic tests, `1.0` gives full variance, and live matches use `0.7`.
- **`possessed_by` vs `last_possessing_team` (`BallState`)**: `possessed_by` is the current physical ball carrier (cleared to `None` immediately when kicked), whereas `last_possessing_team` retains team control while a pass or shot travels freely (cleared only on `Setup`), allowing scripts and boundary rules to distinguish an in-flight team ball from a neutral ball.

## Requirements

### Requirement: Game Initialization and Partial State Merging

`Game::new` (defaulting to `Setup("kick off")`) and `Game::with_stage` SHALL initialize `GameState` deterministically at `elapsed_time = 0.0`. Players start off-field at `(-5.0, 0.0, field_length / 2.0)` in `Setup(_)` or at `regions["start"].center()` (panicking if `"start"` is missing) in `Play` | `GameOver`, with `needs_decision = true`. `Game::apply_state_spec` MUST merge only `Some` fields from `PartialStateSpec` and reject out-of-bounds `player_index`.

#### Scenario: Immediate initial ball pickup readiness and selective spec override
- **WHEN** `Game` is initialized and an optional `PartialStateSpec` is applied
- **THEN** `BallState` starts at `config.ball.initial_position` with `last_possession_change_time = -1.0` so the `1.0` s possession cooldown does not block pickup at `t >= 0.0`, and `apply_state_spec` overrides only present `stage`, `ball`, `players` positions, `setup`, and `score` fields while leaving omitted fields at stage defaults

### Requirement: Configurable World Step Execution and Live Simulation Ordering

`World::step(&mut self, timestep: f32)` SHALL compute `new_ts = elapsed_time + timestep`, call each registered `System::update(&mut game, new_ts)` in insertion order **before** setting `elapsed_time = new_ts`, and call `game.finish_step(new_ts)`. `World` MUST allow arbitrary system lists (replay, tests), while **live match** worlds MUST order: Sport Game Manager, `PlayerReactionSystem`, `BallPossessionSystem`, `DecisionSystem`, `ActionSystem`, `PhysicsSystem`.

#### Scenario: Pre-step elapsed_time visibility and pipeline composability
- **WHEN** `World::step(timestep)` runs at `elapsed_time = t_prev` on either a 6-system live match world or a reduced replay/test world
- **THEN** every registered system receives `timestamp = t_prev + timestep` while observing `game.state.elapsed_time == t_prev`, and in a live match world flags raised by earlier systems propagate through `DecisionSystem`, `ActionSystem`, and `PhysicsSystem` within the **same** step

### Requirement: Temperature-Controlled Randomness (`RngManager`)

`DefaultRngManager` (`RngConfig { temperature, seed }`) SHALL assert `0.0 <= temperature <= 1.0`. At `temperature == 0.0`, `next()` MUST return `0.5`, `randomize(base, _)` MUST return exact `base`, and `randomize_range(_)` MUST return `0.0`. At `temperature > 0.0`, `randomize(base, pct)` MUST scale `base` by a uniform offset within `±(temperature * pct)`, and `randomize_range(max)` MUST sample uniformly within `[-temperature * max, +temperature * max]`.

#### Scenario: Zero-temperature determinism
- **WHEN** `DefaultRngManager` is created with `temperature = 0.0`
- **THEN** tackle contest scores equal exact `tackle_rate` and kick speeds and directions experience zero random deviation

### Requirement: Player Reaction System Scheduling

`PlayerReactionSystem` SHALL update `PlayerState::needs_decision` per player:
- In `GameStage::Setup(_)`: set `needs_decision = current_decision.is_none()`.
- In `GameStage::Play` and `GameStage::GameOver`: NEVER clear an already-`true` `needs_decision`; set `needs_decision = true` when `timestamp - last_decision_time >= interval`, where `interval` is linearly interpolated by clamped `reaction_rate` (`10..=100`) from `3.0` s (at `rate <= 10`) down to `0.5` s (at `rate >= 100`).

#### Scenario: Linear reaction interval bounds and midpoint scheduling
- **WHEN** `PlayerReactionSystem` updates players in `GameStage::Play`
- **THEN** `reaction_rate = 100` schedules decisions every `0.5` s, `reaction_rate = 10` every `3.0` s, midpoint `reaction_rate = 55` every `1.75` s (`timestamp - last_decision_time >= interval`), and any `needs_decision == true` raised earlier (e.g., on a possession change) remains `true` even before `interval` elapses

### Requirement: Ball Possession Candidate Selection and Cooldown

`BallPossessionSystem` SHALL return immediately in `Setup(_)` or while `timestamp - last_possession_change_time < 1.0` s (`POSSESSION_COOLDOWN`). Otherwise, it SHALL collect candidates within 3D distance `<= 1.0` m (`POSSESSION_RADIUS`) of the ball: if `possessed_by == Some(owner)`, ONLY opponents (`team != owner_team`) are eligible; if `possessed_by == None`, all players within `1.0` m are eligible. Empty candidate lists MUST leave possession unchanged.

#### Scenario: Teammates never steal from current ball owner
- **WHEN** a player possesses the ball while teammates are within the `1.0` m radius and all opponents are beyond `1.0` m
- **THEN** no candidates are eligible, `ball_state.possessed_by` stays unchanged, and neither the `1.0` s cooldown nor `PossessionChange` is triggered

### Requirement: Ball Possession Tackle Contest and Transition Side Effects

In `BallPossessionSystem`, a single candidate wins without RNG; multiple candidates are scored in ascending `player_index` order via `rng.randomize(tackle_rate as f32, 0.5)` (`±50%` at `temperature = 1.0`) and sorted descending (lower `player_index` wins ties). On possession change, the system MUST update `possessed_by`, set `last_possession_change_time = timestamp` and `last_possessing_team = Some(winner_team)`, set `needs_decision = true` on **all** players, and emit `PossessionChange`.

#### Scenario: Deterministic tie-breaking and global wake-up on possession change
- **WHEN** multiple candidates within `1.0` m contest a ball after cooldown and tie on effective tackle score (such as equal `tackle_rate` at `temperature = 0.0`)
- **THEN** the candidate with the lowest `player_index` wins possession, `last_possessing_team` becomes that winner's team, all players get `needs_decision = true`, and `JournalEvent::PossessionChange` is recorded

### Requirement: Decision System Arrival Check and Setup Guard

For each player on every step regardless of `GameStage` or reaction timer, `DecisionSystem` SHALL check whether `current_decision` is `Run(target)` within XZ distance `< 0.5` m. On arrival, it MUST assign `Decision::Stop` (`decision_processed = false`, preserving `decision_reason`, and clearing `needs_decision` in `Setup`), emit `DecisionAssigned { Stop, reason: None }`, and skip further checks. In `Setup(_)`, `DecisionMaker` MUST NEVER be called (`needs_decision = false`).

#### Scenario: Automatic stop upon crossing the 0.5 m horizontal arrival threshold
- **WHEN** a player executing `Decision::Run` comes within `< 0.5` m of the resolved target point in the XZ plane
- **THEN** `DecisionSystem` assigns `Decision::Stop` (`decision_processed = false`), emits `JournalEvent::DecisionAssigned` with `reason = None`, and `ActionSystem` zeroes the player's velocity within the same step

### Requirement: Decision Maker Invocation, Orientation Flip, and Error Isolation

Outside `Setup` when `needs_decision == true`, `DecisionSystem` SHALL call `decision_maker.make_decision(game, idx)`, flip `Team::B` decisions to Display Orientation, store `current_decision`/`decision_reason` (`decision_processed = false`), clear `needs_decision`, set `last_decision_time = timestamp`, and emit `DecisionAssigned`. On `Err`, it MUST still clear `needs_decision`, update `last_decision_time`, store `last_error`, and invoke `on_error`.

#### Scenario: Rate-limited script error isolation and default stub
- **WHEN** `DecisionMaker::make_decision` returns an error (syntax, runtime error, or timeout) with the default error handler (or when default `PlaceholderDecisionMaker` runs to a random `GridCell`)
- **THEN** on error `current_decision` becomes `None`, `last_error` records the failure, and `needs_decision` is cleared with `last_decision_time = timestamp` so the failing script is throttled until the next reaction interval or possession change

### Requirement: Action System Stop and Run Velocity Computation

`ActionSystem` SHALL process every player with `current_decision.is_some() && !decision_processed` and set `decision_processed = true` (sharing identical target/velocity kinematics with replay). `Decision::Stop` sets `velocity = 0`. `Decision::Run(target)` snapshots the target point once (`Ball` resolves to the ball's position at processing time); if 3D distance `< 0.5` m, `velocity = 0`, otherwise player speed is `(speed_rate as f32 / 100.0) * 10.0` m/s along the normalized direction vector.

#### Scenario: Delayed-pursuit ball chasing vs live arrival check
- **WHEN** `ActionSystem` processes `Decision::Run(DecisionTarget::Ball)` while the ball is moving
- **THEN** the player's velocity vector aims straight at where the ball was when `ActionSystem` processed the decision until a new decision is made, whereas `DecisionSystem`'s `< 0.5` m arrival check evaluates against the ball's **current** position on every tick

### Requirement: Action System Kick Mechanics

`Decision::Kick(target)` in `ActionSystem` MUST be a no-op unless `possessed_by == Some(player_index)` and MUST NOT alter `player_state.velocity`. When possessed, `ActionSystem` SHALL sample speed from `shot_power / 5.0` m/s (`±25%` spread) and XZ angular deviation linearly interpolated by `shot_accuracy` (`10..=100`) from `±45°` down to `±5°`, set `ball_state.velocity = (vx, 0.0, vz)`, clear `possessed_by = None`, set `last_possession_change_time = timestamp`, and emit `KickOutcome`.

#### Scenario: Planar XZ kick launch, degenerate-target fallback, and pickup cooldown
- **WHEN** the ball owner executes `Decision::Kick(target)`
- **THEN** `ball_state.velocity` lies strictly in the ground plane (`vy = 0.0`, falling back to `+X` direction if horizontal distance to `target` is `< 0.001` m; aerial kicks are unimplemented), `possessed_by` becomes `None`, `last_possessing_team` is retained, and advancing `last_possession_change_time` activates the `1.0` s cooldown so the kicker cannot immediately reclaim the ball on the next tick

### Requirement: Physics System Kinematics and Ball Friction

`PhysicsSystem` SHALL compute `dt = timestamp - self.last_update` (init `0.0`), update `last_update = timestamp`, and return if `dt <= 0.0`. Players integrate `position += velocity * dt` without inertia, collisions, or boundary clamping. A possessed ball snaps to its owner's position with `velocity = 0`; a moving free ball decelerates by `2.0` m/s² (clamped at `0.0` speed) **before** integrating `ball.position += ball.velocity * dt` (no gravity or bounce).

#### Scenario: Semi-implicit free-ball friction integration without direction reversal
- **WHEN** `PhysicsSystem` advances a moving free ball over `dt > 0.0`
- **THEN** ball speed is first reduced by `2.0 * dt` m/s (clamped to `>= 0.0` so friction never reverses velocity) and the updated velocity is then multiplied by `dt` to advance `ball.position`
