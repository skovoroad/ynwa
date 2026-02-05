use crate::game::Game;
use crate::system::System;

/// Player reaction system - determines when players need to make decisions
pub struct PlayerReactionSystem;

impl PlayerReactionSystem {
    pub fn new() -> Self {
        Self
    }

    // TODO: cache the value
    fn reaction_interval(reaction_rate: u32) -> f32 {
        // reaction_rate 100 -> 0.5s, reaction_rate 10 -> 3.0s (linear)
        // Formula: interval = 0.5 + (100 - rate) * (3.0 - 0.5) / (100 - 10)
        let rate = reaction_rate.clamp(10, 100) as f32;
        0.5 + (100.0 - rate) * 2.5 / 90.0
    }
}

impl System for PlayerReactionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let player_count = game.config().players.len();
        
        for i in 0..player_count {
            let reaction_rate = game.config().players[i].reaction_rate;
            let interval = Self::reaction_interval(reaction_rate);
            let player_state = &mut game.state.player_states[i];

            if timestamp - player_state.last_decision_time >= interval {
                player_state.needs_decision = true;
            }
        }
    }
}

impl Default for PlayerReactionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::field::Field;
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_game() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 11);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let players = vec![
            PlayerDef::new(Team::A, 1, "Player 1".to_string(), 100, 50, 50, "function make_decision() return {} end".to_string(), start_region.clone()),
            PlayerDef::new(Team::A, 2, "Player 2".to_string(), 55, 50, 50, "function make_decision() return {} end".to_string(), start_region.clone()),
            PlayerDef::new(Team::A, 3, "Player 3".to_string(), 10, 50, 50, "function make_decision() return {} end".to_string(), start_region.clone()),
        ];

        let config = GameConfig {
            field,
            players,
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
        };

        Game::new(config)
    }

    #[test]
    fn test_reaction_interval() {
        assert!((PlayerReactionSystem::reaction_interval(100) - 0.5).abs() < 0.01);
        assert!((PlayerReactionSystem::reaction_interval(10) - 3.0).abs() < 0.01);
        assert!((PlayerReactionSystem::reaction_interval(55) - 1.75).abs() < 0.01);
    }

    #[test]
    fn test_reaction_interval_clamping() {
        assert!((PlayerReactionSystem::reaction_interval(0) - 3.0).abs() < 0.01);
        assert!((PlayerReactionSystem::reaction_interval(150) - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_update_sets_needs_decision_flag() {
        let mut game = create_test_game();
        let mut system = PlayerReactionSystem::new();

        // Initially all players need decision
        assert!(game.state.player_states[0].needs_decision);
        assert!(game.state.player_states[1].needs_decision);
        assert!(game.state.player_states[2].needs_decision);

        // Clear flags manually
        for player_state in &mut game.state.player_states {
            player_state.needs_decision = false;
        }

        // Player 1 (rate=100): interval=0.5s, should need decision at 0.5s
        system.update(&mut game, 0.5);
        assert!(game.state.player_states[0].needs_decision);
        assert!(!game.state.player_states[1].needs_decision); // interval=1.75s
        assert!(!game.state.player_states[2].needs_decision); // interval=3.0s
    }

    #[test]
    fn test_update_respects_different_reaction_rates() {
        let mut game = create_test_game();
        let mut system = PlayerReactionSystem::new();

        // Clear all flags
        for player_state in &mut game.state.player_states {
            player_state.needs_decision = false;
        }

        // At 2.0s:
        // Player 1 (rate=100, interval=0.5s): should need decision multiple times
        // Player 2 (rate=55, interval=1.75s): should need decision once
        // Player 3 (rate=10, interval=3.0s): should NOT need decision yet
        system.update(&mut game, 2.0);
        
        assert!(game.state.player_states[0].needs_decision);
        assert!(game.state.player_states[1].needs_decision);
        assert!(!game.state.player_states[2].needs_decision);
    }

    #[test]
    fn test_update_uses_last_decision_time() {
        let mut game = create_test_game();
        let mut system = PlayerReactionSystem::new();

        // Set last_decision_time for player 1 to 1.0s
        game.state.player_states[0].last_decision_time = 1.0;
        game.state.player_states[0].needs_decision = false;

        // At timestamp 1.4s, only 0.4s passed since last decision (interval=0.5s)
        system.update(&mut game, 1.4);
        assert!(!game.state.player_states[0].needs_decision);

        // At timestamp 1.6s, 0.6s passed since last decision - should need decision
        system.update(&mut game, 1.6);
        assert!(game.state.player_states[0].needs_decision);
    }

    #[test]
    fn test_update_does_not_clear_needs_decision_flag() {
        let mut game = create_test_game();
        let mut system = PlayerReactionSystem::new();

        // Player already needs decision
        game.state.player_states[0].needs_decision = true;
        game.state.player_states[0].last_decision_time = 0.0;

        // Run update - flag should remain true
        system.update(&mut game, 0.6);
        assert!(game.state.player_states[0].needs_decision);
    }
}
