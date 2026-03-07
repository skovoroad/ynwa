use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, GameStage, PlayerDef, RefereeDef};
use crate::region::GridCell;
use crate::team::Team;

fn create_test_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap()).unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::new(config)
}

#[test]
fn test_decision_system_clears_needs_decision() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new();

    game.state.player_states[0].needs_decision = true;
    game.state.player_states[0].last_decision_time = 0.0;

    system.update(&mut game, 1.0);

    assert!(!game.state.player_states[0].needs_decision);
    assert_eq!(game.state.player_states[0].last_decision_time, 1.0);
}

#[test]
fn test_decision_system_creates_decision() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new();

    game.state.player_states[0].needs_decision = true;
    game.state.player_states[0].current_decision = None;

    system.update(&mut game, 1.0);

    assert!(game.state.player_states[0].current_decision.is_some());
    assert!(!game.state.player_states[0].decision_processed);
}

#[test]
fn test_decision_system_creates_run_decision() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new();

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    match &game.state.player_states[0].current_decision {
        Some(Decision::Run(DecisionTarget::GridCell(cell))) => {
            let grid_dims = game.config().field.grid_dimensions();
            assert!(cell.col >= 1 && cell.col <= grid_dims.columns);
            assert!(cell.row >= 1 && cell.row <= grid_dims.rows);
        }
        _ => panic!("Expected Run decision with GridCell target"),
    }
}

#[test]
fn test_decision_system_preserves_previous_decision() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new();

    game.state.player_states[0].needs_decision = true;
    system.update(&mut game, 1.0);

    let first_decision = game.state.player_states[0].current_decision.clone();
    assert!(first_decision.is_some());

    game.state.player_states[0].decision_processed = true;
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 2.0);

    assert!(matches!(
        game.state.player_states[0].current_decision,
        Some(_)
    ));
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_placeholder_decision_maker() {
    let game = create_test_game();
    let mut maker = PlaceholderDecisionMaker::new();

    let (decision, reason) = maker.make_decision(&game, 0).expect("Should not error");

    match decision {
        Decision::Run(DecisionTarget::GridCell(cell)) => {
            let grid_dims = game.config().field.grid_dimensions();
            assert!(cell.col >= 1 && cell.col <= grid_dims.columns);
            assert!(cell.row >= 1 && cell.row <= grid_dims.rows);
        }
        _ => panic!("Expected Run decision with GridCell target"),
    }
    assert_eq!(reason, None); // Placeholder doesn't provide reasons
}

// Test error handling
struct ErrorDecisionMaker;

impl DecisionMaker for ErrorDecisionMaker {
    fn make_decision(
        &mut self,
        _game: &Game,
        _player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        Err(DecisionError::ScriptError("Test error".to_string()))
    }
}

#[test]
fn test_decision_system_handles_error_with_default_handler() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Default handler returns None - no decision
    assert!(game.state.player_states[0].current_decision.is_none());
    // needs_decision should be cleared (rate-limited, no immediate retry)
    assert!(!game.state.player_states[0].needs_decision);
    // timestamp should be updated
    assert_eq!(game.state.player_states[0].last_decision_time, 1.0);
}

#[test]
fn test_decision_system_with_custom_error_handler() {
    let mut game = create_test_game();

    // Custom handler that returns a specific cell on error
    let mut system = DecisionSystem::new()
        .with_decision_maker(Box::new(ErrorDecisionMaker))
        .with_error_handler(|_error, _idx| {
            Some(Decision::Run(DecisionTarget::GridCell(
                GridCell::new(13, 22).unwrap(),
            )))
        });

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Should use custom error handler
    match &game.state.player_states[0].current_decision {
        Some(Decision::Run(DecisionTarget::GridCell(cell))) => {
            assert_eq!(cell.col, 13);
            assert_eq!(cell.row, 22);
        }
        other => panic!("Expected custom Run decision, got {:?}", other),
    }
    // needs_decision should be cleared since handler provided a decision
    assert!(!game.state.player_states[0].needs_decision);
}

#[test]
fn test_decision_system_updates_timestamp_on_error() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));

    game.state.player_states[0].needs_decision = true;
    game.state.player_states[0].last_decision_time = 5.0;

    system.update(&mut game, 10.0);

    // timestamp SHOULD be updated even when handler returns None (rate-limited approach)
    assert_eq!(game.state.player_states[0].last_decision_time, 10.0);
    assert!(!game.state.player_states[0].needs_decision); // Cleared
    assert!(game.state.player_states[0].current_decision.is_none());
}

#[test]
fn test_decision_system_rate_limits_errors() {
    use std::sync::{Arc, Mutex};

    struct FlakeyDecisionMaker {
        attempts: Arc<Mutex<i32>>,
    }

    impl DecisionMaker for FlakeyDecisionMaker {
        fn make_decision(
            &mut self,
            _game: &Game,
            _idx: usize,
        ) -> Result<(Decision, Option<String>), DecisionError> {
            let mut count = self.attempts.lock().unwrap();
            *count += 1;

            if *count == 1 {
                Err(DecisionError::ScriptError(
                    "First attempt fails".to_string(),
                ))
            } else {
                Ok((
                    Decision::Run(DecisionTarget::GridCell(GridCell::new(13, 22).unwrap())),
                    None,
                ))
            }
        }
    }

    let attempts = Arc::new(Mutex::new(0));
    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(FlakeyDecisionMaker {
        attempts: attempts.clone(),
    }));

    game.state.player_states[0].needs_decision = true;

    // First update: error, should be rate-limited (no immediate retry)
    system.update(&mut game, 1.0);
    assert!(!game.state.player_states[0].needs_decision); // Cleared
    assert!(game.state.player_states[0].current_decision.is_none());
    assert_eq!(*attempts.lock().unwrap(), 1);

    // Second update: nothing happens (needs_decision=false)
    system.update(&mut game, 2.0);
    assert!(!game.state.player_states[0].needs_decision);
    assert_eq!(*attempts.lock().unwrap(), 1); // No second call!

    // Player must be marked as needs_decision by PlayerReactionSystem
    // to try again based on reaction_rate
}

#[test]
fn test_decision_error_timeout_variant() {
    struct TimeoutDecisionMaker;

    impl DecisionMaker for TimeoutDecisionMaker {
        fn make_decision(
            &mut self,
            _game: &Game,
            _idx: usize,
        ) -> Result<(Decision, Option<String>), DecisionError> {
            Err(DecisionError::Timeout(
                "Execution took too long".to_string(),
            ))
        }
    }

    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(TimeoutDecisionMaker));

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Should handle Timeout error with rate-limiting
    assert!(game.state.player_states[0].current_decision.is_none());
    assert!(!game.state.player_states[0].needs_decision); // Cleared, rate-limited
    assert_eq!(game.state.player_states[0].last_decision_time, 1.0);
}

#[test]
fn test_decision_error_runtime_variant() {
    struct RuntimeErrorDecisionMaker;

    impl DecisionMaker for RuntimeErrorDecisionMaker {
        fn make_decision(
            &mut self,
            _game: &Game,
            _idx: usize,
        ) -> Result<(Decision, Option<String>), DecisionError> {
            Err(DecisionError::RuntimeError("Internal error".to_string()))
        }
    }

    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(RuntimeErrorDecisionMaker));

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Should handle RuntimeError with rate-limiting
    assert!(game.state.player_states[0].current_decision.is_none());
    assert!(!game.state.player_states[0].needs_decision); // Cleared, rate-limited
    assert_eq!(game.state.player_states[0].last_decision_time, 1.0);
}

#[test]
fn test_error_handler_receives_correct_player_index() {
    // Test that error handler is called with correct player_index
    // We can't capture in fn pointer, so we test indirectly through decisions

    let mut game = create_test_game();

    // Handler that returns different decisions based on player_index
    fn indexed_handler(_error: &DecisionError, idx: usize) -> Option<Decision> {
        // Return decision with row = player_index + 1 for verification
        Some(Decision::Run(DecisionTarget::GridCell(
            GridCell::new(13, (idx as u32) + 1).unwrap(),
        )))
    }

    let mut system = DecisionSystem::new()
        .with_decision_maker(Box::new(ErrorDecisionMaker))
        .with_error_handler(indexed_handler);

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Check that handler was called with player_index=0
    match &game.state.player_states[0].current_decision {
        Some(Decision::Run(DecisionTarget::GridCell(cell))) => {
            assert_eq!(cell.row, 1); // idx=0 → row=1
        }
        other => panic!("Expected Run decision, got {:?}", other),
    }
}

#[test]
fn test_custom_handler_returning_none_explicitly() {
    let mut game = create_test_game();

    // Custom handler that explicitly returns None (same as default)
    let mut system = DecisionSystem::new()
        .with_decision_maker(Box::new(ErrorDecisionMaker))
        .with_error_handler(|_error, _idx| None);

    game.state.player_states[0].needs_decision = true;
    game.state.player_states[0].last_decision_time = 3.0;

    system.update(&mut game, 5.0);

    // Should behave like default handler with rate-limiting
    assert!(game.state.player_states[0].current_decision.is_none());
    assert!(!game.state.player_states[0].needs_decision); // Cleared
    assert_eq!(game.state.player_states[0].last_decision_time, 5.0); // Updated
}

#[test]
fn test_error_message_saved_to_player_state() {
    let mut game = create_test_game();
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));

    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 1.0);

    // Error message should be saved
    assert!(game.state.player_states[0].last_error.is_some());
    let error_msg = game.state.player_states[0].last_error.as_ref().unwrap();
    assert!(error_msg.contains("Test error"));
}

#[test]
fn test_error_cleared_on_successful_decision() {
    let mut game = create_test_game();

    // First: cause an error
    let mut system = DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));

    game.state.player_states[0].needs_decision = true;
    system.update(&mut game, 1.0);

    assert!(game.state.player_states[0].last_error.is_some());


    // Now: switch to working decision maker
    let mut system = DecisionSystem::new();
    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 2.0);

    // Error should be cleared
    assert!(game.state.player_states[0].last_error.is_none());
    assert!(game.state.player_states[0].current_decision.is_some());
}

// ── Setup stage: arrival check ───────────────────────────────────────────────

fn make_setup_game_with_player_at(x: f32, z: f32) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Setup("start".to_string()));
    game.state.player_states[0].position = crate::field::zones::Point3D::from_meters(x, 0.0, z);
    game
}

#[test]
fn test_setup_arrival_check_stops_player_when_close_to_target() {
    // Player is within 0.5m of the Run target → DecisionSystem must override with Stop
    let target_x = 30.0_f32;
    let target_z = 20.0_f32;
    let mut game = make_setup_game_with_player_at(target_x + 0.3, target_z + 0.3);
    let mut system = DecisionSystem::new();

    // Give the player a Run decision pointing to (target_x, target_z)
    let target = crate::field::zones::Point3D::from_meters(target_x, 0.0, target_z);
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::Point(target)));
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 1.0);

    assert!(
        matches!(game.state.player_states[0].current_decision, Some(Decision::Stop)),
        "Expected Stop when player is within arrival threshold"
    );
}

#[test]
fn test_setup_arrival_check_does_not_stop_player_when_far() {
    // Player is more than 0.5m away → decision must remain Run
    let target_x = 30.0_f32;
    let target_z = 20.0_f32;
    let mut game = make_setup_game_with_player_at(target_x + 5.0, target_z);
    let mut system = DecisionSystem::new();

    let target = crate::field::zones::Point3D::from_meters(target_x, 0.0, target_z);
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::Point(target)));
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 1.0);

    assert!(
        matches!(
            game.state.player_states[0].current_decision,
            Some(Decision::Run(_))
        ),
        "Expected Run to be preserved when player is far from target"
    );
}

#[test]
fn test_setup_arrival_check_skipped_when_no_decision() {
    // Player has no current decision → arrival check must not panic or assign Stop
    let mut game = make_setup_game_with_player_at(50.0, 30.0);
    let mut system = DecisionSystem::new();

    game.state.player_states[0].current_decision = None;
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 1.0);

    assert!(
        game.state.player_states[0].current_decision.is_none(),
        "No decision should be created by arrival check alone"
    );
}

#[test]
fn test_setup_arrival_check_skipped_when_decision_is_stop() {
    // Player already has Stop (already arrived) → no change
    let mut game = make_setup_game_with_player_at(30.0, 20.0);
    let mut system = DecisionSystem::new();

    game.state.player_states[0].current_decision = Some(Decision::Stop);
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 1.0);

    assert!(
        matches!(game.state.player_states[0].current_decision, Some(Decision::Stop))
    );
}

#[test]
fn test_play_stage_arrival_check_fires_and_stops_player() {
    // In Play stage the arrival check MUST override a Run decision with Stop
    // when the player reaches their target. This prevents overshooting caused
    // by the reaction-rate gap between script calls.
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Play);
    let target = crate::field::zones::Point3D::from_meters(30.0, 0.0, 20.0);
    // Put player right on top of the target
    game.state.player_states[0].position =
        crate::field::zones::Point3D::from_meters(30.0, 0.0, 20.0);
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::Point(target)));
    game.state.player_states[0].needs_decision = false;

    let mut system = DecisionSystem::new();
    system.update(&mut game, 1.0);

    // Arrival check must have replaced Run with Stop
    assert!(
        matches!(
            game.state.player_states[0].current_decision,
            Some(Decision::Stop)
        ),
        "Arrival check must fire in Play stage and replace Run with Stop"
    );

    // needs_decision must NOT be suppressed in Play — the reaction timer continues
    // so the script will get called at the next reaction interval.
    // (needs_decision was false before, arrival check must not set it to true either)
    assert!(
        !game.state.player_states[0].needs_decision,
        "Arrival check in Play stage must not touch needs_decision"
    );
}

#[test]
fn test_setup_arrival_check_does_not_call_script() {
    // When arrival check fires, the script must not be called (needs_decision was false)
    // We verify this by using an ErrorDecisionMaker: if the script were called it would
    // wipe current_decision; since it must not be called, Stop must remain.
    let target_x = 30.0_f32;
    let target_z = 20.0_f32;
    let mut game = make_setup_game_with_player_at(target_x + 0.1, target_z);

    let target = crate::field::zones::Point3D::from_meters(target_x, 0.0, target_z);
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::Point(target)));
    game.state.player_states[0].needs_decision = false;

    let mut system =
        DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));

    system.update(&mut game, 1.0);

    // Arrival check fired → Stop; ErrorDecisionMaker was NOT called
    assert!(
        matches!(game.state.player_states[0].current_decision, Some(Decision::Stop)),
        "Arrival check must override to Stop without calling the script"
    );
    // No error must have been recorded
    assert!(game.state.player_states[0].last_error.is_none());
}

#[test]
fn test_setup_stop_blocks_script_even_when_needs_decision_true() {
    // A player with Stop in Setup must never be re-polled, even if some other system
    // (e.g. BallPossessionSystem in a future edge case) wrote needs_decision = true.
    // DecisionSystem is the final guard.
    let mut game = make_setup_game_with_player_at(30.0, 20.0);
    game.state.player_states[0].current_decision = Some(Decision::Stop);
    game.state.player_states[0].needs_decision = true; // externally forced

    let mut system = DecisionSystem::new().with_decision_maker(Box::new(ErrorDecisionMaker));
    system.update(&mut game, 1.0);

    // Script must not have been called — decision stays Stop, no error recorded
    assert!(
        matches!(game.state.player_states[0].current_decision, Some(Decision::Stop)),
        "Stop must be preserved"
    );
    assert!(game.state.player_states[0].last_error.is_none(), "script must not have been called");
    assert!(!game.state.player_states[0].needs_decision, "needs_decision must be cleared");
}
