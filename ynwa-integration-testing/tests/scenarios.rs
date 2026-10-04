//! Integration harness: runs every scenario under `scenarios/` and asserts they all pass.
//!
//! `cargo test -p ynwa-integration-testing` executes this test. Set `YNWA_SCENARIO=<name>` to run
//! a single scenario while debugging; an unknown name fails with the list of available ones.

use std::env;
use std::path::{Path, PathBuf};

use ynwa_integration_testing::{run_scenario, ScenarioError, ScenarioReport};

/// Root directory holding one subdirectory per scenario.
fn scenarios_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scenarios")
}

/// Scenario directories: immediate subdirectories that contain `scenario.toml`, sorted by name.
fn discover_scenarios() -> Vec<PathBuf> {
    let root = scenarios_root();
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(&root)
        .unwrap_or_else(|error| panic!("cannot read '{}': {error}", root.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.join("scenario.toml").is_file())
        .collect();
    dirs.sort();
    dirs
}

fn scenario_name(dir: &Path) -> String {
    dir.file_name()
        .expect("scenario directory has a name")
        .to_string_lossy()
        .into_owned()
}

/// Restricts the discovered scenarios to the one named by `filter`, if it is set. An unknown name
/// yields an error listing the available scenario names.
fn select_scenarios(all: Vec<PathBuf>, filter: Option<&str>) -> Result<Vec<PathBuf>, String> {
    let Some(filter) = filter else {
        return Ok(all);
    };
    let selected: Vec<PathBuf> = all
        .iter()
        .filter(|dir| scenario_name(dir) == filter)
        .cloned()
        .collect();
    if selected.is_empty() {
        let available: Vec<String> = all.iter().map(|dir| scenario_name(dir)).collect();
        return Err(format!(
            "YNWA_SCENARIO='{filter}' matches no scenario; available: {}",
            available.join(", ")
        ));
    }
    Ok(selected)
}

/// Runs each selected scenario and aggregates the failures into a single report.
fn run_selected<F>(selected: &[PathBuf], run: F) -> Result<(), String>
where
    F: Fn(&Path) -> Result<ScenarioReport, ScenarioError>,
{
    let mut failures = Vec::new();
    for dir in selected {
        let name = scenario_name(dir);
        match run(dir) {
            Ok(report) if report.passed => {}
            Ok(report) => failures.push(format!(
                "scenario '{}' FAILED:\n{}",
                report.name,
                report.diff.unwrap_or_default()
            )),
            Err(error) => failures.push(format!("scenario '{name}' failed: {error}")),
        }
    }
    if failures.is_empty() {
        return Ok(());
    }
    Err(format!(
        "{} of {} scenarios failed:\n\n{}",
        failures.len(),
        selected.len(),
        failures.join("\n\n")
    ))
}

#[test]
fn run_all_scenarios() {
    let all = discover_scenarios();
    assert!(
        !all.is_empty(),
        "no scenarios found in '{}'",
        scenarios_root().display()
    );
    let selected = select_scenarios(all, env::var("YNWA_SCENARIO").ok().as_deref())
        .unwrap_or_else(|error| panic!("{error}"));

    run_selected(&selected, run_scenario).unwrap_or_else(|report| panic!("{report}"));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir(name: &str) -> PathBuf {
        PathBuf::from(name)
    }

    fn report(name: &str, passed: bool, diff: Option<&str>) -> ScenarioReport {
        ScenarioReport {
            name: name.to_string(),
            passed,
            diff: diff.map(str::to_string),
        }
    }

    #[test]
    fn scenario_name_is_the_last_component() {
        assert_eq!(scenario_name(Path::new("scenarios/corner")), "corner");
    }

    #[test]
    fn no_filter_selects_every_scenario() {
        let all = vec![dir("a"), dir("b")];
        assert_eq!(
            select_scenarios(all, None).unwrap(),
            vec![dir("a"), dir("b")]
        );
    }

    #[test]
    fn filter_selects_only_the_named_scenario() {
        let all = vec![dir("a"), dir("b")];
        assert_eq!(select_scenarios(all, Some("b")).unwrap(), vec![dir("b")]);
    }

    #[test]
    fn unknown_filter_is_an_error_listing_available_scenarios() {
        let all = vec![dir("corner"), dir("goal_into_a")];
        let error = select_scenarios(all, Some("nope")).unwrap_err();
        assert!(error.contains("nope"), "{error}");
        assert!(error.contains("corner"), "{error}");
        assert!(error.contains("goal_into_a"), "{error}");
    }

    #[test]
    fn passing_scenarios_produce_no_failure() {
        let selected = vec![dir("ok")];
        let outcome = run_selected(&selected, |_| Ok(report("ok", true, None)));
        assert!(outcome.is_ok());
    }

    #[test]
    fn failed_expectation_is_aggregated_into_the_report() {
        let selected = vec![dir("ok"), dir("bad")];
        let outcome = run_selected(&selected, |path| {
            if scenario_name(path) == "bad" {
                Ok(report("bad", false, Some("positions differ")))
            } else {
                Ok(report("ok", true, None))
            }
        });
        let error = outcome.unwrap_err();
        assert!(error.contains("1 of 2 scenarios failed"), "{error}");
        assert!(error.contains("positions differ"), "{error}");
    }

    #[test]
    fn load_error_is_aggregated_into_the_report() {
        let selected = vec![dir("broken")];
        let outcome = run_selected(&selected, |_| Err(ScenarioError::new("cannot read state")));
        let error = outcome.unwrap_err();
        assert!(error.contains("broken"), "{error}");
        assert!(error.contains("cannot read state"), "{error}");
    }
}
