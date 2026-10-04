use super::{run_scenario, ScenarioReport};

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn run_scenario_passes_for_the_valid_fixture() {
    let report = run_scenario(&fixture("valid")).expect("scenario must run");

    assert_eq!(
        report,
        ScenarioReport {
            name: "valid".to_string(),
            passed: true,
            diff: None,
        }
    );
}

#[test]
fn run_scenario_reports_error_for_missing_directory() {
    let error = run_scenario(&fixture("does_not_exist")).unwrap_err();

    assert!(
        error.message().contains("initial_state.toml"),
        "unexpected error: {error}"
    );
}

#[test]
fn run_scenario_reports_failed_expectations() {
    let report = run_scenario(&fixture("failing")).expect("scenario must run");

    assert_eq!(report.name, "failing");
    assert!(!report.passed);
    let diff = report.diff.expect("a failed report carries a diff");
    assert!(
        diff.contains("final_state.stage"),
        "unexpected diff: {diff}"
    );
    assert!(
        diff.contains("final_state.score"),
        "unexpected diff: {diff}"
    );
}
