//! Running a scenario directory end to end and reporting whether its expectations held.

use std::path::{Path, PathBuf};

use ynwa_core::record::Record;
use ynwa_core::write_json_record;

use crate::compare::compare_outcome;
use crate::loader::{load_scenario, LoadedScenario, ScenarioError};
use crate::runner::run;

/// Directory, relative to the home directory, where every scenario run leaves its recording. Games
/// recorded by the player live next to it in `.ynwa/games`.
const RECORDINGS_DIR: &str = ".ynwa/scenarios";

/// Result of running one scenario: its name, whether every expectation matched, and, on failure,
/// a human-readable diff of all mismatches.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioReport {
    pub name: String,
    pub passed: bool,
    pub diff: Option<String>,
}

/// Loads, runs and checks the scenario rooted at `dir`.
///
/// The run is written to [`recording_path`] before the expectations are compared, so a failed
/// scenario still leaves a recording for the player.
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
    write_record(&name, &outcome.record)?;
    let diffs = compare_outcome(
        &outcome,
        &expect,
        final_state.as_ref(),
        &config,
        plan.timestep,
    )?;

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

/// Path of the recording `run_scenario` leaves for the scenario `name`.
pub fn recording_path(name: &str) -> Result<PathBuf, ScenarioError> {
    let home =
        env_home::env_home_dir().ok_or_else(|| ScenarioError::new("home directory not found"))?;
    Ok(home.join(RECORDINGS_DIR).join(format!("{name}.jsonl")))
}

fn write_record(name: &str, record: &Record) -> Result<(), ScenarioError> {
    let path = recording_path(name)?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|error| {
            ScenarioError::new(format!("cannot create '{}': {error}", dir.display()))
        })?;
    }
    write_json_record(&path, record).map_err(ScenarioError::new)
}

#[cfg(test)]
#[path = "tests/scenario_tests.rs"]
mod tests;
