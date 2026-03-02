//! Abstraction layer for loading team and player data.
//! `TeamRepository` decouples the rest of the codebase from the concrete storage backend.

/// Immutable player attributes — independent of tactics or formation.
pub struct PlayerStatic {
    pub name: String,
    pub reaction_rate: u32,
    pub speed_rate: u32,
    pub tackle_rate: u32,
    pub shot_power: u32,
    pub shot_accuracy: u32,
}

/// Tactical player attributes — position within a specific formation.
pub struct PlayerTactical {
    /// Jersey number (1–99).
    pub number: u32,
    /// Grid notation, e.g. `"N3"`.
    pub start_position: String,
    pub attack_position: String,
    pub defence_position: String,
}

/// All data needed to build one player.
pub struct PlayerRecord {
    pub static_data: PlayerStatic,
    pub tactical: PlayerTactical,
    /// `None` means the player uses team tactics entirely (equivalent to an empty script).
    pub script: Option<String>,
}

/// All data needed to build one team.
pub struct TeamRecord {
    pub players: Vec<PlayerRecord>,
    /// Contents of `preamble.lua`.
    pub preamble: String,
}

pub trait TeamRepository {
    type Error: std::fmt::Display;

    fn load_team(&self, team_id: &str) -> Result<TeamRecord, Self::Error>;
}
