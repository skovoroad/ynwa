//! Codec-independent model of a full recording.

use crate::game::{Game, GameConfig, GameState, StatSet};
use crate::journal::JournalEntry;
use crate::team::Team;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Final state of a playthrough, captured as the recording's trailer.
///
/// Statistics are taken from `GameState` rather than reconstructed from the journal, so metrics
/// accumulated silently during simulation are preserved.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RecordFooter {
    pub total_steps: u64,
    pub elapsed_time: f32,
    #[serde(serialize_with = "crate::sorted_collections::map")]
    pub team_stats: HashMap<Team, StatSet>,
    /// Parallel to `GameState::player_stats`.
    pub player_stats: Vec<StatSet>,
}

impl RecordFooter {
    /// Builds a footer from the state reached at the end of a playthrough.
    pub fn from_state(state: &GameState) -> Self {
        Self {
            total_steps: state.step_count,
            elapsed_time: state.elapsed_time,
            team_stats: state.team_stats.clone(),
            player_stats: state.player_stats.clone(),
        }
    }
}

/// Header, event journal and footer of a playthrough.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub header: RecordHeader,
    pub footer: RecordFooter,
    pub journal: Vec<JournalEntry>,
}

impl Record {
    /// Assembles a record from the pre-step header, the final state and the buffered journal.
    pub fn from_final_state(
        header: RecordHeader,
        final_state: &GameState,
        journal: Vec<JournalEntry>,
    ) -> Self {
        Self {
            header,
            footer: RecordFooter::from_state(final_state),
            journal,
        }
    }
}

#[cfg(test)]
#[path = "tests/record_tests.rs"]
mod tests;
