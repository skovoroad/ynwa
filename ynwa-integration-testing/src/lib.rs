//! Integration testing of the full chain: core - standard library - player scripts.
//!
//! A scenario directory describes a deterministic simulation run. This crate turns such a
//! directory into a world, runs it and compares the outcome with the declared expectations.
//! Currently it exposes only the scenario model: parsing and validation of `initial_state.toml`,
//! `final_state.toml` and `scenario.toml`.
//!
//! Run the scenarios with `cargo test -p ynwa-integration-testing`; set `YNWA_SCENARIO=<name>` to
//! run a single scenario.

pub mod scenario;

#[cfg(test)]
#[path = "tests/scenario_tests.rs"]
mod scenario_tests;

pub use scenario::{
    BallOwner, BallStateDef, EventKind, EventMatcher, Expect, ExpectedEventDef, FinalState,
    InitialState, JournalMatch, NoneToken, PlayerPlacement, PlayerRef, RunPlan, ScenarioDef,
    SetupDef, StageName, StopCriterionDef, StopWhen, TeamOrNone,
};
