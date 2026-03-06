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
    /// Tactical number (1–N, contiguous within the team).
    pub number: u32,
    /// Grid notation, e.g. `"N3"`.
    pub start_position: String,
    pub attack_position: String,
    pub defence_position: String,
    pub goal_kick_own_position: Option<String>,
    pub goal_kick_opp_position: Option<String>,
    pub corner_own_left: Option<String>,
    pub corner_own_right: Option<String>,
    pub corner_opp_left: Option<String>,
    pub corner_opp_right: Option<String>,
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
    fn load_team(&self, team_id: &str) -> Result<TeamRecord, String>;
}
