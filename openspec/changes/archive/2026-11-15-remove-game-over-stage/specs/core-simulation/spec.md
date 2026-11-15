# Spec Delta

## MODIFIED Requirements

### Requirement: Game Initialization and Partial State Merging

`Game::new` (defaulting to `Setup("kick off")`) and `Game::with_stage` SHALL initialize `GameState` deterministically at `elapsed_time = 0.0` and `step_count = 0`. Players start off-field at `(-5.0, 0.0, field_length / 2.0)` in `Setup(_)` or at `regions["start"].center()` (panicking if `"start"` is missing) in `Play`, with `needs_decision = true`. `Game::apply_state_spec` MUST merge only `Some` fields from `PartialStateSpec` and reject out-of-bounds `player_index`.

#### Scenario: Immediate initial ball pickup readiness and selective spec override
- **WHEN** `Game` is initialized and an optional `PartialStateSpec` is applied
- **THEN** `BallState` starts at `config.ball.initial_position` with `last_possession_change_time = -1.0` so the `1.0` s possession cooldown does not block pickup at `t >= 0.0`, and `apply_state_spec` overrides only present `stage`, `ball`, `players` positions, `setup`, and `score` fields while leaving omitted fields at stage defaults

### Requirement: Player Reaction System Scheduling

`PlayerReactionSystem` SHALL update `PlayerState::needs_decision` per player:
- In `GameStage::Setup(_)`: set `needs_decision = current_decision.is_none()`.
- In `GameStage::Play`: NEVER clear an already-`true` `needs_decision`; set `needs_decision = true` when `timestamp - last_decision_time >= interval`, where `interval` is linearly interpolated by clamped `reaction_rate` (`10..=100`) from `3.0` s (at `rate <= 10`) down to `0.5` s (at `rate >= 100`).

#### Scenario: Linear reaction interval bounds and midpoint scheduling
- **WHEN** `PlayerReactionSystem` updates players in `GameStage::Play`
- **THEN** `reaction_rate = 100` schedules decisions every `0.5` s, `reaction_rate = 10` every `3.0` s, midpoint `reaction_rate = 55` every `1.75` s (`timestamp - last_decision_time >= interval`), and any `needs_decision == true` raised earlier (e.g., on a possession change) remains `true` even before `interval` elapses
