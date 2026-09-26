//! YNWA Football Manager - Core Library

pub mod codec;
pub mod field;
pub mod game;
pub mod journal;
pub mod orientation;
pub mod physics_util;
pub mod record;
pub mod region;
pub mod repository;
pub mod rng;
pub mod system;
pub mod systems;
pub mod team;
pub mod world;

#[cfg(test)]
mod test_utils;

#[cfg(test)]
#[path = "tests/codec_tests.rs"]
mod codec_tests;

#[cfg(test)]
#[path = "tests/journal_tests.rs"]
mod journal_tests;

#[cfg(test)]
#[path = "tests/serde_tests.rs"]
mod serde_tests;

pub use codec::{
    FileJournalRecorder, JsonRecordCodec, JsonRecordReader, JsonRecordWriter, RecordReader,
    RecordWriter,
};
pub use field::zones::{Point3D, Velocity3D};
pub use game::{
    BallDef, BallState, Decision, DecisionTarget, Game, GameConfig, GameStage, GameState,
    PlayerDef, PlayerState, RefereeDef, RefereeState, StatSet, REGION_START_POSITION,
};
pub use journal::{
    CollectJournalRecorder, EventsCollection, JournalEntry, JournalEvent, JournalSink,
    NullJournalSink,
};
pub use record::{Record, RecordHeader};

pub use orientation::{
    flip_grid_cell_orientation, flip_point_orientation, flip_region_orientation,
};
pub use physics_util::{distance, distance_length};
pub use region::{GridCell, GridDimensions, Region, RegionError};
pub use repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord, TeamRepository};
pub use rng::{DefaultRngManager, RngConfig, RngManager};
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
