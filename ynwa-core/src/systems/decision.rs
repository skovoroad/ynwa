use crate::game::{Decision, DecisionTarget, Game};
use crate::region::GridCell;
use crate::system::System;
use rand::Rng;

/// Decision making system - processes player decisions
/// 
/// This is a placeholder system. In the future, multiple different decision systems
/// will be available, but only one will be used during gameplay.
pub struct DecisionSystem;

impl DecisionSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for DecisionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let grid_dims = game.config().field.grid_dimensions();
        let mut rng = rand::rng();
        
        for player_state in game.state.player_states.iter_mut() {
            if player_state.needs_decision {
                let col = rng.random_range(1..=grid_dims.columns);
                let row = rng.random_range(1..=grid_dims.rows);
                let cell = GridCell::new(col, row).expect("Generated cell should be valid");
                let decision = Decision::Run(DecisionTarget::GridCell(cell));
                
                player_state.current_decision = Some(decision);
                player_state.decision_processed = false;
                player_state.needs_decision = false;
                player_state.last_decision_time = timestamp;
            }
        }
    }
}

impl Default for DecisionSystem {
    fn default() -> Self {
        Self::new()
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
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
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
                // Check that the cell is within field bounds
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

        // First decision
        game.state.player_states[0].needs_decision = true;
        system.update(&mut game, 1.0);
        
        let first_decision = game.state.player_states[0].current_decision.clone();
        assert!(first_decision.is_some());

        // Mark as processed and no new decision needed
        game.state.player_states[0].decision_processed = true;
        game.state.player_states[0].needs_decision = false;

        system.update(&mut game, 2.0);

        // Decision should be preserved
        assert!(matches!(
            game.state.player_states[0].current_decision,
            Some(_)
        ));
        assert!(game.state.player_states[0].decision_processed);
    }
}
