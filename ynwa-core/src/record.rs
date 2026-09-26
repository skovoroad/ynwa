//! Codec-independent model of a full recording.

use crate::game::{GameConfig, GameStage};
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

/// Header, step count and event journal of a playthrough.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub header: RecordHeader,
    pub total_steps: u64,
    pub journal: Vec<JournalEntry>,
}
