//! Running a scenario directory end to end and reporting whether its expectations held.

use std::path::Path;

use crate::compare::compare_outcome;
use crate::loader::{load_scenario, LoadedScenario, ScenarioError};
use crate::runner::run;

/// Result of running one scenario: its name, whether every expectation matched, and, on failure,
/// a human-readable diff of all mismatches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioReport {
    pub name: String,
    pub passed: bool,
    pub diff: Option<String>,
}

/// Loads, runs and checks the scenario rooted at `dir`.
pub fn run_scenario(dir: &Path) -> Result<ScenarioReport, ScenarioError> {
    let LoadedScenario {
        name,
        world,
        run: plan,
        expect,
        final_state,
    } = load_scenario(dir)?;
    let config = world.game().config().clone();
    let outcome = run(world, &plan)?;
    let diffs = compare_outcome(&outcome, &expect, final_state.as_ref(), &config, plan.dt)?;

    let diff = if diffs.is_empty() {
        None
    } else {
        Some(diffs.join("\n"))
    };
    Ok(ScenarioReport {
        name,
        passed: diff.is_none(),
        diff,
    })
}

#[cfg(test)]
#[path = "tests/scenario_tests.rs"]
mod tests;
