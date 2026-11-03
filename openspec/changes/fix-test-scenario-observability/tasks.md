# Tasks

## 1. Out-of-bounds scenarios (goal line / touchline)

- [ ] 1.1 Update `scenarios/corner/initial_state.toml`: ball `z: 5 → 40`, `vz: -20 → -15`; verify with `YNWA_SCENARIO=corner cargo test -p ynwa-integration-testing`
- [ ] 1.2 Update `scenarios/goal_kick/initial_state.toml`: ball `z: 5 → 40`, `vz: -20 → -15`; verify with `YNWA_SCENARIO=goal_kick cargo test -p ynwa-integration-testing`
- [ ] 1.3 Update `scenarios/throw_in/initial_state.toml`: ball `x: 34 → 10`; verify with `YNWA_SCENARIO=throw_in cargo test -p ynwa-integration-testing`

## 2. Goal scenarios

- [ ] 2.1 Update `scenarios/goal_into_b/initial_state.toml`: ball `z: 99 → 70`, `vz: 20 → 15`; verify with `YNWA_SCENARIO=goal_into_b cargo test -p ynwa-integration-testing`
- [ ] 2.2 Update `scenarios/goal_into_a/initial_state.toml`: ball `z: 5 → 35`, `vz: -20 → -15`; verify with `YNWA_SCENARIO=goal_into_a cargo test -p ynwa-integration-testing`

## 3. Kick scenario

- [ ] 3.1 Update `scenarios/stdlib_kick_to_goal/initial_state.toml`: ball and player A#1 `z: 102 → 82`; verify with `YNWA_SCENARIO=stdlib_kick_to_goal cargo test -p ynwa-integration-testing`

## 4. Integration verification

- [ ] 4.1 Run the full `cargo test -p ynwa-integration-testing` and confirm all scenarios and the replay lockstep test pass
- [ ] 4.2 Replay each recording (`./replay.sh ~/.ynwa/scenarios/<name>.jsonl`) and confirm the pre-event phase is visible for ~3 s before the event stops the run
