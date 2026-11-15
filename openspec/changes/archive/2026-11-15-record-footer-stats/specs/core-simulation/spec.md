# Spec Delta

## MODIFIED Requirements

### Requirement: Game Initialization and Partial State Merging

`Game::new` (defaulting to `Setup("kick off")`) and `Game::with_stage` SHALL initialize `GameState` deterministically at `elapsed_time = 0.0` and `step_count = 0`. Players start off-field at `(-5.0, 0.0, field_length / 2.0)` in `Setup(_)` or at `regions["start"].center()` (panicking if `"start"` is missing) in `Play` | `GameOver`, with `needs_decision = true`. `Game::apply_state_spec` MUST merge only `Some` fields from `PartialStateSpec` and reject out-of-bounds `player_index`.

#### Scenario: Immediate initial ball pickup readiness and selective spec override
- **WHEN** `Game` is initialized and an optional `PartialStateSpec` is applied
- **THEN** `BallState` starts at `config.ball.initial_position` with `last_possession_change_time = -1.0` so the `1.0` s possession cooldown does not block pickup at `t >= 0.0`, and `apply_state_spec` overrides only present `stage`, `ball`, `players` positions, `setup`, and `score` fields while leaving omitted fields at stage defaults

### Requirement: Configurable World Step Execution and Live Simulation Ordering

`World::step(&mut self, timestep: f32)` SHALL compute `new_ts = elapsed_time + timestep`, call each registered `System::update(&mut game, new_ts)` in insertion order **before** advancing `game.step(timestep)` (`elapsed_time = new_ts` and `step_count += 1`). `World` MUST allow arbitrary system lists (replay, tests), while **live match** worlds MUST order: Sport Game Manager, `PlayerReactionSystem`, `BallPossessionSystem`, `DecisionSystem`, `ActionSystem`, `PhysicsSystem`.

#### Scenario: Pre-step elapsed_time visibility and pipeline composability
- **WHEN** `World::step(timestep)` runs at `elapsed_time = t_prev` and `step_count = n_prev` on either a 6-system live match world or a reduced replay/test world
- **THEN** every registered system receives `timestamp = t_prev + timestep` while observing `game.state.elapsed_time == t_prev` and `game.state.step_count == n_prev`, after which `elapsed_time` becomes `new_ts` and `step_count` becomes `n_prev + 1`, and in a live match world flags raised by earlier systems propagate through `DecisionSystem`, `ActionSystem`, and `PhysicsSystem` within the **same** step

