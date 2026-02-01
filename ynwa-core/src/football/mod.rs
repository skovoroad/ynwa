pub mod field_builder;

pub use field_builder::{create_football_field, create_football_field_with_dimensions};

use crate::config::SerializableGameConfig;
use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
use crate::region::{GridCell, Region};
use crate::team::Team;

/// Creates a complete football game configuration with standard field and default entities
pub fn create_football_game_config() -> GameConfig {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();

    // Create default players - 11 per team
    let mut players = Vec::new();
    for i in 0..11 {
        // Create a simple start region for each player
        let row = i + 1;
        let start_region = Region::new(
            Team::A,
            GridCell::new(1, row).unwrap(),
            GridCell::new(2, row).unwrap(),
            grid_dims,
        )
        .unwrap();

        players.push(PlayerDef::new(
            Team::A,
            i + 1,
            format!("Player A{}", i + 1),
            50, // reaction_rate
            50, // speed_rate
            start_region,
        ));
    }
    for i in 0..11 {
        // Create a simple start region for each player
        let row = i + 1;
        let start_region = Region::new(
            Team::B,
            GridCell::new(25, row).unwrap(),
            GridCell::new(26, row).unwrap(),
            grid_dims,
        )
        .unwrap();

        players.push(PlayerDef::new(
            Team::B,
            i + 1,
            format!("Player B{}", i + 1),
            50, // reaction_rate
            50, // speed_rate
            start_region,
        ));
    }

    GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
    }
}

/// Creates a game configuration from a TOML file
pub fn create_football_game_config_from_file(path: &std::path::Path) -> Result<GameConfig, String> {
    let config = SerializableGameConfig::from_file(path)?;
    let field = create_football_field();
    config.to_game_config(field)
}

/// Creates a game configuration from a TOML string
pub fn create_football_game_config_from_toml(toml_str: &str) -> Result<GameConfig, String> {
    let config = SerializableGameConfig::from_toml(toml_str)?;
    let field = create_football_field();
    config.to_game_config(field)
}
