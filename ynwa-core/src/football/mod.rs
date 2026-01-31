pub mod field_builder;

pub use field_builder::{create_football_field, create_football_field_with_dimensions};

use crate::game::{GameConfig, PlayerDef, BallDef, RefereeDef};
use crate::team::Team;

/// Creates a complete football game configuration with standard field and default entities
pub fn create_football_game_config() -> GameConfig {
    let field = create_football_field();
    
    // Create default players - 11 per team
    let mut players = Vec::new();
    for _ in 0..11 {
        players.push(PlayerDef { team: Team::A });
    }
    for _ in 0..11 {
        players.push(PlayerDef { team: Team::B });
    }
    
    GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
    }
}
