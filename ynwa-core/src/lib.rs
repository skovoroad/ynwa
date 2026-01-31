//! YNWA Football Manager - Core Library

pub mod config;
pub mod field;
pub mod football;
pub mod game;
pub mod region;
pub mod team;

pub use config::{PlayerConfig, SerializableGameConfig};
pub use game::{
    BallDef, BallState, Game, GameConfig, GameEvent, GameState, PlayerDef, PlayerState, RefereeDef,
    RefereeState,
};

pub use football::{
    create_football_game_config, 
    create_football_game_config_from_file,
    create_football_game_config_from_toml,
};
pub use region::{GridCell, GridDimensions, Region, RegionError};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub fn init() {
    println!("YNWA Core initialized (version {})", version());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!version().is_empty());
    }
}
