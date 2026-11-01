use super::{recording_path, run_scenario, ScenarioReport};
use crate::loader::{load_scenario, LoadedScenario};
use crate::runner::run;
use ynwa_core::{json_journal_file_reader, RecordReader};

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn run_scenario_passes_and_writes_the_record_it_ran() {
    let LoadedScenario {
        world, run: plan, ..
    } = load_scenario(&fixture("valid")).expect("fixture must load");
    let expected = run(world, &plan).expect("trivial run must succeed").record;

    let report = run_scenario(&fixture("valid")).expect("scenario must run");

    assert_eq!(
        report,
        ScenarioReport {
            name: "valid".to_string(),
            passed: true,
            diff: None,
        }
    );

    let path = recording_path("valid").expect("home directory must be available");
    let actual = json_journal_file_reader(path)
        .expect("recording must open")
        .read()
        .expect("recording must parse");
    assert_eq!(actual, expected);
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

    let path = recording_path("failing").expect("home directory must be available");
    let recording = json_journal_file_reader(path)
        .expect("a failed scenario must still leave a recording")
        .read()
        .expect("recording must parse");
    assert_eq!(recording.total_steps, 10);
}
