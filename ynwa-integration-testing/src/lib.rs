//! Integration testing of the full chain: core - standard library - player scripts.
//!
//! A scenario directory describes a deterministic simulation run. This crate turns such a
//! directory into a world, runs it and compares the outcome with the declared expectations.
//! It exposes the scenario model (parsing and validation of `initial_state.toml`,
//! `final_state.toml` and `scenario.toml`) and the loader that assembles a ready-to-run world.
//!
//! Run the scenarios with `cargo test -p ynwa-integration-testing`; set `YNWA_SCENARIO=<name>` to
//! run a single scenario.
//!
//! ## Scenarios
//!
//! Every directory under `scenarios/` that contains a `scenario.toml` is one case:
//!
//! ```text
//! scenarios/<name>/
//!   initial_state.toml   # partial start state (stage, ball, player positions)
//!   final_state.toml     # optional partial state checked after the run
//!   scenario.toml        # run plan ([run]) and expectations ([expect])
//!   teams/               # FsTeamRepository tree: preambles, tactics, scripts
//! ```
//!
//! [`tests/scenarios.rs`] runs them all and fails with a per-scenario diff if an expectation does
//! not hold. Runs are fully deterministic: the runner uses a fixed RNG seed and zero temperature.

pub mod compare;
pub mod criterion;
pub mod dto;
pub mod loader;
pub mod runner;
pub mod scenario;

pub use compare::{compare_outcome, TOLERANCE};
pub use criterion::{EventMatcher, StageMatcher, StopCriterion, StopReason};
pub use dto::{
    BallOwnerDto, BallStateDto, EventKindDto, ExpectDto, ExpectedEventDto, ExpectedFinalStateDto,
    InitialStateDto, JournalMatchDto, NoneToken, PlayerPlacementDto, PlayerRefDto, RunPlanDto,
    ScenarioDto, SetupDto, StageNameDto, StopCriterionDto, StopExpectationDto, TeamOrNoneDto,
};
pub use loader::{load_scenario, LoadedScenario, ScenarioError, ScenarioLoader};
pub use runner::{run, RunOutcome};
pub use scenario::{run_scenario, ScenarioReport};
