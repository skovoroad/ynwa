# Tasks

## 1. Runner trailing step

- [ ] 1.1 In `ynwa-integration-testing/src/runner.rs`, after the stop loop, run one extra `world.step(run.timestep)` and increment `steps` when `stop_reason` is `StopReason::Event`, preserving the stop reason and not re-evaluating criteria; verify `cargo test -p ynwa-integration-testing` passes (unit, `tests/replay.rs`, and all scenarios)
- [ ] 1.2 Add a runner unit test in `ynwa-integration-testing/src/tests/runner_tests.rs` asserting an event stop records the trailing step (journal gains the trailing tick, `steps` is the triggering step + 1, stop reason is unchanged); verify `cargo test -p ynwa-integration-testing runner::tests` passes
- [ ] 1.3 Run `cargo clippy --workspace --all-targets` and `cargo fmt --all -- --check` and confirm no warnings
