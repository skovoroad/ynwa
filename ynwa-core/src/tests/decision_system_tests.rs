use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
use crate::region::{GridCell};
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
