//! Unit tests for the replay driver: each journaled event maps onto the same state change the
//! original systems produced, events become due at their own timestamp, and sequences of events
//! (bursts within one tick, order-dependent combinations, tick boundaries) replay consistently.

use super::ReplayDriver;
use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, PlayerDef, RefereeDef,
    ScriptingConfig, REGION_START_POSITION,
};
use crate::journal::{JournalEntry, JournalEvent};
use crate::region::GridCell;
use crate::system::System;
use crate::systems::PhysicsSystem;
use crate::team::Team;
use crate::test_utils::deterministic_rng;
use crate::world::World;
use std::collections::HashMap;
use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

fn game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 16);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(12, 8).unwrap(), GridCell::new(12, 8).unwrap())
        .unwrap();

    let player = PlayerDef::new(
        Team::A,
        1,
        "Runner".to_string(),
        String::new(),
        HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
    .with_speed_rate(100);

    let config = GameConfig {
        field,
        players: vec![player],
        ball: BallDef {
            initial_position: Point3D::from_meters(10.0, 0.0, 20.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play, deterministic_rng())
}

/// Two Team A players and one Team B player, so bursts and indices above zero are covered.
fn three_player_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 16);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(12, 8).unwrap(), GridCell::new(12, 8).unwrap())
        .unwrap();

    let make_player = |team: Team, number: u32| {
        PlayerDef::new(
            team,
            number,
            format!("P{}", number),
            String::new(),
            HashMap::from([(REGION_START_POSITION.to_string(), start_region.clone())]),
        )
        .with_speed_rate(100)
    };

    let config = GameConfig {
        field,
        players: vec![
            make_player(Team::A, 1),
            make_player(Team::A, 2),
            make_player(Team::B, 1),
        ],
        ball: BallDef {
            initial_position: Point3D::from_meters(10.0, 0.0, 20.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play, deterministic_rng())
}

fn entry(timestamp: f32, event: JournalEvent) -> JournalEntry {
    JournalEntry { timestamp, event }
}

fn run_decision(player_index: usize, target: Point3D, reason: Option<&str>) -> JournalEvent {
    JournalEvent::DecisionAssigned {
        player_index,
        decision: Decision::Run(DecisionTarget::Point(target)),
        reason: reason.map(str::to_string),
    }
}

fn speed(velocity: &Velocity3D) -> f32 {
    let vx = velocity.x.get::<meter_per_second>();
    let vy = velocity.y.get::<meter_per_second>();
    let vz = velocity.z.get::<meter_per_second>();
    (vx * vx + vy * vy + vz * vz).sqrt()
}

fn score(game: &Game, team: Team) -> f64 {
    game.state
        .team_stats
        .get(&team)
        .map(|stats| stats.get("score"))
        .unwrap_or(0.0)
}

fn stat(game: &Game, team: Team, key: &str) -> f64 {
    game.state
        .team_stats
        .get(&team)
        .map(|stats| stats.get(key))
        .unwrap_or(0.0)
}

/// Asserts the replay contract fields of two worlds are identical (both replays are deterministic).
fn assert_same_physical_state(expected: &Game, actual: &Game) {
    assert_eq!(
        expected.state.player_states.len(),
        actual.state.player_states.len()
    );
    for (index, (expected_player, actual_player)) in expected
        .state
        .player_states
        .iter()
        .zip(&actual.state.player_states)
        .enumerate()
    {
        assert_eq!(
            expected_player.position, actual_player.position,
            "player {index} position"
        );
        assert_eq!(
            expected_player.velocity, actual_player.velocity,
            "player {index} velocity"
        );
    }
    let expected_ball = &expected.state.ball_state;
    let actual_ball = &actual.state.ball_state;
    assert_eq!(
        expected_ball.position, actual_ball.position,
        "ball position"
    );
    assert_eq!(
        expected_ball.velocity, actual_ball.velocity,
        "ball velocity"
    );
    assert_eq!(
        expected_ball.possessed_by, actual_ball.possessed_by,
        "possessed_by"
    );
    assert_eq!(
        expected_ball.last_possessing_team, actual_ball.last_possessing_team,
        "last_possessing_team"
    );
    assert_eq!(
        expected_ball.last_possession_change_time, actual_ball.last_possession_change_time,
        "last_possession_change_time"
    );
    assert_eq!(expected.state.stage, actual.state.stage, "stage");
    assert_eq!(
        expected.state.restart_position, actual.state.restart_position,
        "restart_position"
    );
    assert_eq!(
        expected.state.restart_team, actual.state.restart_team,
        "restart_team"
    );
    assert_eq!(
        expected.state.team_stats, actual.state.team_stats,
        "team_stats"
    );
}

#[test]
fn run_decision_assigns_velocity_and_flags() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            decision: Decision::Run(DecisionTarget::Point(Point3D::from_meters(0.0, 0.0, 30.0))),
            reason: Some("press".to_string()),
        },
    )]);

    driver.update(&mut game, 0.1);

    let player = &game.state.player_states[0];
    assert!(player.velocity.x.get::<meter_per_second>() < 0.0);
    assert!((speed(&player.velocity) - 10.0).abs() < 1e-3);
    assert!(matches!(player.current_decision, Some(Decision::Run(_))));
    assert_eq!(player.decision_reason.as_deref(), Some("press"));
    assert!(player.decision_processed);
    assert!(!player.needs_decision);
    assert_eq!(player.last_decision_time, 0.1);
    assert!(player.last_error.is_none());
}

#[test]
fn stop_decision_zeroes_velocity() {
    let mut game = game();
    game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(4.0, 0.0, 3.0);
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            decision: Decision::Stop,
            reason: None,
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(game.state.player_states[0].velocity, Velocity3D::default());
}

#[test]
fn kick_decision_leaves_player_velocity_unchanged() {
    let mut game = game();
    let velocity = Velocity3D::from_meters_per_second(4.0, 0.0, 3.0);
    game.state.player_states[0].velocity = velocity;
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            decision: Decision::Kick(Point3D::from_meters(0.0, 0.0, 0.0)),
            reason: None,
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(game.state.player_states[0].velocity, velocity);
}

#[test]
fn possession_change_updates_ball_and_requests_decisions() {
    let mut game = game();
    game.state.player_states[0].needs_decision = false;
    let mut driver = ReplayDriver::new(vec![entry(
        0.2,
        JournalEvent::PossessionChange {
            possessed_by: Some(0),
            last_possessing_team: Some(Team::A),
        },
    )]);

    driver.update(&mut game, 0.2);

    assert_eq!(game.state.ball_state.possessed_by, Some(0));
    assert_eq!(game.state.ball_state.last_possessing_team, Some(Team::A));
    assert_eq!(game.state.ball_state.last_possession_change_time, 0.2);
    assert!(game.state.player_states[0].needs_decision);
}

#[test]
fn kick_outcome_sets_ball_velocity_and_releases_possession() {
    let mut game = game();
    game.state.ball_state.possessed_by = Some(0);
    let ball_velocity = Velocity3D::from_meters_per_second(-20.0, 0.0, 0.0);
    let mut driver = ReplayDriver::new(vec![entry(
        0.3,
        JournalEvent::KickOutcome {
            player_index: 0,
            ball_velocity,
        },
    )]);

    driver.update(&mut game, 0.3);

    assert_eq!(game.state.ball_state.velocity, ball_velocity);
    assert_eq!(game.state.ball_state.possessed_by, None);
    assert_eq!(game.state.ball_state.last_possession_change_time, 0.3);
}

#[test]
fn stage_change_updates_stage() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::StageChange {
            stage: GameStage::Setup("throw in".to_string()),
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(game.state.stage, GameStage::Setup("throw in".to_string()));
}

#[test]
fn restart_set_updates_restart_fields() {
    let mut game = game();
    let position = Point3D::from_meters(0.0, 0.0, 0.0);
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::RestartSet {
            restart_position: Some(position),
            restart_team: Some(Team::B),
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(game.state.restart_position, Some(position));
    assert_eq!(game.state.restart_team, Some(Team::B));
}

#[test]
fn decisions_reset_clears_players() {
    let mut game = game();
    game.state.player_states[0].current_decision = Some(Decision::Stop);
    game.state.player_states[0].is_ready = true;
    game.state.player_states[0].needs_decision = false;
    let mut driver = ReplayDriver::new(vec![entry(0.1, JournalEvent::DecisionsReset)]);

    driver.update(&mut game, 0.1);

    let player = &game.state.player_states[0];
    assert!(player.current_decision.is_none());
    assert!(!player.is_ready);
    assert!(player.needs_decision);
}

#[test]
fn stat_update_increments_team_stat() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::StatUpdate {
            team: Team::A,
            key: "score".to_string(),
            delta: 1.0,
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(
        game.state.team_stats.get(&Team::A).unwrap().get("score"),
        1.0
    );
}

#[test]
fn external_event_is_ignored() {
    let mut game = game();
    let stage = game.state.stage.clone();
    let mut driver = ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::json!({ "Goal": "A" }),
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(game.state.stage, stage);
    assert!(game.state.ball_state.possessed_by.is_none());
}

#[test]
fn events_apply_once_when_their_timestamp_is_reached() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![entry(
        0.2,
        JournalEvent::StatUpdate {
            team: Team::A,
            key: "score".to_string(),
            delta: 1.0,
        },
    )]);

    driver.update(&mut game, 0.1);
    assert_eq!(
        game.state.team_stats.get(&Team::A).unwrap().get("score"),
        0.0
    );

    driver.update(&mut game, 0.2);
    driver.update(&mut game, 0.3);
    assert_eq!(
        game.state.team_stats.get(&Team::A).unwrap().get("score"),
        1.0
    );
}

#[test]
fn events_at_the_same_timestamp_apply_in_journal_order() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.1,
            JournalEvent::PossessionChange {
                possessed_by: Some(0),
                last_possessing_team: Some(Team::A),
            },
        ),
        entry(
            0.1,
            JournalEvent::KickOutcome {
                player_index: 0,
                ball_velocity: Velocity3D::from_meters_per_second(-5.0, 0.0, 0.0),
            },
        ),
    ]);

    driver.update(&mut game, 0.1);

    // KickOutcome is applied after PossessionChange, so possession ends up released.
    assert_eq!(game.state.ball_state.possessed_by, None);
}

#[test]
fn setup_burst_assigns_decisions_to_every_player() {
    let mut game = three_player_game();
    let target = Point3D::from_meters(0.0, 0.0, 30.0);
    let mut driver = ReplayDriver::new(vec![
        entry(0.1, run_decision(0, target, Some("setup"))),
        entry(0.1, run_decision(1, target, None)),
        entry(
            0.1,
            JournalEvent::DecisionAssigned {
                player_index: 2,
                decision: Decision::Stop,
                reason: None,
            },
        ),
    ]);

    driver.update(&mut game, 0.1);

    for index in [0, 1] {
        let player = &game.state.player_states[index];
        assert!(
            player.velocity.x.get::<meter_per_second>() < 0.0,
            "player {index}"
        );
        assert!(
            (speed(&player.velocity) - 10.0).abs() < 1e-3,
            "player {index}"
        );
        assert!(player.decision_processed, "player {index}");
        assert!(!player.needs_decision, "player {index}");
    }
    assert_eq!(
        game.state.player_states[2].velocity,
        Velocity3D::default(),
        "player 2 stops"
    );
    assert!(matches!(
        game.state.player_states[2].current_decision,
        Some(Decision::Stop)
    ));
}

#[test]
fn goal_tick_burst_applies_score_reset_restart_and_stage() {
    let mut game = three_player_game();
    for player in &mut game.state.player_states {
        player.current_decision = Some(Decision::Stop);
        player.is_ready = true;
        player.needs_decision = false;
    }
    game.state.restart_position = Some(Point3D::from_meters(1.0, 2.0, 3.0));

    let mut driver = ReplayDriver::new(vec![
        entry(
            0.5,
            JournalEvent::External {
                kind: "football_event".to_string(),
                data: serde_json::json!({ "Goal": "A" }),
            },
        ),
        entry(
            0.5,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0,
            },
        ),
        entry(0.5, JournalEvent::DecisionsReset),
        entry(
            0.5,
            JournalEvent::RestartSet {
                restart_position: None,
                restart_team: Some(Team::B),
            },
        ),
        entry(
            0.5,
            JournalEvent::StageChange {
                stage: GameStage::Setup("kick off".to_string()),
            },
        ),
    ]);

    driver.update(&mut game, 0.5);

    assert_eq!(score(&game, Team::A), 1.0);
    for (index, player) in game.state.player_states.iter().enumerate() {
        assert!(
            player.current_decision.is_none(),
            "player {index} decision cleared"
        );
        assert!(!player.is_ready, "player {index} not ready");
        assert!(player.needs_decision, "player {index} needs decision");
    }
    assert_eq!(game.state.restart_position, None);
    assert_eq!(game.state.restart_team, Some(Team::B));
    assert_eq!(game.state.stage, GameStage::Setup("kick off".to_string()));
}

#[test]
fn kick_tick_applies_decision_before_outcome() {
    let mut game = game();
    game.state.ball_state.possessed_by = Some(0);
    let player_velocity = Velocity3D::from_meters_per_second(3.0, 0.0, 2.0);
    game.state.player_states[0].velocity = player_velocity;
    let ball_velocity = Velocity3D::from_meters_per_second(-20.0, 0.0, 0.0);

    let mut driver = ReplayDriver::new(vec![
        entry(
            0.2,
            JournalEvent::DecisionAssigned {
                player_index: 0,
                decision: Decision::Kick(Point3D::from_meters(0.0, 0.0, 30.0)),
                reason: None,
            },
        ),
        entry(
            0.2,
            JournalEvent::KickOutcome {
                player_index: 0,
                ball_velocity,
            },
        ),
    ]);

    driver.update(&mut game, 0.2);

    assert!(matches!(
        game.state.player_states[0].current_decision,
        Some(Decision::Kick(_))
    ));
    assert_eq!(game.state.player_states[0].velocity, player_velocity);
    assert_eq!(game.state.ball_state.velocity, ball_velocity);
    assert_eq!(game.state.ball_state.possessed_by, None);
    assert_eq!(game.state.ball_state.last_possession_change_time, 0.2);
}

#[test]
fn empty_journal_leaves_state_untouched() {
    let mut game = game();
    let mut driver = ReplayDriver::new(Vec::new());

    driver.update(&mut game, 0.1);
    driver.update(&mut game, 5.0);

    assert_eq!(game.state.stage, GameStage::Play);
    assert_eq!(game.state.player_states[0].velocity, Velocity3D::default());
    assert!(game.state.player_states[0].current_decision.is_none());
    assert_eq!(game.state.ball_state.possessed_by, None);
    assert_eq!(score(&game, Team::A), 0.0);
    assert_eq!(game.state.elapsed_time, 0.0);
}

#[test]
fn events_apply_at_the_first_tick_that_reaches_them() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.25,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "early".to_string(),
                delta: 1.0,
            },
        ),
        entry(
            10.0,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "late".to_string(),
                delta: 100.0,
            },
        ),
    ]);

    driver.update(&mut game, 0.1);
    driver.update(&mut game, 0.2);
    assert_eq!(stat(&game, Team::A, "early"), 0.0, "not due yet");

    driver.update(&mut game, 0.3);
    assert_eq!(stat(&game, Team::A, "early"), 1.0, "due event applied once");
    assert_eq!(
        stat(&game, Team::A, "late"),
        0.0,
        "event after the last tick stays pending"
    );

    driver.update(&mut game, 0.9);
    assert_eq!(stat(&game, Team::A, "early"), 1.0);
    assert_eq!(stat(&game, Team::A, "late"), 0.0);
}

#[test]
fn event_at_time_zero_applies_on_the_first_tick() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![entry(
        0.0,
        JournalEvent::StatUpdate {
            team: Team::A,
            key: "score".to_string(),
            delta: 1.0,
        },
    )]);

    driver.update(&mut game, 0.1);

    assert_eq!(score(&game, Team::A), 1.0);
}

#[test]
fn later_possession_change_overwrites_earlier_one() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.1,
            JournalEvent::PossessionChange {
                possessed_by: Some(0),
                last_possessing_team: Some(Team::A),
            },
        ),
        entry(
            0.2,
            JournalEvent::PossessionChange {
                possessed_by: None,
                last_possessing_team: Some(Team::A),
            },
        ),
    ]);

    driver.update(&mut game, 0.1);
    driver.update(&mut game, 0.2);

    assert_eq!(game.state.ball_state.possessed_by, None);
    assert_eq!(game.state.ball_state.last_possessing_team, Some(Team::A));
    assert_eq!(game.state.ball_state.last_possession_change_time, 0.2);
}

#[test]
fn possession_change_after_kick_restores_ownership() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.1,
            JournalEvent::KickOutcome {
                player_index: 0,
                ball_velocity: Velocity3D::from_meters_per_second(-5.0, 0.0, 0.0),
            },
        ),
        entry(
            0.2,
            JournalEvent::PossessionChange {
                possessed_by: Some(0),
                last_possessing_team: Some(Team::A),
            },
        ),
    ]);

    driver.update(&mut game, 0.1);
    assert_eq!(game.state.ball_state.possessed_by, None);

    driver.update(&mut game, 0.2);
    assert_eq!(game.state.ball_state.possessed_by, Some(0));
    assert_eq!(game.state.ball_state.last_possessing_team, Some(Team::A));
    assert_eq!(game.state.ball_state.last_possession_change_time, 0.2);
}

#[test]
fn assignment_after_reset_in_the_same_tick_survives() {
    let mut game = game();
    game.state.player_states[0].current_decision = Some(Decision::Stop);
    let mut driver = ReplayDriver::new(vec![
        entry(0.1, JournalEvent::DecisionsReset),
        entry(
            0.1,
            run_decision(0, Point3D::from_meters(0.0, 0.0, 30.0), Some("setup")),
        ),
    ]);

    driver.update(&mut game, 0.1);

    assert!(matches!(
        game.state.player_states[0].current_decision,
        Some(Decision::Run(_))
    ));
    assert_eq!(game.state.player_states[0].last_decision_time, 0.1);
}

#[test]
fn reset_after_assignment_in_a_later_tick_clears_it() {
    let mut game = game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.1,
            run_decision(0, Point3D::from_meters(0.0, 0.0, 30.0), None),
        ),
        entry(0.2, JournalEvent::DecisionsReset),
    ]);

    driver.update(&mut game, 0.1);
    assert!(game.state.player_states[0].current_decision.is_some());

    driver.update(&mut game, 0.2);
    let player = &game.state.player_states[0];
    assert!(player.current_decision.is_none());
    assert!(!player.is_ready);
    assert!(player.needs_decision);
}

#[test]
fn stat_updates_accumulate_per_team_and_key() {
    let mut game = three_player_game();
    let mut driver = ReplayDriver::new(vec![
        entry(
            0.1,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0,
            },
        ),
        entry(
            0.2,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0,
            },
        ),
        entry(
            0.3,
            JournalEvent::StatUpdate {
                team: Team::B,
                key: "shots".to_string(),
                delta: 2.0,
            },
        ),
        entry(
            0.4,
            JournalEvent::StatUpdate {
                team: Team::B,
                key: "shots".to_string(),
                delta: 3.0,
            },
        ),
    ]);

    driver.update(&mut game, 0.1);
    driver.update(&mut game, 0.2);
    driver.update(&mut game, 0.3);
    driver.update(&mut game, 0.4);

    assert_eq!(score(&game, Team::A), 2.0);
    assert_eq!(stat(&game, Team::A, "shots"), 0.0);
    assert_eq!(stat(&game, Team::B, "shots"), 5.0);
    assert_eq!(score(&game, Team::B), 0.0);
}

#[test]
fn applied_event_affects_physics_of_the_same_tick() {
    let game = game();
    let mut world = World::new(game);
    world.add_system(Box::new(ReplayDriver::new(vec![entry(
        0.1,
        JournalEvent::KickOutcome {
            player_index: 0,
            ball_velocity: Velocity3D::from_meters_per_second(1.0, 0.0, 0.0),
        },
    )])));
    world.add_system(Box::new(PhysicsSystem::new()));

    world.step(0.1);

    // Ball starts at x = 10; friction turns 1 m/s into 0.8 m/s over the tick, so it moves 0.08 m.
    let x = world.game().state.ball_state.position.x.get::<meter>();
    assert!((x - 10.08).abs() < 1e-4, "ball x after one tick: {x}");
    assert_eq!(
        world.game().state.ball_state.last_possession_change_time,
        0.1
    );
}

/// A mixed sequence spanning several ticks: decisions, possession, kick, score, reset, stage.
fn sequence_journal() -> Vec<JournalEntry> {
    vec![
        entry(
            0.1,
            run_decision(0, Point3D::from_meters(50.0, 0.0, 30.0), Some("run")),
        ),
        entry(
            0.2,
            JournalEvent::PossessionChange {
                possessed_by: Some(0),
                last_possessing_team: Some(Team::A),
            },
        ),
        entry(
            0.3,
            JournalEvent::DecisionAssigned {
                player_index: 0,
                decision: Decision::Kick(Point3D::from_meters(0.0, 0.0, 30.0)),
                reason: None,
            },
        ),
        entry(
            0.3,
            JournalEvent::KickOutcome {
                player_index: 0,
                ball_velocity: Velocity3D::from_meters_per_second(-15.0, 0.0, 2.0),
            },
        ),
        entry(
            0.5,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0,
            },
        ),
        entry(0.6, JournalEvent::DecisionsReset),
        entry(
            0.6,
            JournalEvent::RestartSet {
                restart_position: Some(Point3D::from_meters(0.0, 0.0, 0.0)),
                restart_team: Some(Team::B),
            },
        ),
        entry(
            0.7,
            JournalEvent::StageChange {
                stage: GameStage::Setup("throw in".to_string()),
            },
        ),
    ]
}

#[test]
fn two_replays_of_the_same_journal_agree() {
    let mut first = three_player_game();
    let mut second = three_player_game();
    let mut first_driver = ReplayDriver::new(sequence_journal());
    let mut second_driver = ReplayDriver::new(sequence_journal());

    for tick in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7] {
        first_driver.update(&mut first, tick);
        second_driver.update(&mut second, tick);
    }

    assert_same_physical_state(&first, &second);
}
