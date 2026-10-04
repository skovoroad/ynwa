use super::{compile_criteria, run};
use crate::criterion::{EventMatcher, StageMatcher, StopCriterion, StopReason};
use crate::dto::{EventKindDto, RunPlanDto, StopCriterionDto};
use crate::loader::{load_scenario, LoadedScenario};
use ynwa_core::game::GameStage;
use ynwa_core::team::Team;

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
