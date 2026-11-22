# Spec Delta

## ADDED Requirements

### Requirement: Setup-Stage Accelerated Pacing

`ynwa-player` SHALL advance the simulation at a fixed `4x` wall-clock speed while the game is in the `Setup` stage, and at normal `1x` speed during the `Play` stage, in all three modes (`Play`, `Record`, `Replay`). The acceleration SHALL change only wall-clock pacing: the simulation step and the resulting match clock, journal, and replay determinism MUST be unchanged.

#### Scenario: Setup completes in a quarter of the wall-clock time

- **WHEN** the game is in the `Setup` stage in any of `Play`, `Record`, or `Replay` mode
- **THEN** the match clock advances at `4x` wall-clock speed, so a setup stage of `N` match-seconds completes in approximately `N/4` wall-clock seconds

#### Scenario: Play resumes at normal speed

- **WHEN** the game transitions from `Setup` to `Play`
- **THEN** the player resumes `1x` pacing (real time)

#### Scenario: Acceleration does not alter the simulation

- **WHEN** the player accelerates the `Setup` stage
- **THEN** the simulation step stays fixed and the recorded journal and replayed match are identical to an unaccelerated run; only wall-clock pacing differs
