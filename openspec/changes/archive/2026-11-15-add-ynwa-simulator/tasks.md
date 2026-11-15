# Tasks

> **Prerequisite**: Complete and apply `configurable-match-teams`, `record-footer-stats`, and `client-owned-match-duration` before starting these tasks.

## 1. Crate Scaffolding and CLI Parser

- [x] 1.1 Create the `ynwa-simulator` binary crate (`ynwa-simulator/Cargo.toml` and `ynwa-simulator/src/main.rs`), register it in the root `Cargo.toml` workspace members, and verify `cargo check -p ynwa-simulator` succeeds.
- [x] 1.2 Implement `ynwa-simulator/src/cli.rs` parsing `--teams`, `--preambles`, `--team-a`, `--team-b`, `--out`, `--seed`, `--temperature`, `--timestep`, and `--duration` plus output path resolution (`.jsonl` file vs directory with `<team_a>_vs_<team_b>_YYYYMMDD-HHMMSS[_2..].jsonl`), and verify parameterized unit tests in `ynwa-simulator/src/cli_tests.rs` pass via `cargo test -p ynwa-simulator`.

## 2. Headless Simulation Runner and End-to-End Verification

- [x] 2.1 Implement the synchronous model-time simulation loop (`while world.game().state().elapsed_time < cli.duration`) in `ynwa-simulator/src/runner.rs` and `ynwa-simulator/src/main.rs` using `FootballWorldBuilder` (with configured teams, RNG seed/temperature, and no placeholder fallback) and `FileJournalRecorder`, and verify unit/integration tests in `ynwa-simulator/src/runner_tests.rs` pass via `cargo test -p ynwa-simulator`.
- [x] 2.2 Verify end-to-end execution by running `cargo test`, `cargo clippy`, and `cargo fmt --check` across the entire workspace.
