use crate::game::{Game, GameStage};
use crate::region::Region;
use crate::system::System;
use uom::si::length::meter;

/// Football game manager - manages football-specific game logic
pub struct FootballGameManager;

impl FootballGameManager {
    pub fn new() -> Self {
        Self
    }
}

impl System for FootballGameManager {
    fn update(&mut self, game: &mut Game, _timestamp: f32) {
        match &game.state.stage {
            GameStage::Setup(stage_name) if stage_name == "Prepare" => {
                self.check_player_readiness(game);

                // If all players are ready, transition to Play
                if game.state.player_states.iter().all(|p| p.is_ready) {
                    game.state.stage = GameStage::Play;
                }
            }
            _ => {}
        }
    }
}

impl FootballGameManager {
    fn check_player_readiness(&self, game: &mut Game) {
        let field_width = game.config().field.width().get::<meter>();
        let grid_dims = game.config().field.grid_dimensions();

        // Collect player start regions first to avoid borrowing issues
        let start_regions: Vec<_> = game
            .config()
            .players
            .iter()
            .map(|player_def| {
                player_def
                    .regions
                    .get("start position")
                    .expect("Player must have 'start position' region")
                    .clone()
            })
            .collect();

        for (idx, player_state) in game.state.player_states.iter_mut().enumerate() {
            if player_state.is_ready {
                continue; // Already ready
            }

            let start_region = &start_regions[idx];

            // Check if player is inside their start region
            if is_player_in_start_region(
                &player_state.position,
                start_region,
                grid_dims,
                field_width,
            ) {
                player_state.is_ready = true;
            }
        }
    }
}

fn is_player_in_start_region(
    position: &crate::field::zones::Point3D,
    start_region: &Region,
    grid_dims: crate::region::GridDimensions,
    field_width: f32,
) -> bool {
    start_region.contains_point(grid_dims, field_width, position)
}

impl Default for FootballGameManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, GameStage, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;
    use uom::si::length::meter;

    fn create_test_game_setup() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![
                PlayerDef::new(
                    Team::A,
                    1,
                    "Test Player 1".to_string(),
                    50,
                    50,
                    50,
                    50,
                    50,
                    "function make_decision() return {} end".to_string(),
                    start_region.clone(),
                ),
                PlayerDef::new(
                    Team::A,
                    2,
                    "Test Player 2".to_string(),
                    50,
                    50,
                    50,
                    50,
                    50,
                    "function make_decision() return {} end".to_string(),
                    start_region,
                ),
            ],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::with_stage(config, GameStage::Setup("Prepare".to_string()))
    }

    #[test]
    fn test_players_start_at_edge_in_setup() {
        let game = create_test_game_setup();

        let field_width = game.config().field.width().get::<meter>();
        let expected_x = field_width / 2.0; // Center along width
        let expected_z = -5.0; // Behind goal line

        for (idx, player_state) in game.state.player_states.iter().enumerate() {
            assert!(
                (player_state.position.x.get::<meter>() - expected_x).abs() < 0.01,
                "Player {} X: {} vs expected {}",
                idx,
                player_state.position.x.get::<meter>(),
                expected_x
            );
            assert!(
                (player_state.position.z.get::<meter>() - expected_z).abs() < 0.01,
                "Player {} Z: {} vs expected {}",
                idx,
                player_state.position.z.get::<meter>(),
                expected_z
            );
            assert!(!player_state.is_ready);
        }
    }

    #[test]
    fn test_check_player_readiness_when_not_in_region() {
        let mut game = create_test_game_setup();
        let mut manager = FootballGameManager::new();

        // Players are at edge, not in start region
        manager.update(&mut game, 0.0);

        // Should still be in Setup stage
        assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
        assert!(!game.state.player_states[0].is_ready);
        assert!(!game.state.player_states[1].is_ready);
    }

    #[test]
    fn test_check_player_readiness_when_in_region() {
        let mut game = create_test_game_setup();
        let mut manager = FootballGameManager::new();

        // Move player 0 into start region
        let start_region = &game.config().players[0].regions["start position"];
        let center = start_region.center(
            game.config().field.grid_dimensions(),
            game.config().field.width().get::<meter>(),
        );
        game.state.player_states[0].position = center;

        manager.update(&mut game, 0.0);

        // Player 0 should be ready, player 1 not
        assert!(game.state.player_states[0].is_ready);
        assert!(!game.state.player_states[1].is_ready);

        // Should still be in Setup (not all ready)
        assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
    }

    #[test]
    fn test_transition_to_play_when_all_ready() {
        let mut game = create_test_game_setup();
        let mut manager = FootballGameManager::new();

        // Collect start region centers first to avoid borrowing issues
        let centers: Vec<_> = game
            .config()
            .players
            .iter()
            .map(|player_def| {
                let start_region = &player_def.regions["start position"];
                start_region.center(
                    game.config().field.grid_dimensions(),
                    game.config().field.width().get::<meter>(),
                )
            })
            .collect();

        // Move all players into their start regions
        for (idx, player_state) in game.state.player_states.iter_mut().enumerate() {
            player_state.position = centers[idx].clone();
        }

        manager.update(&mut game, 0.0);

        // All players should be ready
        assert!(game.state.player_states[0].is_ready);
        assert!(game.state.player_states[1].is_ready);

        // Should transition to Play
        assert_eq!(game.state.stage, GameStage::Play);
    }

    #[test]
    fn test_no_updates_in_play_stage() {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();
        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
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
                50,
                50,
                "function make_decision() return {} end".to_string(),
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::with_stage(config, GameStage::Play);
        let mut manager = FootballGameManager::new();

        let initial_stage = game.state.stage.clone();
        manager.update(&mut game, 0.0);

        // Stage should remain unchanged
        assert_eq!(game.state.stage, initial_stage);
    }
}
