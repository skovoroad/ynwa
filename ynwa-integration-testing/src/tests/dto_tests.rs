use crate::dto::NoneToken;
use crate::*;
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Score, Velocity3D};

fn parse_initial(source: &str) -> InitialStateDto {
    InitialStateDto::parse(source).expect("initial state must parse")
}

fn parse_final(source: &str) -> ExpectedFinalStateDto {
    ExpectedFinalStateDto::parse(source).expect("final state must parse")
}

fn parse_scenario(source: &str) -> ScenarioDto {
    ScenarioDto::parse(source).expect("scenario must parse")
}

#[test]
fn minimal_initial_state_is_empty() {
    assert_eq!(parse_initial(""), InitialStateDto::default());
}

#[test]
fn initial_state_play_with_ball_and_players() {
    let state = parse_initial(
        r#"
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 10.0 }
velocity = { x = 1.0, y = 0.0, z = 2.0 }
possessed_by = { team = "A", number = 9 }
last_possessing_team = "B"

[[players]]
team = "A"
number = 9
position = { x = 34.0, y = 0.0, z = 10.0 }
"#,
    );

    assert_eq!(state.stage, Some(StageNameDto::Play));
    let ball = state.ball.expect("ball present");
    assert_eq!(ball.position, Some(Point3D::from_meters(34.0, 0.0, 10.0)));
    assert_eq!(
        ball.velocity,
        Some(Velocity3D::from_meters_per_second(1.0, 0.0, 2.0))
    );
    assert_eq!(
        ball.possessed_by,
        Some(BallOwnerDto::Player(PlayerRefDto {
            team: Team::A,
            number: 9
        }))
    );
    assert_eq!(
        ball.last_possessing_team,
        Some(TeamOrNoneDto::Team(Team::B))
    );
    assert_eq!(
        state.players,
        vec![PlayerPlacementDto {
            team: Team::A,
            number: 9,
            position: Point3D::from_meters(34.0, 0.0, 10.0),
        }]
    );
}

#[test]
fn initial_state_setup_with_restart() {
    let state = parse_initial(
        r#"
stage = "Setup"
setup_reason = "throw in"

[setup]
restart_position = { x = 0.0, y = 0.0, z = 5.5 }
restart_team = "A"
"#,
    );

    assert_eq!(state.stage, Some(StageNameDto::Setup));
    assert_eq!(state.setup_reason.as_deref(), Some("throw in"));
    let setup = state.setup.expect("setup present");
    assert_eq!(
        setup.restart_position,
        Some(Point3D::from_meters(0.0, 0.0, 5.5))
    );
    assert_eq!(setup.restart_team, Some(TeamOrNoneDto::Team(Team::A)));
}

#[test]
fn ball_owner_and_team_accept_none_marker() {
    let state = parse_initial(
        r#"
stage = "Play"

[ball]
possessed_by = "none"
last_possessing_team = "none"
"#,
    );

    let ball = state.ball.expect("ball present");
    assert_eq!(ball.possessed_by, Some(BallOwnerDto::Free(NoneToken)));
    assert_eq!(
        ball.last_possessing_team,
        Some(TeamOrNoneDto::None(NoneToken))
    );
}

#[test]
fn setup_stage_forbids_ball_state() {
    let error = InitialStateDto::parse(
        "stage = \"Setup\"\n[ball]\nposition = { x = 1.0, y = 0.0, z = 1.0 }\n",
    )
    .unwrap_err();

    assert!(error.contains("ball"), "unexpected error: {error}");
}

#[test]
fn omitted_stage_resolves_to_setup_and_forbids_ball() {
    let error =
        InitialStateDto::parse("[ball]\nposition = { x = 1.0, y = 0.0, z = 1.0 }\n").unwrap_err();

    assert!(error.contains("Setup"), "unexpected error: {error}");
}

#[test]
fn zero_player_number_is_rejected() {
    let cases = [
        (
            "[[players]]\nteam = \"A\"\nnumber = 0\nposition = { x = 0.0, y = 0.0, z = 0.0 }\n",
            "placement",
        ),
        (
            "stage = \"Play\"\n[ball]\npossessed_by = { team = \"B\", number = 0 }\n",
            "ball owner",
        ),
    ];

    for (source, label) in cases {
        let error = InitialStateDto::parse(source).unwrap_err();
        assert!(
            error.contains("number must be positive"),
            "{label}: unexpected error: {error}"
        );
    }
}

#[test]
fn final_state_allows_setup_ball_and_parses_score() {
    let state = parse_final(
        r#"
stage = "Setup"
setup_reason = "throw in"
score = { A = 1, B = 0 }

[ball]
position = { x = 5.0, y = 0.0, z = 5.0 }
possessed_by = "none"

[setup]
restart_team = "B"
"#,
    );

    assert_eq!(state.stage, Some(StageNameDto::Setup));
    assert_eq!(state.setup_reason.as_deref(), Some("throw in"));
    assert_eq!(state.score, Some(Score { a: 1, b: 0 }));
    assert_eq!(
        state.ball.expect("ball present").possessed_by,
        Some(BallOwnerDto::Free(NoneToken))
    );
    assert_eq!(
        state.setup.expect("setup present").restart_team,
        Some(TeamOrNoneDto::Team(Team::B))
    );
}

#[test]
fn invalid_team_label_is_rejected() {
    assert!(
        InitialStateDto::parse("stage = \"Play\"\n[ball]\nlast_possessing_team = \"C\"\n").is_err()
    );
}

#[test]
fn invalid_ball_owner_is_rejected() {
    assert!(
        InitialStateDto::parse("stage = \"Play\"\n[ball]\npossessed_by = \"bogus\"\n").is_err()
    );
}

#[test]
fn invalid_stage_name_is_rejected() {
    assert!(InitialStateDto::parse("stage = \"Half\"\n").is_err());
}

#[test]
fn run_plan_parses_all_criteria() {
    let plan = RunPlanDto::parse(
        r#"
dt = 0.1

[[stop]]
when = "stage"
stage = "Setup"
setup_reason = "goal kick"

[[stop]]
when = "event"
event = "Goal"
team = "B"

[[stop]]
when = "steps"
steps = 10000

[[stop]]
when = "time"
time = 60.0
"#,
    )
    .expect("run plan must parse");

    assert_eq!(plan.dt, 0.1);
    assert_eq!(
        plan.stop,
        vec![
            StopCriterionDto::Stage {
                stage: StageNameDto::Setup,
                setup_reason: Some("goal kick".to_string()),
            },
            StopCriterionDto::Event {
                event: EventKindDto::Goal,
                team: Some(Team::B),
            },
            StopCriterionDto::Steps { steps: 10000 },
            StopCriterionDto::Time { time: 60.0 },
        ]
    );
}

#[test]
fn run_plan_requires_a_safety_criterion() {
    let error =
        RunPlanDto::parse("dt = 0.1\n[[stop]]\nwhen = \"stage\"\nstage = \"Play\"\n").unwrap_err();

    assert!(error.contains("steps"), "unexpected error: {error}");
}

#[test]
fn run_plan_requires_positive_dt() {
    for dt in ["0.0", "-0.5"] {
        let source = format!("dt = {dt}\n[[stop]]\nwhen = \"steps\"\nsteps = 1\n");
        assert!(
            RunPlanDto::parse(&source).is_err(),
            "dt = {dt} must be rejected"
        );
    }
}

#[test]
fn invalid_stop_when_is_rejected() {
    assert!(RunPlanDto::parse("dt = 0.1\n[[stop]]\nwhen = \"bogus\"\n").is_err());
}

#[test]
fn scenario_parses_run_and_expect() {
    let scenario = parse_scenario(
        r#"
[run]
dt = 0.1

[[run.stop]]
when = "event"
event = "Goal"
team = "B"

[[run.stop]]
when = "steps"
steps = 10000

[expect]
journal_match = "subsequence"

[expect.stop]
when = "event"
event = "Goal"
team = "B"
steps = 15

[[expect.journal]]
type = "stage_change"
stage = "Setup"
at = 1.5

[[expect.journal]]
type = "football_event"
event = "Goal"
team = "B"
"#,
    );

    assert_eq!(scenario.run.dt, 0.1);
    assert_eq!(scenario.expect.journal_match, JournalMatchDto::Subsequence);
    assert_eq!(
        scenario.expect.stop,
        Some(StopExpectationDto::Event {
            event: EventKindDto::Goal,
            team: Some(Team::B),
            steps: Some(15),
        })
    );
    assert_eq!(
        scenario.expect.journal,
        vec![
            ExpectedEventDto::StageChange {
                stage: Some(StageNameDto::Setup),
                setup_reason: None,
                at: Some(1.5),
            },
            ExpectedEventDto::FootballEvent {
                event: EventKindDto::Goal,
                team: Some(Team::B),
                at: None,
            },
        ]
    );
}

#[test]
fn expect_defaults_when_omitted() {
    let scenario = parse_scenario("[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 3\n");

    assert_eq!(scenario.expect.journal_match, JournalMatchDto::Exact);
    assert!(scenario.expect.stop.is_none());
    assert!(scenario.expect.journal.is_empty());
}

#[test]
fn parses_every_journal_record_type() {
    let scenario = parse_scenario(
        r#"
[run]
dt = 0.1

[[run.stop]]
when = "steps"
steps = 10

[[expect.journal]]
type = "decision_assigned"
player = { team = "A", number = 9 }
decision = "Stop"
reason = "hold"

[[expect.journal]]
type = "possession_change"
possessed_by = { team = "B", number = 4 }
last_possessing_team = "none"

[[expect.journal]]
type = "kick_outcome"
player = { team = "A", number = 10 }
ball_velocity = { x = 1.0, y = 0.0, z = 2.0 }

[[expect.journal]]
type = "stage_change"
stage = "Setup"
setup_reason = "throw in"

[[expect.journal]]
type = "restart_set"
position = { x = 1.0, y = 0.0, z = 2.0 }
team = "A"

[[expect.journal]]
type = "decisions_reset"

[[expect.journal]]
type = "stat_update"
team = "A"
key = "score"
delta = 1.0

[[expect.journal]]
type = "football_event"
event = "Touchline"
team = "A"
"#,
    );

    use ExpectedEventDto::*;
    let journal = &scenario.expect.journal;
    assert_eq!(journal.len(), 8);
    assert_eq!(
        journal[0],
        DecisionAssigned {
            player: Some(PlayerRefDto {
                team: Team::A,
                number: 9
            }),
            decision: Some("Stop".to_string()),
            reason: Some("hold".to_string()),
            at: None,
        }
    );
    assert_eq!(
        journal[1],
        PossessionChange {
            possessed_by: Some(BallOwnerDto::Player(PlayerRefDto {
                team: Team::B,
                number: 4
            })),
            last_possessing_team: Some(TeamOrNoneDto::None(NoneToken)),
            at: None,
        }
    );
    assert_eq!(
        journal[2],
        KickOutcome {
            player: Some(PlayerRefDto {
                team: Team::A,
                number: 10
            }),
            ball_velocity: Some(Velocity3D::from_meters_per_second(1.0, 0.0, 2.0)),
            at: None,
        }
    );
    assert_eq!(
        journal[3],
        StageChange {
            stage: Some(StageNameDto::Setup),
            setup_reason: Some("throw in".to_string()),
            at: None,
        }
    );
    assert_eq!(
        journal[4],
        RestartSet {
            position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
            team: Some(TeamOrNoneDto::Team(Team::A)),
            at: None,
        }
    );
    assert_eq!(journal[5], DecisionsReset { at: None });
    assert_eq!(
        journal[6],
        StatUpdate {
            team: Some(Team::A),
            key: Some("score".to_string()),
            delta: Some(1.0),
            at: None,
        }
    );
    assert_eq!(
        journal[7],
        FootballEvent {
            event: EventKindDto::Touchline,
            team: Some(Team::A),
            at: None,
        }
    );
}

#[test]
fn stop_criterion_rejects_team_for_game_end() {
    let source = "dt = 0.1\n[[stop]]\nwhen = \"event\"\nevent = \"GameEnd\"\nteam = \"A\"\n[[stop]]\nwhen = \"steps\"\nsteps = 1\n";

    let error = RunPlanDto::parse(source).unwrap_err();

    assert!(error.contains("GameEnd"), "unexpected error: {error}");
}

#[test]
fn expect_stop_rejects_team_for_game_end() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 5\n[expect.stop]\nwhen = \"event\"\nevent = \"GameEnd\"\nteam = \"B\"\n",
    )
    .unwrap_err();

    assert!(error.contains("GameEnd"), "unexpected error: {error}");
}

#[test]
fn journal_football_event_rejects_team_for_game_end() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 5\n[[expect.journal]]\ntype = \"football_event\"\nevent = \"GameEnd\"\nteam = \"A\"\n",
    )
    .unwrap_err();

    assert!(error.contains("GameEnd"), "unexpected error: {error}");
}

#[test]
fn unknown_fields_are_rejected() {
    assert!(InitialStateDto::parse("stage = \"Play\"\nbogus = 1\n").is_err());
    assert!(ExpectedFinalStateDto::parse("stage = \"Play\"\nbogus = 1\n").is_err());
    assert!(
        RunPlanDto::parse("dt = 0.1\nbogus = 1\n[[stop]]\nwhen = \"steps\"\nsteps = 1\n").is_err()
    );
    assert!(InitialStateDto::parse("stage = \"Play\"\n[ball]\nbogus = 1\n").is_err());
    assert!(InitialStateDto::parse("stage = \"Setup\"\n[setup]\nbogus = 1\n").is_err());
    assert!(InitialStateDto::parse(
        "stage = \"Play\"\n[[players]]\nteam = \"A\"\nnumber = 1\nposition = { x = 0.0, y = 0.0, z = 0.0 }\nbogus = 1\n"
    )
    .is_err());
    assert!(ScenarioDto::parse(
        "bogus = 1\n[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 1\n"
    )
    .is_err());
    assert!(ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 1\n[[expect.journal]]\ntype = \"decision_assigned\"\nplayer = { team = \"A\", number = 1, bogus = 1 }\n"
    )
    .is_err());
}

#[test]
fn stop_criterion_rejects_unknown_field() {
    let error = RunPlanDto::parse("dt = 0.1\n[[stop]]\nwhen = \"steps\"\nsteps = 1\nbogus = 1\n")
        .unwrap_err();

    assert!(error.contains("bogus"), "unexpected error: {error}");
}

#[test]
fn expect_stop_rejects_unknown_field() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 1\n[expect.stop]\nwhen = \"steps\"\nsteps = 1\nbogus = 1\n",
    )
    .unwrap_err();

    assert!(error.contains("bogus"), "unexpected error: {error}");
}

#[test]
fn journal_entry_rejects_unknown_field() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 1\n[[expect.journal]]\ntype = \"decisions_reset\"\nbogus = 1\n",
    )
    .unwrap_err();

    assert!(error.contains("bogus"), "unexpected error: {error}");
}

#[test]
fn expect_stop_stage_requires_target_stage() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 5\n[expect.stop]\nwhen = \"stage\"\n",
    )
    .unwrap_err();

    assert!(error.contains("stage"), "unexpected error: {error}");
}

#[test]
fn stop_criterion_requires_its_payload() {
    let cases = [
        ("when = \"stage\"", "stage"),
        ("when = \"event\"", "event"),
        ("when = \"steps\"", "steps"),
        ("when = \"time\"", "time"),
    ];

    for (criterion, field) in cases {
        let source = format!("dt = 0.1\n[[stop]]\n{criterion}\n");
        let error = RunPlanDto::parse(&source).unwrap_err();
        assert!(
            error.contains(field),
            "criterion {criterion}: unexpected error: {error}"
        );
    }
}

#[test]
fn run_plan_rejects_non_finite_dt() {
    for dt in ["nan", "inf"] {
        let source = format!("dt = {dt}\n[[stop]]\nwhen = \"steps\"\nsteps = 1\n");
        assert!(
            RunPlanDto::parse(&source).is_err(),
            "dt = {dt} must be rejected"
        );
    }
}

#[test]
fn expect_stop_requires_its_payload() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 5\n[expect.stop]\nwhen = \"event\"\n",
    )
    .unwrap_err();

    assert!(error.contains("event"), "unexpected error: {error}");
}

#[test]
fn final_state_rejects_zero_player_number() {
    let error = ExpectedFinalStateDto::parse(
        "[[players]]\nteam = \"A\"\nnumber = 0\nposition = { x = 0.0, y = 0.0, z = 0.0 }\n",
    )
    .unwrap_err();

    assert!(
        error.contains("number must be positive"),
        "unexpected error: {error}"
    );
}

#[test]
fn final_state_rejects_zero_ball_owner_number() {
    let error = ExpectedFinalStateDto::parse(
        "stage = \"Play\"\n[ball]\npossessed_by = { team = \"A\", number = 0 }\n",
    )
    .unwrap_err();

    assert!(
        error.contains("number must be positive"),
        "unexpected error: {error}"
    );
}

#[test]
fn journal_football_event_requires_event() {
    let error = ScenarioDto::parse(
        "[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 5\n[[expect.journal]]\ntype = \"football_event\"\nteam = \"A\"\n",
    )
    .unwrap_err();

    assert!(error.contains("event"), "unexpected error: {error}");
}

#[test]
fn parses_remaining_stage_and_event_variants() {
    let state = parse_initial("stage = \"GameOver\"\n");
    assert_eq!(state.stage, Some(StageNameDto::GameOver));

    let scenario = parse_scenario(
        r#"
[run]
dt = 0.1

[[run.stop]]
when = "event"
event = "GoalLine"
team = "A"

[[run.stop]]
when = "steps"
steps = 5

[[expect.journal]]
type = "football_event"
event = "GameEnd"
"#,
    );

    assert_eq!(
        scenario.run.stop[0],
        StopCriterionDto::Event {
            event: EventKindDto::GoalLine,
            team: Some(Team::A),
        }
    );
    assert_eq!(
        scenario.expect.journal[0],
        ExpectedEventDto::FootballEvent {
            event: EventKindDto::GameEnd,
            team: None,
            at: None,
        }
    );
}
