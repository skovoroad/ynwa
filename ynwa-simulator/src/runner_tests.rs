use std::path::{Path, PathBuf};

use tempfile::TempDir;
use ynwa_core::team::Team;
use ynwa_core::{json_journal_file_reader, GameStage, Record, RecordReader};

use super::run;
use crate::cli::Cli;

fn workspace_path(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(relative)
}

fn cli_with(out: PathBuf, timestep: f32, duration: f32) -> Cli {
    Cli {
        teams_path: workspace_path("../teams"),
        preambles_path: workspace_path("../ynwa-scripts/preambles"),
        team_a_id: "team_a".to_string(),
        team_b_id: "team_b".to_string(),
        out: Some(out),
        seed: Some(11),
        temperature: 0.7,
        timestep,
        duration,
    }
}

fn read_record(path: &Path) -> Record {
    let mut reader = json_journal_file_reader(path).unwrap();
    reader.read().unwrap()
}

#[test]
fn run_steps_until_the_duration_elapses_and_finalizes_the_recording() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("nested/match.jsonl");

    let written = run(&cli_with(path.clone(), 0.25, 1.0)).unwrap();

    assert_eq!(written, path);
    let record = read_record(&path);
    assert_eq!(record.header.timestep, 0.25);
    assert_eq!(record.header.initial_state.elapsed_time, 0.0);
    assert_eq!(
        record.header.initial_state.stage,
        GameStage::Setup("kick off".to_string())
    );
    assert_eq!(record.footer.total_steps, 4);
    assert_eq!(record.footer.elapsed_time, 1.0);
    assert_eq!(
        record.footer.player_stats.len(),
        record.header.config.players.len()
    );
    assert!(record.footer.team_stats.contains_key(&Team::A));
    assert!(record.footer.team_stats.contains_key(&Team::B));
}

#[test]
fn a_duration_shorter_than_the_timestep_runs_exactly_one_step() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("short.jsonl");

    run(&cli_with(path.clone(), 0.25, 0.1)).unwrap();

    let record = read_record(&path);
    assert_eq!(record.footer.total_steps, 1);
    assert_eq!(record.footer.elapsed_time, 0.25);
}

#[test]
fn identical_parameters_produce_identical_recordings() {
    let dir = TempDir::new().unwrap();
    let first = cli_with(dir.path().join("first.jsonl"), 0.25, 1.0);
    let second = cli_with(dir.path().join("second.jsonl"), 0.25, 1.0);

    let first_path = run(&first).unwrap();
    let second_path = run(&second).unwrap();

    assert_ne!(first_path, second_path);
    assert_eq!(read_record(&first_path), read_record(&second_path));
}

#[test]
fn a_directory_out_receives_a_descriptive_filename() {
    let dir = TempDir::new().unwrap();

    let path = run(&cli_with(dir.path().to_path_buf(), 0.25, 0.25)).unwrap();

    assert_eq!(path.parent().unwrap(), dir.path());
    let name = path.file_name().unwrap().to_str().unwrap();
    assert!(
        name.starts_with("team_a_vs_team_b_") && name.ends_with(".jsonl"),
        "unexpected recording name: {name}"
    );
}

#[test]
fn an_unknown_team_fails_instead_of_falling_back_to_placeholders() {
    let dir = TempDir::new().unwrap();
    let mut cli = cli_with(dir.path().join("match.jsonl"), 0.25, 0.25);
    cli.team_a_id = "missing".to_string();

    let error = run(&cli).unwrap_err();

    assert!(error.contains("missing"), "unexpected error: {error}");
}

#[test]
fn missing_preambles_fail_the_run() {
    let dir = TempDir::new().unwrap();
    let mut cli = cli_with(dir.path().join("match.jsonl"), 0.25, 0.25);
    cli.preambles_path = dir.path().join("absent-preambles");

    let error = run(&cli).unwrap_err();

    assert!(error.contains("core.lua"), "unexpected error: {error}");
    assert!(!dir.path().join("match.jsonl").exists());
}
