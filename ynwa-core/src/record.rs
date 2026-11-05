//! Codec-independent model of a full recording.

use crate::game::{Game, GameConfig, GameState};
use crate::journal::JournalEntry;
use serde::{Deserialize, Serialize};

/// Everything needed to reconstruct the initial state before replaying.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordHeader {
    pub config: GameConfig,
    /// Full state of the game at the moment the recording started; the stage is part of it.
    pub initial_state: GameState,
    /// Fixed simulation step used for physics integration during replay.
    pub timestep: f32,
}

impl RecordHeader {
    /// Builds a header from the current configuration and state of `game`, so callers do not
    /// duplicate `config`/`initial_state` when starting a recording.
    pub fn from_game(game: &Game, timestep: f32) -> Self {
        Self {
            config: game.config().clone(),
            initial_state: game.state().clone(),
            timestep,
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
