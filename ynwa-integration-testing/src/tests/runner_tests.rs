use super::{compile_criteria, run};
use crate::criterion::{EventMatcher, StageMatcher, StopCriterion, StopReason};
use crate::dto::{EventKindDto, RunPlanDto, StageNameDto, StopCriterionDto};
use crate::loader::{load_scenario, LoadedScenario};
use ynwa_core::game::GameStage;
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Velocity3D};
use ynwa_football::events::FootballEvent;

fn fixture(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn compile_criteria_preserves_order_and_parameters() {
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
steps = 100
"#,
    )
    .unwrap();

    assert_eq!(
        compile_criteria(&plan).unwrap(),
        vec![
            StopCriterion::OnStage(StageMatcher::Setup {
                reason: Some("goal kick".to_string()),
            }),
            StopCriterion::OnEvent(EventMatcher::Goal {
                team: Some(Team::B),
            }),
            StopCriterion::OnSteps(100),
        ]
    );
}

#[test]
fn compile_rejects_team_on_game_end() {
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![StopCriterionDto::Event {
            event: EventKindDto::GameEnd,
            team: Some(Team::A),
        }],
    };

    let error = compile_criteria(&plan).unwrap_err();

    assert!(
        error.message().contains("GameEnd"),
        "unexpected error: {error}"
    );
}

#[test]
fn trivial_scenario_stops_at_the_step_limit() {
    let loaded = load_scenario(&fixture("valid")).expect("fixture must load");
    let LoadedScenario {
        world, run: plan, ..
    } = loaded;

    let outcome = run(world, &plan).expect("trivial run must succeed");

    assert_eq!(outcome.steps, 10);
    assert_eq!(outcome.stop_reason, StopReason::Steps(10));
    assert_eq!(outcome.final_state.stage, GameStage::Play);
}

#[test]
fn compile_criteria_covers_time() {
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![
            StopCriterionDto::Time { time: 2.5 },
            StopCriterionDto::Steps { steps: 5 },
        ],
    };

    assert_eq!(
        compile_criteria(&plan).unwrap(),
        vec![StopCriterion::OnTime(2.5), StopCriterion::OnSteps(5)]
    );
}

#[test]
fn stops_on_stage() {
    let LoadedScenario { world, .. } = load_scenario(&fixture("valid")).expect("fixture must load");
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![
            StopCriterionDto::Stage {
                stage: StageNameDto::Play,
                setup_reason: None,
            },
            StopCriterionDto::Steps { steps: 100 },
        ],
    };

    let outcome = run(world, &plan).expect("run must succeed");

    assert_eq!(outcome.stop_reason, StopReason::Stage(StageMatcher::Play));
    assert_eq!(outcome.steps, 1);
}

#[test]
fn stops_on_time() {
    let LoadedScenario { world, .. } = load_scenario(&fixture("valid")).expect("fixture must load");
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![
            StopCriterionDto::Time { time: 0.5 },
            StopCriterionDto::Steps { steps: 100 },
        ],
    };

    let outcome = run(world, &plan).expect("run must succeed");

    assert_eq!(outcome.stop_reason, StopReason::Time(0.5));
    assert_eq!(outcome.steps, 5);
}

#[test]
fn stops_on_football_event() {
    let LoadedScenario { mut world, .. } =
        load_scenario(&fixture("valid")).expect("fixture must load");
    {
        let ball = &mut world.game_mut().state.ball_state;
        ball.position = Point3D::from_meters(-0.5, 0.0, 52.0);
        ball.velocity = Velocity3D::from_meters_per_second(0.0, 0.0, 0.0);
        ball.possessed_by = None;
        ball.last_possessing_team = Some(Team::A);
    }
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![
            StopCriterionDto::Event {
                event: EventKindDto::Touchline,
                team: None,
            },
            StopCriterionDto::Steps { steps: 100 },
        ],
    };

    let outcome = run(world, &plan).expect("run must succeed");

    assert_eq!(
        outcome.stop_reason,
        StopReason::Event(EventMatcher::Touchline { team: None })
    );
    assert_eq!(outcome.steps, 1);
    assert_eq!(outcome.football_events.len(), 1);
    assert!(matches!(
        outcome.football_events[0].1,
        FootballEvent::Touchline(_, _)
    ));
}

#[test]
fn player_error_fails_the_run() {
    let LoadedScenario { mut world, .. } =
        load_scenario(&fixture("valid")).expect("fixture must load");
    world.game_mut().state.stage = GameStage::Setup("bogus".to_string());
    let plan = RunPlanDto {
        dt: 0.1,
        stop: vec![StopCriterionDto::Steps { steps: 1 }],
    };

    let error = run(world, &plan).unwrap_err();

    assert!(
        error.message().contains("unknown setup reason"),
        "unexpected error: {error}"
    );
}
