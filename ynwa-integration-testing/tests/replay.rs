//! A real scenario's recording, read back from disk, must replay in lockstep with the original run.
//!
//! The check runs step by step (as the football lockstep tests do), so an event applied one tick
//! late or a divergence that self-corrects by the end is caught.

use std::path::{Path, PathBuf};

use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

use ynwa_core::field::zones::{Point3D, Velocity3D};
use ynwa_core::game::GameState;
use ynwa_core::{json_journal_file_reader, RecordReader};
use ynwa_football::create_football_replay_world;
use ynwa_integration_testing::{
    load_scenario, recording_path, run_scenario, LoadedScenario, TOLERANCE,
};

const SCENARIO: &str = "stdlib_kick_to_goal";

fn manifest_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn scenario_dir() -> PathBuf {
    manifest_dir().join("scenarios").join(SCENARIO)
}

fn recording_path_of_scenario() -> PathBuf {
    recording_path(SCENARIO).expect("home directory must be available")
}

#[test]
fn recorded_scenario_replays_equivalently() {
    let report = run_scenario(&scenario_dir()).expect("scenario must run");
    assert!(
        report.passed,
        "scenario failed:\n{}",
        report.diff.unwrap_or_default()
    );

    let record = json_journal_file_reader(recording_path_of_scenario())
        .expect("the scenario must leave a recording")
        .read()
        .expect("the recording must parse");
    let dt = record.header.fixed_dt;
    let total_steps = record.total_steps;
    assert!(total_steps > 0, "the scenario must run at least one step");

    let LoadedScenario {
        mut world,
        run: plan,
        ..
    } = load_scenario(&scenario_dir()).expect("scenario must load");
    assert_eq!(plan.dt, dt);
    assert_eq!(
        world.game().state(),
        &record.header.initial_state,
        "the recording must capture the scenario's start"
    );

    let mut replay = create_football_replay_world(record).expect("replay world must build");

    for step in 0..total_steps {
        assert_equivalent(world.game().state(), replay.game().state(), step);
        world.step(dt);
        replay.step(dt);
    }
    assert_equivalent(world.game().state(), replay.game().state(), total_steps);
}

/// The replay contract: every physically significant field must match within [`TOLERANCE`].
fn assert_equivalent(expected: &GameState, actual: &GameState, step: u64) {
    assert!(
        close(expected.elapsed_time, actual.elapsed_time),
        "step {step}: elapsed_time: {} != {}",
        expected.elapsed_time,
        actual.elapsed_time
    );
    assert_eq!(expected.stage, actual.stage, "step {step}: stage");
    assert_eq!(
        expected.player_states.len(),
        actual.player_states.len(),
        "step {step}: player count"
    );
    for (index, (expected_player, actual_player)) in expected
        .player_states
        .iter()
        .zip(&actual.player_states)
        .enumerate()
    {
        assert!(
            close_point(&expected_player.position, &actual_player.position),
            "step {step}: player {index} position"
        );
        assert!(
            close_velocity(&expected_player.velocity, &actual_player.velocity),
            "step {step}: player {index} velocity"
        );
    }
    assert!(
        close_point(&expected.ball_state.position, &actual.ball_state.position),
        "step {step}: ball position"
    );
    assert!(
        close_velocity(&expected.ball_state.velocity, &actual.ball_state.velocity),
        "step {step}: ball velocity"
    );
    assert_eq!(
        expected.ball_state.possessed_by, actual.ball_state.possessed_by,
        "step {step}: possessed_by"
    );
    assert_eq!(
        expected.ball_state.last_possessing_team, actual.ball_state.last_possessing_team,
        "step {step}: last_possessing_team"
    );
    assert!(
        close(
            expected.ball_state.last_possession_change_time,
            actual.ball_state.last_possession_change_time
        ),
        "step {step}: last_possession_change_time"
    );
    assert!(
        close_optional_point(expected.restart_position, actual.restart_position),
        "step {step}: restart_position"
    );
    assert_eq!(
        expected.restart_team, actual.restart_team,
        "step {step}: restart_team"
    );
    assert_eq!(
        expected.team_stats, actual.team_stats,
        "step {step}: team_stats"
    );
}

fn close(expected: f32, actual: f32) -> bool {
    (expected - actual).abs() <= TOLERANCE
}

fn close_point(expected: &Point3D, actual: &Point3D) -> bool {
    close(expected.x.get::<meter>(), actual.x.get::<meter>())
        && close(expected.y.get::<meter>(), actual.y.get::<meter>())
        && close(expected.z.get::<meter>(), actual.z.get::<meter>())
}

fn close_velocity(expected: &Velocity3D, actual: &Velocity3D) -> bool {
    close(
        expected.x.get::<meter_per_second>(),
        actual.x.get::<meter_per_second>(),
    ) && close(
        expected.y.get::<meter_per_second>(),
        actual.y.get::<meter_per_second>(),
    ) && close(
        expected.z.get::<meter_per_second>(),
        actual.z.get::<meter_per_second>(),
    )
}

fn close_optional_point(expected: Option<Point3D>, actual: Option<Point3D>) -> bool {
    match (expected, actual) {
        (None, None) => true,
        (Some(expected), Some(actual)) => close_point(&expected, &actual),
        _ => false,
    }
}
