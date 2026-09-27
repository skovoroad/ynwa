//! Codec-independent model of a full recording.

use crate::game::{Game, GameConfig, GameStage};
use crate::journal::JournalEntry;
use serde::{Deserialize, Serialize};

/// Everything needed to reconstruct the initial state before replaying.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub config: GameConfig,
    pub initial_stage: GameStage,
    /// Fixed simulation step used for physics integration during replay.
    pub fixed_dt: f32,
}

impl RecordHeader {
    /// Builds a header from the current configuration and stage of `game`, so callers do not
    /// duplicate `config`/`initial_stage` when starting a recording.
    pub fn from_game(game: &Game, fixed_dt: f32) -> Self {
        Self {
            config: game.config().clone(),
            initial_stage: game.state().stage.clone(),
            fixed_dt,
        }
    }
}

/// Header, step count and event journal of a playthrough.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub header: RecordHeader,
    pub total_steps: u64,
    pub journal: Vec<JournalEntry>,
}

#[cfg(test)]
#[path = "tests/record_tests.rs"]
mod tests;
