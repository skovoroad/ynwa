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
    /// Positions in Play phase. Keys are sport-specific (e.g. `"attack"`, `"defence"`).
    pub play_positions: std::collections::HashMap<String, String>,
    /// Set-piece positions. Keys and their meaning are defined by the sport layer.
    /// Value is a grid notation string (e.g. `"K7"`) or the special marker `"on_ball"`.
    pub set_piece_positions: std::collections::HashMap<String, String>,
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
