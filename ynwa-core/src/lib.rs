//! YNWA Football Manager - Core Library

pub mod config;
pub mod field;
pub mod game;
pub mod orientation;
pub mod physics_util;
pub mod region;
pub mod system;
pub mod systems;
pub mod team;
pub mod world;

pub use config::{PlayerConfig, SerializableGameConfig};
pub use field::zones::{Point3D, Velocity3D};
pub use game::{
    BallDef, BallState, Decision, DecisionTarget, Game, GameConfig, GameStage, GameState,
    PlayerDef, PlayerState, RefereeDef, RefereeState, StatSet,
};

pub use orientation::{
    flip_grid_cell_orientation, flip_point_orientation, flip_region_orientation,
};
pub use physics_util::{distance, distance_length};
pub use region::{GridCell, GridDimensions, Region, RegionError};
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
