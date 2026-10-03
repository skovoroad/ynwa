use crate::scenario::NoneToken;
use crate::*;
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Score, Velocity3D};

fn parse_initial(source: &str) -> InitialState {
    InitialState::parse(source).expect("initial state must parse")
}

fn parse_final(source: &str) -> FinalState {
    FinalState::parse(source).expect("final state must parse")
}

fn parse_scenario(source: &str) -> ScenarioDef {
    ScenarioDef::parse(source).expect("scenario must parse")
}

#[test]
fn minimal_initial_state_is_empty() {
    assert_eq!(parse_initial(""), InitialState::default());
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

    assert_eq!(state.stage, Some(StageName::Play));
    let ball = state.ball.expect("ball present");
    assert_eq!(ball.position, Some(Point3D::from_meters(34.0, 0.0, 10.0)));
    assert_eq!(
        ball.velocity,
        Some(Velocity3D::from_meters_per_second(1.0, 0.0, 2.0))
    );
    assert_eq!(
        ball.possessed_by,
        Some(BallOwner::Player(PlayerRef {
            team: Team::A,
            number: 9
        }))
    );
    assert_eq!(ball.last_possessing_team, Some(TeamOrNone::Team(Team::B)));
    assert_eq!(
        state.players,
        vec![PlayerPlacement {
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

    assert_eq!(state.stage, Some(StageName::Setup));
    assert_eq!(state.setup_reason.as_deref(), Some("throw in"));
    let setup = state.setup.expect("setup present");
    assert_eq!(
        setup.restart_position,
        Some(Point3D::from_meters(0.0, 0.0, 5.5))
    );
    assert_eq!(setup.restart_team, Some(TeamOrNone::Team(Team::A)));
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
    assert_eq!(ball.possessed_by, Some(BallOwner::Free(NoneToken)));
    assert_eq!(ball.last_possessing_team, Some(TeamOrNone::None(NoneToken)));
}

#[test]
fn setup_stage_forbids_ball_state() {
    let error = InitialState::parse(
        "stage = \"Setup\"\n[ball]\nposition = { x = 1.0, y = 0.0, z = 1.0 }\n",
    )
    .unwrap_err();

    assert!(error.contains("ball"), "unexpected error: {error}");
}

#[test]
fn omitted_stage_resolves_to_setup_and_forbids_ball() {
    let error =
        InitialState::parse("[ball]\nposition = { x = 1.0, y = 0.0, z = 1.0 }\n").unwrap_err();

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
        let error = InitialState::parse(source).unwrap_err();
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

    assert_eq!(state.stage, Some(StageName::Setup));
    assert_eq!(state.setup_reason.as_deref(), Some("throw in"));
    assert_eq!(state.score, Some(Score { a: 1, b: 0 }));
    assert_eq!(
        state.ball.expect("ball present").possessed_by,
        Some(BallOwner::Free(NoneToken))
    );
    assert_eq!(
        state.setup.expect("setup present").restart_team,
        Some(TeamOrNone::Team(Team::B))
    );
}

#[test]
fn invalid_team_label_is_rejected() {
    assert!(
        InitialState::parse("stage = \"Play\"\n[ball]\nlast_possessing_team = \"C\"\n").is_err()
    );
}

#[test]
fn invalid_ball_owner_is_rejected() {
    assert!(InitialState::parse("stage = \"Play\"\n[ball]\npossessed_by = \"bogus\"\n").is_err());
}

#[test]
fn invalid_stage_name_is_rejected() {
    assert!(InitialState::parse("stage = \"Half\"\n").is_err());
}

#[test]
fn run_plan_parses_all_criteria() {
    let plan = RunPlan::parse(
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
    assert_eq!(plan.stop.len(), 4);
    assert_eq!(plan.stop[0].when, StopWhen::Stage);
    assert_eq!(plan.stop[0].stage, Some(StageName::Setup));
    assert_eq!(plan.stop[0].setup_reason.as_deref(), Some("goal kick"));
    assert_eq!(
        plan.stop[1].event_matcher(),
        Some(EventMatcher {
            event: EventKind::Goal,
            team: Some(Team::B),
        })
    );
    assert_eq!(plan.stop[2].when, StopWhen::Steps);
    assert_eq!(plan.stop[2].steps, Some(10000));
    assert_eq!(plan.stop[3].when, StopWhen::Time);
    assert_eq!(plan.stop[3].time, Some(60.0));
}

#[test]
fn run_plan_requires_a_safety_criterion() {
    let error =
        RunPlan::parse("dt = 0.1\n[[stop]]\nwhen = \"stage\"\nstage = \"Play\"\n").unwrap_err();

    assert!(error.contains("steps"), "unexpected error: {error}");
}

#[test]
fn run_plan_requires_positive_dt() {
    for dt in ["0.0", "-0.5"] {
        let source = format!("dt = {dt}\n[[stop]]\nwhen = \"steps\"\nsteps = 1\n");
        assert!(
            RunPlan::parse(&source).is_err(),
            "dt = {dt} must be rejected"
        );
    }
}

#[test]
fn invalid_stop_when_is_rejected() {
    assert!(RunPlan::parse("dt = 0.1\n[[stop]]\nwhen = \"bogus\"\n").is_err());
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
    assert_eq!(scenario.expect.journal_match, JournalMatch::Subsequence);
    let stop = scenario.expect.stop.expect("stop expectation present");
    assert_eq!(stop.when, StopWhen::Event);
    assert_eq!(stop.event, Some(EventKind::Goal));
    assert_eq!(stop.team, Some(Team::B));
    assert_eq!(stop.steps, Some(15));
    assert_eq!(
        scenario.expect.journal,
        vec![
            ExpectedEventDef::StageChange {
                stage: Some(StageName::Setup),
                setup_reason: None,
                at: Some(1.5),
            },
            ExpectedEventDef::FootballEvent {
                event: Some(EventKind::Goal),
                team: Some(Team::B),
                at: None,
            },
        ]
    );
}

#[test]
fn expect_defaults_when_omitted() {
    let scenario = parse_scenario("[run]\ndt = 0.1\n[[run.stop]]\nwhen = \"steps\"\nsteps = 3\n");

    assert_eq!(scenario.expect.journal_match, JournalMatch::Exact);
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

    use ExpectedEventDef::*;
    let journal = &scenario.expect.journal;
    assert_eq!(journal.len(), 8);
    assert_eq!(
        journal[0],
        DecisionAssigned {
            player: Some(PlayerRef {
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
            possessed_by: Some(BallOwner::Player(PlayerRef {
                team: Team::B,
                number: 4
            })),
            last_possessing_team: Some(TeamOrNone::None(NoneToken)),
            at: None,
        }
    );
    assert_eq!(
        journal[2],
        KickOutcome {
            player: Some(PlayerRef {
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
            stage: Some(StageName::Setup),
            setup_reason: Some("throw in".to_string()),
            at: None,
        }
    );
    assert_eq!(
        journal[4],
        RestartSet {
            position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
            team: Some(TeamOrNone::Team(Team::A)),
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
            event: Some(EventKind::Touchline),
            team: Some(Team::A),
            at: None,
        }
    );
}

#[test]
fn event_matcher_is_absent_for_non_event_criteria() {
    let criterion = StopCriterionDef {
        when: StopWhen::Steps,
        stage: None,
        setup_reason: None,
        event: None,
        team: None,
        steps: Some(5),
        time: None,
    };

    assert_eq!(criterion.event_matcher(), None);
}
