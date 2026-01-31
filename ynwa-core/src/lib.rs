//! YNWA Football Manager - Core Library

pub mod field;
pub mod team;
pub mod football;
pub mod game;
pub mod region;

pub use game::{
    Game, GameConfig, GameState, GameEvent,
    PlayerDef, BallDef, RefereeDef,
    PlayerState, BallState, RefereeState,
};

pub use football::create_football_game_config;
pub use region::{Region, GridCell, RegionError};

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
