## MODIFIED Requirements

### Requirement: Decision System Arrival Check and Setup Guard

For each player on every step regardless of `GameStage` or reaction timer, `DecisionSystem` SHALL check whether `current_decision` is `Run(target)` within XZ distance `< 0.5` m of the resolved target point (`Point`, `GridCell` center, or `Region` center). On arrival, it MUST assign `Decision::Stop` (`decision_processed = false`, preserving `decision_reason`, and clearing `needs_decision` in `Setup`), emit `DecisionAssigned { Stop, reason: None }`, and skip further checks. In `Setup(_)`, `DecisionMaker` MUST NEVER be called (`needs_decision = false`).

#### Scenario: Automatic stop upon crossing the 0.5 m horizontal arrival threshold
- **WHEN** a player executing `Decision::Run` comes within `< 0.5` m of the resolved target point in the XZ plane
- **THEN** `DecisionSystem` assigns `Decision::Stop` (`decision_processed = false`), emits `JournalEvent::DecisionAssigned` with `reason = None`, and `ActionSystem` zeroes the player's velocity within the same step

### Requirement: Action System Stop and Run Velocity Computation

`ActionSystem` SHALL process every player with `current_decision.is_some() && !decision_processed` and set `decision_processed = true` (sharing identical target/velocity kinematics with replay). `Decision::Stop` sets `velocity = 0`. `Decision::Run(target)` resolves `target` (`Point`, `GridCell`, or `Region`) to a `Point3D`; if 3D distance `< 0.5` m, `velocity = 0`, otherwise player speed is `(speed_rate as f32 / 100.0) * 10.0` m/s along the normalized direction vector.

#### Scenario: Delayed-pursuit ball chasing vs live arrival check
- **WHEN** `ActionSystem` processes `Decision::Run(DecisionTarget::Point)` for a moving target (such as a ball-chasing script decision)
- **THEN** the player's velocity aims straight at that fixed point until a new decision is made, and `DecisionSystem` checks `< 0.5` m XZ arrival against that fixed point (`DecisionTarget::Ball` no longer exists)

#### Scenario: Run velocity resolution for static targets
- **WHEN** `ActionSystem` processes `Decision::Run` with `Point`, `GridCell`, or `Region`
- **THEN** it resolves the target `Point3D` once, sets `velocity = 0` if 3D distance `< 0.5` m, or sets constant velocity at `(speed_rate / 100.0) * 10.0` m/s toward that point, while `DecisionSystem` checks `< 0.5` m XZ arrival on every tick
