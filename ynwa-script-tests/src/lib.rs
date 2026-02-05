// Test helpers for script testing
// This module is used by integration tests

use ynwa_core::field::Field;
use ynwa_core::game::{BallDef, Game, GameConfig, PlayerDef, RefereeDef};
use ynwa_core::region::{GridCell, Region};
use ynwa_core::team::Team;

/// Create a simple test game with one player using the given script
pub fn create_test_game_with_script(script: &str) -> Game {
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
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::new(config)
}
