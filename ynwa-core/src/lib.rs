//! YNWA Football Manager - Core Library

pub mod config;
pub mod field;
pub mod football;
pub mod game;
pub mod physics_util;
pub mod region;
pub mod scripting;
pub mod system;
pub mod systems;
pub mod team;
pub mod world;

pub use config::{PlayerConfig, SerializableGameConfig};
pub use field::zones::{Point3D, Velocity3D};
pub use game::{
    BallDef, BallState, Decision, DecisionTarget, Game, GameConfig, GameEvent, GameState,
    PlayerDef, PlayerState, RefereeDef, RefereeState,
};

pub use football::{
    create_football_world, create_football_world_from_file, create_football_world_from_toml,
};
pub use physics_util::{distance, distance_length};
pub use region::{GridCell, GridDimensions, Region, RegionError};
pub use scripting::{LuaExecutor, ScriptError, ScriptResult};
pub use system::System;
pub use systems::{
    ActionSystem, DecisionMaker, DecisionSystem, PhysicsSystem, PlaceholderDecisionMaker,
    PlayerReactionSystem,
};
pub use world::World;

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
