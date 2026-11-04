# Spec Delta

## MODIFIED Requirements

### Requirement: Configurable World Step Execution and Live Simulation Ordering

`World::step(&mut self, timestep: f32)` SHALL compute `new_ts = elapsed_time + timestep`, call each registered `System::update(&mut game, new_ts)` in insertion order **before** setting `elapsed_time = new_ts`, and call `game.finish_step(new_ts)`. `World` MUST allow arbitrary system lists (replay, tests), while **live match** worlds MUST order: Sport Game Manager, `PlayerReactionSystem`, `BallPossessionSystem`, `DecisionSystem`, `ActionSystem`, `PhysicsSystem`.

#### Scenario: Pre-step elapsed_time visibility and pipeline composability
- **WHEN** `World::step(timestep)` runs at `elapsed_time = t_prev` on either a 6-system live match world or a reduced replay/test world
- **THEN** every registered system receives `timestamp = t_prev + timestep` while observing `game.state.elapsed_time == t_prev`, and in a live match world flags raised by earlier systems propagate through `DecisionSystem`, `ActionSystem`, and `PhysicsSystem` within the **same** step
