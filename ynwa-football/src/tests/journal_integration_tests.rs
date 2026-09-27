//! End-to-end journal recording: a short game with `CollectJournalRecorder`, and
//! targeted checks that `FootballGameManager` records its stage/event effects.

use crate::events::{check_events, FootballEvent};
use crate::game_manager::FootballGameManager;
use crate::test_utils::{
    attach_journal, build_recording_world, deterministic_rng, single_player_config, FIXED_DT, STEPS,
};
use ynwa_core::field::zones::Point3D;
use ynwa_core::game::{Decision, Game, GameStage};
use ynwa_core::journal::JournalEvent;
use ynwa_core::record::RecordHeader;
use ynwa_core::system::System;
use ynwa_core::team::Team;

fn play_game() -> Game {
    Game::with_stage(single_player_config(), GameStage::Play, deterministic_rng())
}

#[test]
fn short_game_records_expected_events_and_builds_record() {
    let (mut world, collection) = build_recording_world();

    for _ in 0..STEPS {
        world.step(FIXED_DT);
    }
    world.game_mut().finish_journal().unwrap();

    let entries = collection.borrow().entries().to_vec();
    let any = |pred: fn(&JournalEvent) -> bool| entries.iter().any(|e| pred(&e.event));

    assert!(
        any(|e| matches!(e, JournalEvent::DecisionAssigned { .. })),
        "expected assigned decisions"
    );
    assert!(
        entries.iter().any(|e| matches!(
            &e.event,
            JournalEvent::DecisionAssigned {
                reason: Some(_),
                ..
            }
        )),
        "expected a decision carrying a reason"
    );
    assert!(
        any(|e| matches!(
            e,
            JournalEvent::StageChange {
                stage: GameStage::Play
            }
        )),
        "expected Setup -> Play transition"
    );
    assert!(
        any(|e| matches!(
            e,
            JournalEvent::StageChange {
                stage: GameStage::Setup(reason)
            } if reason == "throw in"
        )),
        "expected Play -> Setup(throw in) transition"
    );
    assert!(
        any(|e| matches!(
            e,
            JournalEvent::PossessionChange {
                possessed_by: Some(0),
                ..
            }
        )),
        "expected possession change"
    );
    assert!(
        any(|e| matches!(
            e,
            JournalEvent::KickOutcome {
                player_index: 0,
                ..
            }
        )),
        "expected recorded kick outcome"
    );
    assert!(
        any(|e| matches!(e, JournalEvent::External { kind, .. } if kind == "football_event")),
        "expected a football external event"
    );
    assert_eq!(collection.borrow().total_steps(), STEPS);

    let header = RecordHeader {
        config: single_player_config(),
        initial_stage: GameStage::Setup("kick off".to_string()),
        fixed_dt: FIXED_DT,
    };
    let record = collection.borrow_mut().take_record(header);

    assert_eq!(record.total_steps, STEPS);
    assert_eq!(record.journal.len(), entries.len());
    assert_eq!(record.header.fixed_dt, FIXED_DT);
    assert!(record
        .journal
        .iter()
        .any(|e| matches!(e.event, JournalEvent::KickOutcome { .. })));
    assert!(collection.borrow().entries().is_empty());
}

#[test]
fn game_end_is_recorded_as_external_and_stage_change() {
    let mut game = play_game();
    game.state.elapsed_time = 120.0;
    let collection = attach_journal(&mut game);

    FootballGameManager::new().update(&mut game, 120.0);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[0].event,
        JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::to_value(FootballEvent::GameEnd).unwrap(),
        }
    );
    assert_eq!(
        entries[1].event,
        JournalEvent::StageChange {
            stage: GameStage::GameOver
        }
    );
}

#[test]
fn touchline_records_reset_restart_and_stage() {
    let ball_out = Point3D::from_meters(-1.0, 0.0, 30.0);
    let mut game = play_game();
    game.state.ball_state.position = ball_out;
    game.state.ball_state.last_possessing_team = Some(Team::A);
    let collection = attach_journal(&mut game);

    FootballGameManager::new().update(&mut game, 5.0);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 4);
    assert_eq!(
        entries[0].event,
        JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::to_value(FootballEvent::Touchline(ball_out, Team::A)).unwrap(),
        }
    );
    assert_eq!(entries[1].event, JournalEvent::DecisionsReset);
    assert_eq!(
        entries[2].event,
        JournalEvent::RestartSet {
            restart_position: Some(ball_out),
            restart_team: Some(Team::B),
        }
    );
    assert_eq!(
        entries[3].event,
        JournalEvent::StageChange {
            stage: GameStage::Setup("throw in".to_string())
        }
    );
}

#[test]
fn goal_line_records_goal_kick_restart() {
    let ball_out = Point3D::from_meters(50.0, 0.0, -1.0);
    let mut game = play_game();
    game.state.ball_state.position = ball_out;
    game.state.ball_state.last_possessing_team = Some(Team::B); // attacker B -> defending A takes the kick
    let collection = attach_journal(&mut game);

    FootballGameManager::new().update(&mut game, 5.0);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 4);
    assert_eq!(entries[1].event, JournalEvent::DecisionsReset);
    assert_eq!(
        entries[2].event,
        JournalEvent::RestartSet {
            restart_position: Some(Point3D::from_meters(50.0, 0.0, 5.5)),
            restart_team: Some(Team::A),
        }
    );
    assert_eq!(
        entries[3].event,
        JournalEvent::StageChange {
            stage: GameStage::Setup("goal kick".to_string())
        }
    );
}

#[test]
fn goal_line_records_corner_restart() {
    let ball_out = Point3D::from_meters(50.0, 0.0, -1.0);
    let mut game = play_game();
    game.state.ball_state.position = ball_out;
    game.state.ball_state.last_possessing_team = Some(Team::A); // defender A -> corner for attacker B
    let collection = attach_journal(&mut game);

    FootballGameManager::new().update(&mut game, 5.0);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(
        entries[2].event,
        JournalEvent::RestartSet {
            restart_position: Some(Point3D::from_meters(0.0, 0.0, 0.0)),
            restart_team: Some(Team::B),
        }
    );
    assert_eq!(
        entries[3].event,
        JournalEvent::StageChange {
            stage: GameStage::Setup("corner".to_string())
        }
    );
}

#[test]
fn setup_decision_and_play_transition_are_recorded() {
    let mut game = Game::with_stage(
        single_player_config(),
        GameStage::Setup("kick off".to_string()),
        deterministic_rng(),
    );
    let collection = attach_journal(&mut game);

    let mut manager = FootballGameManager::new();
    manager.update(&mut game, 0.1);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 1);
    assert!(matches!(
        entries[0].event,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            ..
        }
    ));

    game.state.player_states[0].current_decision = Some(Decision::Stop);
    manager.update(&mut game, 0.2);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 2);
    assert_eq!(
        entries[1].event,
        JournalEvent::StageChange {
            stage: GameStage::Play
        }
    );
}

/// `check_events` must agree with the recorded external event.
#[test]
fn detected_football_event_matches_recorded_external() {
    let ball_out = Point3D::from_meters(-1.0, 0.0, 30.0);
    let mut game = play_game();
    game.state.ball_state.position = ball_out;
    game.state.ball_state.last_possessing_team = Some(Team::A);

    let expected = check_events(&game).expect("touchline must be detected");
    let collection = attach_journal(&mut game);
    FootballGameManager::new().update(&mut game, 5.0);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(
        entries[0].event,
        JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::to_value(expected).unwrap(),
        }
    );
}
