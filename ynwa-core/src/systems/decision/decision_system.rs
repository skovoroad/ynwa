use crate::game::{convert_decision_to_display_orientation, Decision, DecisionTarget, Game};
use crate::region::GridCell;
use crate::system::System;
use rand::Rng;
use std::fmt;
use uom::si::length::meter;

// Design: DecisionSystem delegates decision-making to DecisionMaker implementations.
// This separates coordination (when to decide) from strategy (what to decide).

/// Errors that can occur during decision-making
#[derive(Debug, Clone)]
pub enum DecisionError {
    ScriptError(String),
    Timeout(String),
    RuntimeError(String),
}

impl fmt::Display for DecisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            DecisionError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            DecisionError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for DecisionError {}

pub trait DecisionMaker {
    fn make_decision(&mut self, game: &Game, player_index: usize) 
        -> Result<Decision, DecisionError>;
}

/// Temporary stub - generates random run decisions until real AI is implemented
pub struct PlaceholderDecisionMaker;

impl PlaceholderDecisionMaker {
    pub fn new() -> Self {
        Self
    }
}

impl DecisionMaker for PlaceholderDecisionMaker {
    fn make_decision(&mut self, game: &Game, _player_index: usize) 
        -> Result<Decision, DecisionError> 
    {
        let grid_dims = game.config().field.grid_dimensions();
        let mut rng = rand::rng();
        
        let col = rng.random_range(1..=grid_dims.columns);
        let row = rng.random_range(1..=grid_dims.rows);
        let cell = GridCell::new(col, row)
            .map_err(|e| DecisionError::RuntimeError(e.to_string()))?;
        
        Ok(Decision::Run(DecisionTarget::GridCell(cell)))
    }
}

impl Default for PlaceholderDecisionMaker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DecisionSystem {
    decision_maker: Box<dyn DecisionMaker>,
    on_error: fn(&DecisionError, usize) -> Option<Decision>,
}

impl DecisionSystem {
    pub fn new() -> Self {
        Self {
            decision_maker: Box::new(PlaceholderDecisionMaker),
            on_error: Self::default_error_handler,
        }
    }

    pub fn with_decision_maker(mut self, decision_maker: Box<dyn DecisionMaker>) -> Self {
        self.decision_maker = decision_maker;
        self
    }

    pub fn with_error_handler(mut self, handler: fn(&DecisionError, usize) -> Option<Decision>) -> Self {
        self.on_error = handler;
        self
    }

    fn default_error_handler(error: &DecisionError, player_index: usize) -> Option<Decision> {
        eprintln!("Player {} decision error: {}", player_index, error);
        None  // No decision on error by default
    }
}

impl Default for DecisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for DecisionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let player_count = game.state.player_states.len();
        
        for player_index in 0..player_count {
            if game.state.player_states[player_index].needs_decision {
                // Get decision or handle error
                let decision_result = self.decision_maker.make_decision(game, player_index);
                
                // Get player's team for coordinate conversion
                let player_team = game.config().players[player_index].team;
                
                // Get field dimensions for coordinate conversion
                let field_width = game.config().field.width().get::<meter>();
                let field_length = game.config().field.length().get::<meter>();
                let grid_dims = game.config().field.grid_dimensions();
                
                let player_state = &mut game.state.player_states[player_index];
                
                match decision_result {
                    Ok(decision) => {
                        // Convert decision from team's orientation to display orientation
                        let display_decision = convert_decision_to_display_orientation(
                            &decision,
                            player_team,
                            field_width,
                            field_length,
                            grid_dims,
                        );
                        
                        // Success: set the decision
                        player_state.current_decision = Some(display_decision);
                        player_state.decision_processed = false;
                        player_state.needs_decision = false;
                        player_state.last_decision_time = timestamp;
                        player_state.last_error = None; // Clear any previous error
                    }
                    Err(error) => {
                        // Error: call error handler and save error message for UI
                        let error_message = error.to_string();
                        let error_decision = (self.on_error)(&error, player_index);
                        
                        // If error handler provides a decision, convert it too
                        let converted_error_decision = error_decision.map(|d| {
                            convert_decision_to_display_orientation(
                                &d,
                                player_team,
                                field_width,
                                field_length,
                                grid_dims,
                            )
                        });
                        
                        // Always treat error as "completed attempt" to prevent storm
                        // This ensures rate-limiting via PlayerReactionSystem's reaction_rate
                        player_state.current_decision = converted_error_decision;
                        player_state.decision_processed = false;
                        player_state.needs_decision = false;
                        player_state.last_decision_time = timestamp;
                        player_state.last_error = Some(error_message);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_game() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                50,
                50,
                50,
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

        let decision = maker.make_decision(&game, 0).expect("Should not error");

        match decision {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                let grid_dims = game.config().field.grid_dimensions();
                assert!(cell.col >= 1 && cell.col <= grid_dims.columns);
                assert!(cell.row >= 1 && cell.row <= grid_dims.rows);
            }
            _ => panic!("Expected Run decision with GridCell target"),
        }
    }

    // Test error handling
    struct ErrorDecisionMaker;
    
    impl DecisionMaker for ErrorDecisionMaker {
        fn make_decision(&mut self, _game: &Game, _player_index: usize) 
            -> Result<Decision, DecisionError> 
        {
            Err(DecisionError::ScriptError("Test error".to_string()))
        }
    }

    #[test]
    fn test_decision_system_handles_error_with_default_handler() {
        let mut game = create_test_game();
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(ErrorDecisionMaker));

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
                Some(Decision::Run(DecisionTarget::GridCell(GridCell::new(13, 22).unwrap())))
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
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(ErrorDecisionMaker));

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
            fn make_decision(&mut self, _game: &Game, _idx: usize) -> Result<Decision, DecisionError> {
                let mut count = self.attempts.lock().unwrap();
                *count += 1;
                
                if *count == 1 {
                    Err(DecisionError::ScriptError("First attempt fails".to_string()))
                } else {
                    Ok(Decision::Run(DecisionTarget::GridCell(GridCell::new(13, 22).unwrap())))
                }
            }
        }
        
        let attempts = Arc::new(Mutex::new(0));
        let mut game = create_test_game();
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(FlakeyDecisionMaker { 
                attempts: attempts.clone() 
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
            fn make_decision(&mut self, _game: &Game, _idx: usize) -> Result<Decision, DecisionError> {
                Err(DecisionError::Timeout("Execution took too long".to_string()))
            }
        }
        
        let mut game = create_test_game();
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(TimeoutDecisionMaker));

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
            fn make_decision(&mut self, _game: &Game, _idx: usize) -> Result<Decision, DecisionError> {
                Err(DecisionError::RuntimeError("Internal error".to_string()))
            }
        }
        
        let mut game = create_test_game();
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(RuntimeErrorDecisionMaker));

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
                GridCell::new(13, (idx as u32) + 1).unwrap()
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
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(ErrorDecisionMaker));

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
        let mut system = DecisionSystem::new()
            .with_decision_maker(Box::new(ErrorDecisionMaker));
        
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
}
