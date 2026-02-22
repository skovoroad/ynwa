use crate::game::{GameConfig, PlayerDef};
use crate::region::Region;
use crate::team::Team;
use serde::{Deserialize, Serialize};

/// Player configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub team: String, // "A" or "B"
    pub number: u32,
    pub name: String,
    pub reaction_rate: u32, // 10-100: player's reaction speed
    pub speed_rate: u32,    // 10-100: player's movement speed
    #[serde(default = "default_tackle_rate")]
    pub tackle_rate: u32, // 10-100: player's ball control ability
    #[serde(default = "default_shot_power")]
    pub shot_power: u32, // 10-100: player's shot power
    #[serde(default = "default_shot_accuracy")]
    pub shot_accuracy: u32, // 10-100: player's shot accuracy
    pub start_position: String, // Grid notation like "A1:B2"
    pub attack_position: String, // Grid notation for attacking position
    pub defence_position: String, // Grid notation for defensive position
    pub script: String,     // Lua script for decision making (mandatory)
}

fn default_tackle_rate() -> u32 {
    50
}

fn default_shot_power() -> u32 {
    50
}

fn default_shot_accuracy() -> u32 {
    50
}

impl PlayerConfig {
    /// Convert to PlayerDef using field's grid dimensions.
    /// Player positions in config are specified in team's own orientation.
    /// This method converts them to absolute field coordinates (Team A orientation).
    pub fn to_player_def(
        &self,
        grid_dims: crate::region::GridDimensions,
    ) -> Result<PlayerDef, String> {
        let team = match self.team.as_str() {
            "A" => Team::A,
            "B" => Team::B,
            _ => return Err(format!("Invalid team '{}'. Must be 'A' or 'B'", self.team)),
        };

        // Parse region in team's own orientation
        let start_region = Region::from_grid_notation(&self.start_position, team, grid_dims)
            .map_err(|e| format!("Invalid start position '{}': {}", self.start_position, e))?;
        
        let attack_region = Region::from_grid_notation(&self.attack_position, team, grid_dims)
            .map_err(|e| format!("Invalid attack position '{}': {}", self.attack_position, e))?;
        
        let defence_region = Region::from_grid_notation(&self.defence_position, team, grid_dims)
            .map_err(|e| format!("Invalid defence position '{}': {}", self.defence_position, e))?;

        // Convert to absolute field coordinates (Team A orientation)
        let start_region_absolute = if team == Team::B {
            start_region
                .flip_orientation(grid_dims)
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            start_region
        };

        let attack_region_absolute = if team == Team::B {
            attack_region
                .flip_orientation(grid_dims)
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            attack_region
        };

        let defence_region_absolute = if team == Team::B {
            defence_region
                .flip_orientation(grid_dims)
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            defence_region
        };

        Ok(PlayerDef::new(
            team,
            self.number,
            self.name.clone(),
            self.script.clone(),
            start_region_absolute,
        )
        .with_reaction_rate(self.reaction_rate)
        .with_speed_rate(self.speed_rate)
        .with_tackle_rate(self.tackle_rate)
        .with_shot_power(self.shot_power)
        .with_shot_accuracy(self.shot_accuracy)
        .with_attack_position(attack_region_absolute)
        .with_defence_position(defence_region_absolute))
    }

    /// Create from PlayerDef.
    /// Converts absolute field coordinates back to team's own orientation.
    pub fn from_player_def(player_def: &PlayerDef) -> Result<Self, String> {
        let team = match player_def.team {
            Team::A => "A".to_string(),
            Team::B => "B".to_string(),
        };

        let start_region = player_def
            .regions
            .get("start position")
            .ok_or("Player must have 'start position' region")?;

        let attack_region = player_def
            .regions
            .get("attack position")
            .ok_or("Player must have 'attack position' region")?;

        let defence_region = player_def
            .regions
            .get("defence position")
            .ok_or("Player must have 'defence position' region")?;

        // Convert from absolute coordinates to team's own orientation
        let start_region_own = if player_def.team == Team::B {
            start_region
                .flip_orientation(crate::region::GridDimensions::new(26, 44))
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            start_region.clone()
        };

        let attack_region_own = if player_def.team == Team::B {
            attack_region
                .flip_orientation(crate::region::GridDimensions::new(26, 44))
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            attack_region.clone()
        };

        let defence_region_own = if player_def.team == Team::B {
            defence_region
                .flip_orientation(crate::region::GridDimensions::new(26, 44))
                .map_err(|e| format!("Failed to flip region orientation: {}", e))?
        } else {
            defence_region.clone()
        };

        Ok(Self {
            team,
            number: player_def.number,
            name: player_def.name.clone(),
            reaction_rate: player_def.reaction_rate,
            speed_rate: player_def.speed_rate,
            tackle_rate: player_def.tackle_rate,
            shot_power: player_def.shot_power,
            shot_accuracy: player_def.shot_accuracy,
            start_position: start_region_own.to_grid_notation(),
            attack_position: attack_region_own.to_grid_notation(),
            defence_position: defence_region_own.to_grid_notation(),
            script: player_def.script.clone(),
        })
    }
}

/// Game configuration for serialization (players only, field is in code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameConfig {
    pub players: Vec<PlayerConfig>,
    /// Paths to preamble files (optional, will use defaults if not specified)
    #[serde(default)]
    pub core_preamble_path: Option<String>,
    #[serde(default)]
    pub stdlib_preamble_path: Option<String>,
    #[serde(default)]
    pub team_a_preamble_path: Option<String>,
    #[serde(default)]
    pub team_b_preamble_path: Option<String>,
    // ...existing code...
}

impl SerializableGameConfig {
    /// Load from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str).map_err(|e| format!("Failed to parse TOML: {}", e))
    }

    /// Load from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;
        Self::from_toml(&content)
    }

    /// Save to TOML string
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self).map_err(|e| format!("Failed to serialize to TOML: {}", e))
    }

    /// Save to TOML file
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), String> {
        let toml_str = self.to_toml()?;
        std::fs::write(path, toml_str)
            .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
    }

    /// Convert to GameConfig (requires field to be provided)
    ///
    /// If `config_dir` is provided, preamble paths are resolved relative to it.
    /// Otherwise, they are resolved relative to the current working directory.
    pub fn to_game_config(
        &self,
        field: crate::field::Field,
        config_dir: Option<&std::path::Path>,
    ) -> Result<GameConfig, String> {
        let grid_dims = field.grid_dimensions();
        let mut players = Vec::new();

        for player_config in &self.players {
            players.push(player_config.to_player_def(grid_dims)?);
        }

        // Helper to resolve path relative to config_dir if provided
        let resolve_path = |path: &str| -> std::path::PathBuf {
            if let Some(base) = config_dir {
                base.join(path)
            } else {
                std::path::PathBuf::from(path)
            }
        };

        // Load preambles from files if paths are specified
        let core_preamble = if let Some(path) = &self.core_preamble_path {
            let full_path = resolve_path(path);
            std::fs::read_to_string(&full_path).map_err(|e| {
                format!(
                    "Failed to read core preamble from '{}': {}",
                    full_path.display(),
                    e
                )
            })?
        } else {
            String::new()
        };

        let stdlib_preamble = if let Some(path) = &self.stdlib_preamble_path {
            let full_path = resolve_path(path);
            std::fs::read_to_string(&full_path).map_err(|e| {
                format!(
                    "Failed to read stdlib preamble from '{}': {}",
                    full_path.display(),
                    e
                )
            })?
        } else {
            String::new()
        };

        let team_a_preamble = if let Some(path) = &self.team_a_preamble_path {
            let full_path = resolve_path(path);
            std::fs::read_to_string(&full_path).map_err(|e| {
                format!(
                    "Failed to read team A preamble from '{}': {}",
                    full_path.display(),
                    e
                )
            })?
        } else {
            String::new()
        };

        let team_b_preamble = if let Some(path) = &self.team_b_preamble_path {
            let full_path = resolve_path(path);
            std::fs::read_to_string(&full_path).map_err(|e| {
                format!(
                    "Failed to read team B preamble from '{}': {}",
                    full_path.display(),
                    e
                )
            })?
        } else {
            String::new()
        };

        Ok(GameConfig {
            field,
            players,
            ball: crate::game::BallDef::default(),
            referees: vec![crate::game::RefereeDef::default()],
            scripting: crate::game::ScriptingConfig {
                core_preamble,
                stdlib_preamble,
                team_a_preamble,
                team_b_preamble,
            },
        })
    }

    /// Create from GameConfig
    pub fn from_game_config(game_config: &GameConfig) -> Result<Self, String> {
        let mut players = Vec::new();

        for player_def in &game_config.players {
            players.push(PlayerConfig::from_player_def(player_def)?);
        }

        Ok(Self {
            players,
            core_preamble_path: None,
            stdlib_preamble_path: None,
            team_a_preamble_path: None,
            team_b_preamble_path: None,
        })
    }
}


#[cfg(test)]
#[path = "tests/config_tests.rs"]
mod tests;
