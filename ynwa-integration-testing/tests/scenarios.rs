//! Integration harness: runs every scenario under `scenarios/` and asserts they all pass.
//!
//! `cargo test -p ynwa-integration-testing` executes this test. Set `YNWA_SCENARIO=<name>` to run
//! a single scenario while debugging; an unknown name fails with the list of available ones.

use std::env;
use std::path::{Path, PathBuf};

use ynwa_integration_testing::run_scenario;

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

/// Restricts the discovered scenarios to the one named by `YNWA_SCENARIO`, if that variable is set.
fn select_scenarios(all: Vec<PathBuf>) -> Vec<PathBuf> {
    let Ok(filter) = env::var("YNWA_SCENARIO") else {
        return all;
    };
    let selected: Vec<PathBuf> = all
        .iter()
        .filter(|dir| scenario_name(dir) == filter)
        .cloned()
        .collect();
    if selected.is_empty() {
        let available: Vec<String> = all.iter().map(|dir| scenario_name(dir)).collect();
        panic!(
            "YNWA_SCENARIO='{filter}' matches no scenario; available: {}",
            available.join(", ")
        );
    }
    selected
}

#[test]
fn run_all_scenarios() {
    let all = discover_scenarios();
    assert!(
        !all.is_empty(),
        "no scenarios found in '{}'",
        scenarios_root().display()
    );
    let selected = select_scenarios(all);

    let mut failures = Vec::new();
    for dir in &selected {
        let name = scenario_name(dir);
        match run_scenario(dir) {
            Ok(report) if report.passed => {}
            Ok(report) => failures.push(format!(
                "scenario '{}' FAILED:\n{}",
                report.name,
                report.diff.unwrap_or_default()
            )),
            Err(error) => failures.push(format!("scenario '{name}' failed: {error}")),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} scenarios failed:\n\n{}",
        failures.len(),
        selected.len(),
        failures.join("\n\n")
    );
}
