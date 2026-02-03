use crate::game::Game;
use crate::system::System;

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
        let state = &mut game.state;

        for player_state in state.player_states.iter_mut() {
            if player_state.needs_decision {
                // TODO: Actual decision making logic will be implemented here
                
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
}
