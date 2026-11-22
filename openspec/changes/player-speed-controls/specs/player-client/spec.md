# Spec Delta

## MODIFIED Requirements

### Requirement: Simulation Pacing and Keyboard Controls

`ynwa-player` SHALL open a resizable `1200 × 900` window (`"YNWA Football Simulator"`). `Play` and `Record` start at `60.0` Hz (`timestep = 1/60` s) and advance simulation steps until `elapsed_time >= duration` (default `120.0` s; `Play` stops stepping while keeping the window open, and `Record` flushes the recording footer while keeping the window open for inspection until `Escape`); `Replay` runs at `rate = 1.0 / timestep` and stops stepping at `effective_steps`. `Space` pauses/resumes, `F11` toggles fullscreen, `Escape` exits cleanly (flushing the recording footer in `Record`), and `+`/`=`/`KP_ADD` and `-`/`KP_SUBTRACT` double/halve the playback `speed` (clamped to `0.25x`–`8x`) in `Play`, `Record`, and `Replay` modes. Changing `speed` SHALL alter only wall-clock pacing and never the simulation `timestep`.

#### Scenario: Preventing timestep drift in Record and Replay modes

- **WHEN** `+` or `-` keys are pressed in `Record` or `Replay` mode
- **THEN** the simulation `timestep` remains fixed and steps stay aligned with `RecordHeader::timestep`, while only the wall-clock playback `speed` changes

#### Scenario: Client-owned match duration termination in Play and Record modes

- **WHEN** `world.game().state().elapsed_time >= duration` is reached in `Play` or `Record` mode
- **THEN** both modes cease advancing `World::step` while continuing to render the final state, and `Record` mode additionally calls `finish_journal()` to flush the recording footer, staying open until `Escape` rather than exiting

#### Scenario: Speed is clamped to the supported range

- **WHEN** the user presses `-` repeatedly or `+` repeatedly
- **THEN** the playback speed stops at the `0.25x` lower bound and the `8x` upper bound
