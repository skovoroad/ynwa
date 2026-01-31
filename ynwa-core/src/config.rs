use crate::game::{GameConfig, PlayerDef};
use crate::region::Region;
use crate::team::Team;
use serde::{Deserialize, Serialize};



/// Player configuration for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub team: String,  // "A" or "B"
    pub number: u32,
    pub name: String,
    pub start_position: String,  // Grid notation like "A1:B2"
}

impl PlayerConfig {
    /// Convert to PlayerDef using field's grid dimensions
    pub fn to_player_def(&self, grid_dims: crate::region::GridDimensions) -> Result<PlayerDef, String> {
        let team = match self.team.as_str() {
            "A" => Team::A,
            "B" => Team::B,
            _ => return Err(format!("Invalid team '{}'. Must be 'A' or 'B'", self.team)),
        };
        
        let start_region = Region::from_grid_notation(&self.start_position, team, grid_dims)
            .map_err(|e| format!("Invalid start position '{}': {}", self.start_position, e))?;
        
        Ok(PlayerDef::new(team, self.number, self.name.clone(), start_region))
    }
    
    /// Create from PlayerDef
    pub fn from_player_def(player_def: &PlayerDef) -> Result<Self, String> {
        let team = match player_def.team {
            Team::A => "A".to_string(),
            Team::B => "B".to_string(),
        };
        
        let start_region = player_def.regions.get("start position")
            .ok_or("Player must have 'start position' region")?;
        
        Ok(Self {
            team,
            number: player_def.number,
            name: player_def.name.clone(),
            start_position: start_region.to_grid_notation(),
        })
    }
}

/// Game configuration for serialization (players only, field is in code)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGameConfig {
    pub players: Vec<PlayerConfig>,
}

impl SerializableGameConfig {
    /// Load from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str)
            .map_err(|e| format!("Failed to parse TOML: {}", e))
    }
    
    /// Load from TOML file
    pub fn from_file(path: &std::path::Path) -> Result<Self, String> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file '{}': {}", path.display(), e))?;
        Self::from_toml(&content)
    }
    
    /// Save to TOML string
    pub fn to_toml(&self) -> Result<String, String> {
        toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize to TOML: {}", e))
    }
    
    /// Save to TOML file
    pub fn to_file(&self, path: &std::path::Path) -> Result<(), String> {
        let toml_str = self.to_toml()?;
        std::fs::write(path, toml_str)
            .map_err(|e| format!("Failed to write file '{}': {}", path.display(), e))
    }
    
    /// Convert to GameConfig (requires field to be provided)
    pub fn to_game_config(&self, field: crate::field::Field) -> Result<GameConfig, String> {
        let grid_dims = field.grid_dimensions();
        let mut players = Vec::new();
        
        for player_config in &self.players {
            players.push(player_config.to_player_def(grid_dims)?);
        }
        
        Ok(GameConfig {
            field,
            players,
            ball: crate::game::BallDef::default(),
            referees: vec![crate::game::RefereeDef::default()],
        })
    }
    
    /// Create from GameConfig
    pub fn from_game_config(game_config: &GameConfig) -> Result<Self, String> {
        let mut players = Vec::new();
        
        for player_def in &game_config.players {
            players.push(PlayerConfig::from_player_def(player_def)?);
        }
        
        Ok(Self { players })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;

    #[test]
    fn test_player_config_roundtrip() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();
        
        let config = PlayerConfig {
            team: "A".to_string(),
            number: 10,
            name: "Test Player".to_string(),
            start_position: "C3:D4".to_string(),
        };
        
        let player_def = config.to_player_def(grid_dims).unwrap();
        assert_eq!(player_def.team, Team::A);
        assert_eq!(player_def.number, 10);
        assert_eq!(player_def.name, "Test Player");
        
        let config_back = PlayerConfig::from_player_def(&player_def).unwrap();
        assert_eq!(config_back.team, "A");
        assert_eq!(config_back.number, 10);
        assert_eq!(config_back.start_position, "C3:D4");
    }

    #[test]
    fn test_serializable_config_toml() {
        let config = SerializableGameConfig {
            players: vec![
                PlayerConfig {
                    team: "A".to_string(),
                    number: 1,
                    name: "Goalkeeper".to_string(),
                    start_position: "A22:B24".to_string(),
                },
                PlayerConfig {
                    team: "B".to_string(),
                    number: 10,
                    name: "Striker".to_string(),
                    start_position: "Y22:Z24".to_string(),
                },
            ],
        };
        
        let toml_str = config.to_toml().unwrap();
        assert!(toml_str.contains("team = \"A\""));
        assert!(toml_str.contains("name = \"Goalkeeper\""));
        
        let parsed = SerializableGameConfig::from_toml(&toml_str).unwrap();
        assert_eq!(parsed.players.len(), 2);
        assert_eq!(parsed.players[0].name, "Goalkeeper");
        assert_eq!(parsed.players[1].number, 10);
    }

    #[test]
    fn test_player_config_invalid_team() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();
        
        let config = PlayerConfig {
            team: "C".to_string(), // Invalid team
            number: 10,
            name: "Test Player".to_string(),
            start_position: "C3:D4".to_string(),
        };
        
        let result = config.to_player_def(grid_dims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid team"));
    }

    #[test]
    fn test_player_config_invalid_grid_notation() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();
        
        let config = PlayerConfig {
            team: "A".to_string(),
            number: 10,
            name: "Test Player".to_string(),
            start_position: "INVALID".to_string(), // Invalid notation
        };
        
        let result = config.to_player_def(grid_dims);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid start position"));
    }

    #[test]
    fn test_player_config_out_of_bounds() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();
        
        let config = PlayerConfig {
            team: "A".to_string(),
            number: 10,
            name: "Test Player".to_string(),
            start_position: "A1:AA50".to_string(), // Out of bounds
        };
        
        let result = config.to_player_def(grid_dims);
        assert!(result.is_err());
    }

    #[test]
    fn test_game_config_roundtrip() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        
        let serializable_config = SerializableGameConfig {
            players: vec![
                PlayerConfig {
                    team: "A".to_string(),
                    number: 1,
                    name: "Goalkeeper".to_string(),
                    start_position: "A22:B24".to_string(),
                },
                PlayerConfig {
                    team: "B".to_string(),
                    number: 9,
                    name: "Forward".to_string(),
                    start_position: "Y22:Z24".to_string(),
                },
            ],
        };
        
        // Convert to GameConfig
        let game_config = serializable_config.to_game_config(field.clone()).unwrap();
        assert_eq!(game_config.players.len(), 2);
        assert_eq!(game_config.players[0].name, "Goalkeeper");
        assert_eq!(game_config.players[0].number, 1);
        assert_eq!(game_config.players[1].name, "Forward");
        assert_eq!(game_config.players[1].number, 9);
        
        // Convert back to SerializableGameConfig
        let config_back = SerializableGameConfig::from_game_config(&game_config).unwrap();
        assert_eq!(config_back.players.len(), 2);
        assert_eq!(config_back.players[0].team, "A");
        assert_eq!(config_back.players[0].start_position, "A22:B24");
        assert_eq!(config_back.players[1].team, "B");
        assert_eq!(config_back.players[1].start_position, "Y22:Z24");
    }

    #[test]
    fn test_to_game_config_with_invalid_player() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        
        let config = SerializableGameConfig {
            players: vec![
                PlayerConfig {
                    team: "X".to_string(), // Invalid team
                    number: 1,
                    name: "Bad Player".to_string(),
                    start_position: "A1:B2".to_string(),
                },
            ],
        };
        
        let result = config.to_game_config(field);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid team"));
    }
}
