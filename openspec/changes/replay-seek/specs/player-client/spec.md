# Spec Delta

## ADDED Requirements

### Requirement: Replay Seek

`Replay` mode SHALL support seeking the playback position forward and backward by a fixed 5 seconds of match time. A seek SHALL move only the replay position: it SHALL NOT change the simulation `timestep` or the wall-clock `speed`. Forward seeks advance the running replay; backward seeks SHALL reproduce the same observable state a fresh replay reaches when stepped to the target position.

#### Scenario: Forward seek within the recording
- **WHEN** the user presses the forward-seek key (`Right`) at a replay position with at least 5 match-seconds remaining
- **THEN** the replay advances by 5 match-seconds and continues stepping from the new position

#### Scenario: Backward seek reproduces the earlier state
- **WHEN** the user presses the backward-seek key (`Left`) at a replay position at least 5 match-seconds after the start
- **THEN** the replay returns to 5 match-seconds earlier, and the resulting observable state matches a fresh replay stepped to that position within the replay equivalence tolerance

#### Scenario: Backward seek clamps to the start
- **WHEN** the user seeks backward from a position fewer than 5 match-seconds after the start
- **THEN** the replay position clamps to the first step (step `0`)

#### Scenario: Forward seek clamps to the end
- **WHEN** the user seeks forward from a position fewer than 5 match-seconds before the end
- **THEN** the replay position clamps to `total_steps` and playback stops at the recording end

#### Scenario: Seek preserves timestep and speed
- **WHEN** the user seeks forward or backward in `Replay` mode
- **THEN** the simulation `timestep` remains fixed, the wall-clock `speed` is unchanged, and the seeked-to position reproduces the same state as a step-by-step replay to that position
