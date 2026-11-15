use super::{parse, DEFAULT_MATCH_DURATION, DEFAULT_TEAM_A_ID, DEFAULT_TEAM_B_ID};

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
fn team_ids_default_to_the_standard_teams() {
    let cli = parse(&args(&[])).unwrap();
    assert_eq!(cli.team_a_id, DEFAULT_TEAM_A_ID);
    assert_eq!(cli.team_b_id, DEFAULT_TEAM_B_ID);
}

#[test]
fn team_flags_override_the_defaults_in_play_and_record_modes() {
    let play = parse(&args(&["--team-a", "pressing", "--team-b", "counter"])).unwrap();
    assert_eq!(play.team_a_id, "pressing");
    assert_eq!(play.team_b_id, "counter");

    let record = parse(&args(&["--record", "--team-a", "pressing"])).unwrap();
    assert_eq!(record.team_a_id, "pressing");
    assert_eq!(record.team_b_id, DEFAULT_TEAM_B_ID);
}

#[test]
fn duplicate_team_flags_are_rejected() {
    for flag in ["--team-a", "--team-b"] {
        let error = parse_error(&[flag, "one", flag, "two"]);
        assert!(
            error.contains("more than once"),
            "{flag}: unexpected error: {error}"
        );
    }
}

#[test]
fn team_flags_are_rejected_with_replay() {
    for flag in ["--team-a", "--team-b"] {
        let error = parse_error(&["--replay", "game.jsonl", flag, "pressing"]);
        assert!(
            error.contains(flag) && error.contains("--replay"),
            "{flag}: unexpected error: {error}"
        );
    }
}

#[test]
fn missing_team_flag_values_are_rejected() {
    for missing in [
        &["--team-a"][..],
        &["--team-b"][..],
        &["--team-a", "--record"][..],
    ] {
        let error = parse_error(missing);
        assert!(
            error.contains("requires a value"),
            "unexpected error: {error}"
        );
    }
}

#[test]
fn duration_defaults_to_the_match_duration() {
    assert_eq!(parse(&args(&[])).unwrap().duration, DEFAULT_MATCH_DURATION);
}

#[test]
fn duration_flag_overrides_the_match_duration() {
    let play = parse(&args(&["--duration", "45.5"])).unwrap();
    assert_eq!(play.duration, 45.5);

    let record = parse(&args(&["--record", "--duration", "300"])).unwrap();
    assert_eq!(record.duration, 300.0);
}

#[test]
fn invalid_duration_is_rejected() {
    for value in ["0", "-1", "nan", "inf", "abc"] {
        let error = parse_error(&["--duration", value]);
        assert!(
            error.contains("--duration"),
            "{value}: unexpected error: {error}"
        );
    }
}

#[test]
fn duration_flag_conflicts_are_rejected() {
    let duplicate = parse_error(&["--duration", "10", "--duration", "20"]);
    assert!(
        duplicate.contains("more than once"),
        "unexpected error: {duplicate}"
    );

    let replay = parse_error(&["--replay", "game.jsonl", "--duration", "10"]);
    assert!(
        replay.contains("--duration") && replay.contains("--replay"),
        "unexpected error: {replay}"
    );

    for missing in [&["--duration"][..], &["--duration", "--record"][..]] {
        let error = parse_error(missing);
        assert!(
            error.contains("requires a value"),
            "unexpected error: {error}"
        );
    }
}
