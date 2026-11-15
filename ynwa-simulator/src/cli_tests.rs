use std::path::{Path, PathBuf};

use tempfile::TempDir;

use super::{
    collision_free_path, default_recordings_dir, parse, recording_path, DEFAULT_DURATION,
    DEFAULT_PREAMBLES_PATH, DEFAULT_TEAMS_PATH, DEFAULT_TEAM_A_ID, DEFAULT_TEAM_B_ID,
    DEFAULT_TEMPERATURE, DEFAULT_TIMESTEP,
};

/// Every value-taking flag with a valid value.
const FLAGS_WITH_VALUES: &[(&str, &str)] = &[
    ("--teams", "custom-teams"),
    ("--preambles", "custom-preambles"),
    ("--team-a", "pressing"),
    ("--team-b", "counter"),
    ("--out", "recordings/match01.jsonl"),
    ("--seed", "42"),
    ("--temperature", "0.5"),
    ("--timestep", "0.05"),
    ("--duration", "90"),
];

fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| value.to_string()).collect()
}

fn parse_error(values: &[&str]) -> String {
    match parse(&args(values)) {
        Ok(_) => panic!("expected an error for {values:?}"),
        Err(error) => error,
    }
}

#[test]
fn defaults_match_the_documented_values() {
    let cli = parse(&args(&[])).unwrap();

    assert_eq!(cli.teams_path, PathBuf::from(DEFAULT_TEAMS_PATH));
    assert_eq!(cli.preambles_path, PathBuf::from(DEFAULT_PREAMBLES_PATH));
    assert_eq!(cli.team_a_id, DEFAULT_TEAM_A_ID);
    assert_eq!(cli.team_b_id, DEFAULT_TEAM_B_ID);
    assert_eq!(cli.out, None);
    assert_eq!(cli.seed, None);
    assert_eq!(cli.temperature, DEFAULT_TEMPERATURE);
    assert_eq!(cli.timestep, DEFAULT_TIMESTEP);
    assert_eq!(cli.duration, DEFAULT_DURATION);
}

#[test]
fn flags_override_the_defaults() {
    let cli = parse(&args(&[
        "--teams",
        "custom-teams",
        "--preambles",
        "custom-preambles",
        "--team-a",
        "pressing",
        "--team-b",
        "counter",
        "--out",
        "recordings/match01.jsonl",
        "--seed",
        "42",
        "--temperature",
        "0.25",
        "--timestep",
        "0.05",
        "--duration",
        "90",
    ]))
    .unwrap();

    assert_eq!(cli.teams_path, PathBuf::from("custom-teams"));
    assert_eq!(cli.preambles_path, PathBuf::from("custom-preambles"));
    assert_eq!(cli.team_a_id, "pressing");
    assert_eq!(cli.team_b_id, "counter");
    assert_eq!(cli.out, Some(PathBuf::from("recordings/match01.jsonl")));
    assert_eq!(cli.seed, Some(42));
    assert_eq!(cli.temperature, 0.25);
    assert_eq!(cli.timestep, 0.05);
    assert_eq!(cli.duration, 90.0);
}

#[test]
fn duplicate_flags_are_rejected() {
    for &(flag, value) in FLAGS_WITH_VALUES {
        let error = parse_error(&[flag, value, flag, value]);
        assert!(
            error.contains("more than once"),
            "{flag}: unexpected error: {error}"
        );
    }
}

#[test]
fn missing_flag_values_are_rejected() {
    for &(flag, _) in FLAGS_WITH_VALUES {
        for values in [vec![flag], vec![flag, "--duration", "10"]] {
            let error = parse_error(&values);
            assert!(
                error.contains("requires a value"),
                "{flag}: unexpected error: {error}"
            );
        }
    }
}

#[test]
fn unknown_flags_are_rejected() {
    let error = parse_error(&["--speed", "1"]);
    assert!(
        error.contains("unknown flag") && error.contains("--speed"),
        "unexpected error: {error}"
    );
}

#[test]
fn positional_arguments_are_rejected() {
    let error = parse_error(&["match01.jsonl"]);
    assert!(
        error.contains("positional argument") && error.contains("match01.jsonl"),
        "unexpected error: {error}"
    );
}

#[test]
fn temperature_accepts_the_closed_unit_interval() {
    for value in ["0", "0.5", "1"] {
        let cli = parse(&args(&["--temperature", value])).unwrap();
        assert_eq!(cli.temperature, value.parse::<f32>().unwrap());
    }
}

#[test]
fn temperature_outside_the_unit_interval_is_rejected() {
    for value in ["-0.1", "1.1", "nan", "inf", "abc"] {
        let error = parse_error(&["--temperature", value]);
        assert!(
            error.contains("--temperature"),
            "{value}: unexpected error: {error}"
        );
    }
}

#[test]
fn timestep_and_duration_must_be_positive_and_finite() {
    for flag in ["--timestep", "--duration"] {
        for value in ["0", "-1", "nan", "inf", "abc"] {
            let error = parse_error(&[flag, value]);
            assert!(
                error.contains(flag),
                "{flag} {value}: unexpected error: {error}"
            );
        }
    }
}

#[test]
fn seed_must_be_an_unsigned_integer() {
    for value in ["-1", "1.5", "abc"] {
        let error = parse_error(&["--seed", value]);
        assert!(
            error.contains("--seed"),
            "{value}: unexpected error: {error}"
        );
    }

    let max = parse(&args(&["--seed", "18446744073709551615"])).unwrap();
    assert_eq!(max.seed, Some(u64::MAX));
}

#[test]
fn explicit_jsonl_path_is_used_as_is_and_its_parents_are_created() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("nested/deep/match01.jsonl");

    let path = recording_path(Some(&out), "pressing", "counter").unwrap();

    assert_eq!(path, out);
    assert!(out.parent().unwrap().is_dir());
}

#[test]
fn a_bare_jsonl_filename_needs_no_parent_directory() {
    let path = recording_path(Some(Path::new("match01.jsonl")), "team_a", "team_b").unwrap();

    assert_eq!(path, PathBuf::from("match01.jsonl"));
}

#[test]
fn directory_out_gets_a_descriptive_timestamped_filename() {
    let dir = TempDir::new().unwrap();

    let path = recording_path(Some(dir.path()), "pressing", "counter").unwrap();

    assert_eq!(path.parent().unwrap(), dir.path());
    let name = path.file_name().unwrap().to_str().unwrap();
    let stamp = name
        .strip_prefix("pressing_vs_counter_")
        .and_then(|rest| rest.strip_suffix(".jsonl"))
        .unwrap_or_else(|| panic!("unexpected recording name: {name}"));
    assert_timestamp(stamp);
}

#[test]
fn a_missing_out_directory_is_created() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("recordings");

    let path = recording_path(Some(&out), "team_a", "team_b").unwrap();

    assert!(out.is_dir());
    assert_eq!(path.parent().unwrap(), out);
}

#[test]
fn a_path_with_another_extension_is_treated_as_a_directory() {
    let dir = TempDir::new().unwrap();
    let out = dir.path().join("archive.tar");

    let path = recording_path(Some(&out), "team_a", "team_b").unwrap();

    assert!(out.is_dir());
    assert_eq!(path.parent().unwrap(), out);
}

#[test]
fn name_collisions_append_a_numeric_suffix() {
    let dir = TempDir::new().unwrap();
    let base = "team_a_vs_team_b_20260101-120000";

    for expected in ["20260101-120000.jsonl", "20260101-120000_2.jsonl"] {
        let path = collision_free_path(dir.path(), base);
        assert_eq!(
            path.file_name().unwrap().to_str().unwrap(),
            format!("team_a_vs_team_b_{}", expected)
        );
        std::fs::write(&path, "").unwrap();
    }

    let path = collision_free_path(dir.path(), base);
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        "team_a_vs_team_b_20260101-120000_3.jsonl"
    );
}

#[test]
fn the_default_recordings_directory_lives_under_the_home_directory() {
    let dir = default_recordings_dir().unwrap();

    assert!(
        dir.ends_with(Path::new(".ynwa/games")),
        "unexpected default recordings directory: {}",
        dir.display()
    );
}

fn assert_timestamp(stamp: &str) {
    let bytes = stamp.as_bytes();
    assert_eq!(
        bytes.len(),
        15,
        "timestamp '{}' must be YYYYMMDD-HHMMSS",
        stamp
    );
    assert!(
        bytes[..8].iter().all(u8::is_ascii_digit),
        "date part of '{}' must be digits",
        stamp
    );
    assert_eq!(
        bytes[8], b'-',
        "timestamp '{}' must separate date and time with '-'",
        stamp
    );
    assert!(
        bytes[9..].iter().all(u8::is_ascii_digit),
        "time part of '{}' must be digits",
        stamp
    );
}
