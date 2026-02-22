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
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "C3:D4".to_string(),
        attack_position: "C1:D2".to_string(), // 2 rows forward (towards opponent goal)
        defence_position: "C5:D6".to_string(), // 2 rows backward (towards own goal)
        script: "function make_decision() return {} end".to_string(),
    };

    let player_def = config.to_player_def(grid_dims).unwrap();
    assert_eq!(player_def.team, Team::A);
    assert_eq!(player_def.number, 10);
    assert_eq!(player_def.name, "Test Player");
    assert_eq!(player_def.script, "function make_decision() return {} end");

    let config_back = PlayerConfig::from_player_def(&player_def).unwrap();
    assert_eq!(config_back.team, "A");
    assert_eq!(config_back.number, 10);
    assert_eq!(config_back.start_position, "C3:D4");
    assert_eq!(config_back.script, "function make_decision() return {} end");
}

#[test]
fn test_serializable_config_toml() {
    let config = SerializableGameConfig {
        players: vec![
            PlayerConfig {
                team: "A".to_string(),
                number: 1,
                name: "Goalkeeper".to_string(),
                reaction_rate: 50,
                speed_rate: 50,
                tackle_rate: 50,
                shot_power: 50,
                shot_accuracy: 50,
                start_position: "A22:B24".to_string(),
                attack_position: "A20:B22".to_string(),
                defence_position: "A24:B26".to_string(),
                script: "function make_decision() return {} end".to_string(),
            },
            PlayerConfig {
                team: "B".to_string(),
                number: 10,
                name: "Striker".to_string(),
                reaction_rate: 50,
                speed_rate: 50,
                tackle_rate: 50,
                shot_power: 50,
                shot_accuracy: 50,
                start_position: "Y22:Z24".to_string(),
                attack_position: "Y20:Z22".to_string(),
                defence_position: "Y24:Z26".to_string(),
                script: "function make_decision() return {} end".to_string(),
            },
        ],
        core_preamble_path: None,
        stdlib_preamble_path: None,
        team_a_preamble_path: None,
        team_b_preamble_path: None,
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
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "C3:D4".to_string(),
        attack_position: "C1:D2".to_string(),
        defence_position: "C5:D6".to_string(),
        script: "function make_decision() return {} end".to_string(),
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
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "INVALID".to_string(), // Invalid notation
        attack_position: "INVALID".to_string(),
        defence_position: "INVALID".to_string(),
        script: "function make_decision() return {} end".to_string(),
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
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "A1:AA50".to_string(), // Out of bounds
        attack_position: "A1:AA50".to_string(),
        defence_position: "A1:AA50".to_string(),
        script: "function make_decision() return {} end".to_string(),
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
                reaction_rate: 50,
                speed_rate: 50,
                tackle_rate: 50,
                shot_power: 50,
                shot_accuracy: 50,
                start_position: "A22:B24".to_string(),
                attack_position: "A20:B22".to_string(),
                defence_position: "A24:B26".to_string(),
                script: "function make_decision() return {} end".to_string(),
            },
            PlayerConfig {
                team: "B".to_string(),
                number: 9,
                name: "Forward".to_string(),
                reaction_rate: 50,
                speed_rate: 50,
                tackle_rate: 50,
                shot_power: 50,
                shot_accuracy: 50,
                start_position: "Y22:Z24".to_string(),
                attack_position: "Y20:Z22".to_string(),
                defence_position: "Y24:Z26".to_string(),
                script: "function make_decision() return {} end".to_string(),
            },
        ],
        core_preamble_path: None,
        stdlib_preamble_path: None,
        team_a_preamble_path: None,
        team_b_preamble_path: None,
    };

    // Convert to GameConfig
    let game_config = serializable_config
        .to_game_config(field.clone(), None)
        .unwrap();
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
        players: vec![PlayerConfig {
            team: "X".to_string(), // Invalid team
            number: 1,
            name: "Bad Player".to_string(),
            reaction_rate: 50,
            speed_rate: 50,
            tackle_rate: 50,
            shot_power: 50,
            shot_accuracy: 50,
            start_position: "A1:B2".to_string(),
            attack_position: "A1:B2".to_string(),
            defence_position: "A1:B2".to_string(),
            script: "function make_decision() return {} end".to_string(),
        }],
        core_preamble_path: None,
        stdlib_preamble_path: None,
        team_a_preamble_path: None,
        team_b_preamble_path: None,
    };

    let result = config.to_game_config(field, None);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Invalid team"));
}

#[test]
fn test_team_b_orientation_flip() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    // Team B player at M42 in their own orientation
    let config_b = PlayerConfig {
        team: "B".to_string(),
        number: 1,
        name: "Goalkeeper B".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "M42".to_string(), // Own orientation: near their goal
        attack_position: "M40".to_string(), // 2 rows forward (towards opponent goal)
        defence_position: "M44".to_string(), // 2 rows backward (towards own goal)
        script: "function make_decision() return {} end".to_string(),
    };

    let player_def_b = config_b.to_player_def(grid_dims).unwrap();

    // After flip, should be at column 14, row 3 in absolute coordinates
    // M42 (col=13, row=42) flips to (col=26-13+1=14, row=44-42+1=3)
    let start_region = player_def_b.regions.get("start position").unwrap();
    assert_eq!(start_region.top_left.col, 14); // Flipped column: N
    assert_eq!(start_region.top_left.row, 3); // 44 - 42 + 1 = 3

    // Team A player at M42 stays at M42 (no flip)
    let config_a = PlayerConfig {
        team: "A".to_string(),
        number: 1,
        name: "Goalkeeper A".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "M42".to_string(),
        attack_position: "M40".to_string(),
        defence_position: "M44".to_string(),
        script: "function make_decision() return {} end".to_string(),
    };

    let player_def_a = config_a.to_player_def(grid_dims).unwrap();
    let start_region_a = player_def_a.regions.get("start position").unwrap();
    assert_eq!(start_region_a.top_left.col, 13); // M = 13 (no flip)
    assert_eq!(start_region_a.top_left.row, 42); // No flip
}

#[test]
fn test_orientation_roundtrip() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    // Original config for Team B in their own orientation
    let original_config = PlayerConfig {
        team: "B".to_string(),
        number: 10,
        name: "Test Player B".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "M25".to_string(), // Own orientation
        attack_position: "M23".to_string(),
        defence_position: "M27".to_string(),
        script: "function make_decision() return {} end".to_string(),
    };

    // Convert to PlayerDef (flips to absolute coordinates)
    let player_def = original_config.to_player_def(grid_dims).unwrap();

    // Convert back to config (flips back to own orientation)
    let config_back = PlayerConfig::from_player_def(&player_def).unwrap();

    // Should match original (note: single cell "M25" becomes "M25:M25" after round-trip)
    assert_eq!(config_back.team, "B");
    // Both "M25" and "M25:M25" are valid representations of the same single cell
    let parsed_back =
        Region::from_grid_notation(&config_back.start_position, grid_dims).unwrap();
    let parsed_original =
        Region::from_grid_notation(&original_config.start_position, grid_dims).unwrap();
    assert_eq!(parsed_back, parsed_original);
}

#[test]
fn test_load_from_default_config_file() {
    use std::path::Path;

    // Test runs from workspace root, so config/ path should work
    // But let's be more robust and try multiple possible paths
    let possible_paths = ["config/default_game.toml", "../config/default_game.toml"];

    let config_path = possible_paths
        .iter()
        .map(Path::new)
        .find(|p| p.exists())
        .expect("Could not find config/default_game.toml");

    let config =
        SerializableGameConfig::from_file(config_path).expect("Failed to load default config file");

    // Verify we have 22 players (11 per team)
    assert_eq!(config.players.len(), 22, "Should have 22 players");

    // Verify all players have scripts
    for player in &config.players {
        assert!(
            !player.script.is_empty(),
            "Player {} should have a script",
            player.name
        );
    }

    // Count players per team
    let team_a_count = config.players.iter().filter(|p| p.team == "A").count();
    let team_b_count = config.players.iter().filter(|p| p.team == "B").count();
    assert_eq!(team_a_count, 11, "Team A should have 11 players");
    assert_eq!(team_b_count, 11, "Team B should have 11 players");

    // Verify all players have valid reaction_rate and speed_rate
    for player in &config.players {
        assert!(
            player.reaction_rate >= 10 && player.reaction_rate <= 100,
            "Player {} has invalid reaction_rate: {}",
            player.name,
            player.reaction_rate
        );
        assert!(
            player.speed_rate >= 10 && player.speed_rate <= 100,
            "Player {} has invalid speed_rate: {}",
            player.name,
            player.speed_rate
        );
    }

    // Verify the config can be converted to GameConfig
    let field = crate::field::Field::from_meters(60.0, 100.0, 26, 44);

    // Resolve paths relative to config file directory
    let config_dir = config_path.parent();
    let game_config = config
        .to_game_config(field, config_dir)
        .expect("Should convert to GameConfig");

    assert_eq!(game_config.players.len(), 22);

    // Verify reaction_rate, speed_rate, and scripts are preserved in PlayerDef
    for player_def in &game_config.players {
        assert!(
            player_def.reaction_rate >= 10 && player_def.reaction_rate <= 100,
            "PlayerDef {} has invalid reaction_rate: {}",
            player_def.name,
            player_def.reaction_rate
        );
        assert!(
            player_def.speed_rate >= 10 && player_def.speed_rate <= 100,
            "PlayerDef {} has invalid speed_rate: {}",
            player_def.name,
            player_def.speed_rate
        );
        assert!(
            !player_def.script.is_empty(),
            "PlayerDef {} should have a script",
            player_def.name
        );
    }
}

#[test]
fn test_player_script_field_mandatory() {
    // Test that script field is mandatory and properly serialized/deserialized
    let config_with_custom_script = PlayerConfig {
        team: "A".to_string(),
        number: 1,
        name: "Test Player".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "M42".to_string(),
        attack_position: "M40".to_string(),
        defence_position: "M44".to_string(),
        script: "function make_decision()\n    return { action = \"custom\" }\nend".to_string(),
    };

    let config_with_placeholder = PlayerConfig {
        team: "A".to_string(),
        number: 2,
        name: "Test Player 2".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 50,
        shot_accuracy: 50,
        start_position: "M42".to_string(),
        attack_position: "M40".to_string(),
        defence_position: "M44".to_string(),
        script: "function make_decision() return {} end".to_string(),
    };

    let serializable = SerializableGameConfig {
        players: vec![
            config_with_custom_script.clone(),
            config_with_placeholder.clone(),
        ],
        core_preamble_path: None,
        stdlib_preamble_path: None,
        team_a_preamble_path: None,
        team_b_preamble_path: None,
    };

    // Serialize to TOML
    let toml_str = serializable.to_toml().unwrap();

    // Should contain scripts for both players
    assert!(toml_str.contains("function make_decision"));

    // Deserialize back
    let parsed = SerializableGameConfig::from_toml(&toml_str).unwrap();
    assert_eq!(parsed.players.len(), 2);

    // Both players must have scripts (mandatory field)
    assert!(
        !parsed.players[0].script.is_empty(),
        "Player 1 must have a script"
    );
    assert!(
        !parsed.players[1].script.is_empty(),
        "Player 2 must have a script"
    );

    // Check custom script content
    assert!(
        parsed.players[0].script.contains("custom"),
        "Player 1 should have custom script"
    );
    assert!(
        parsed.players[1].script.contains("function make_decision"),
        "Player 2 should have a valid script"
    );
}

#[test]
fn test_shot_characteristics_serialization() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    // Create config with specific shot characteristics
    let config = PlayerConfig {
        team: "A".to_string(),
        number: 10,
        name: "Striker".to_string(),
        reaction_rate: 50,
        speed_rate: 50,
        tackle_rate: 50,
        shot_power: 85,
        shot_accuracy: 75,
        start_position: "M25".to_string(),
        attack_position: "M23".to_string(),
        defence_position: "M27".to_string(),
        script: "function make_decision() return {} end".to_string(),
    };

    // Convert to PlayerDef
    let player_def = config.to_player_def(grid_dims).unwrap();
    assert_eq!(player_def.shot_power, 85);
    assert_eq!(player_def.shot_accuracy, 75);

    // Convert back to PlayerConfig
    let config_back = PlayerConfig::from_player_def(&player_def).unwrap();
    assert_eq!(config_back.shot_power, 85);
    assert_eq!(config_back.shot_accuracy, 75);
}

#[test]
fn test_shot_characteristics_toml_roundtrip() {
    let config = SerializableGameConfig {
        players: vec![PlayerConfig {
            team: "A".to_string(),
            number: 9,
            name: "Forward".to_string(),
            reaction_rate: 60,
            speed_rate: 70,
            tackle_rate: 40,
            shot_power: 90,
            shot_accuracy: 80,
            start_position: "M20".to_string(),
            attack_position: "M18".to_string(),
            defence_position: "M22".to_string(),
            script: "function make_decision() return {} end".to_string(),
        }],
        core_preamble_path: None,
        stdlib_preamble_path: None,
        team_a_preamble_path: None,
        team_b_preamble_path: None,
    };

    // Serialize to TOML
    let toml_str = config.to_toml().unwrap();
    assert!(toml_str.contains("shot_power = 90"));
    assert!(toml_str.contains("shot_accuracy = 80"));

    // Deserialize back
    let parsed = SerializableGameConfig::from_toml(&toml_str).unwrap();
    assert_eq!(parsed.players[0].shot_power, 90);
    assert_eq!(parsed.players[0].shot_accuracy, 80);
}

#[test]
fn test_shot_characteristics_defaults() {
    // Test that missing shot_power and shot_accuracy get default values
    let toml_without_shot_fields = r#"
        [[players]]
        team = "A"
        number = 1
        name = "Test Player"
        reaction_rate = 50
        speed_rate = 50
        start_position = "M42"
        attack_position = "M40"
        defence_position = "M44"
        script = "function make_decision() return {} end"
    "#;

    let result = SerializableGameConfig::from_toml(toml_without_shot_fields).unwrap();
    assert_eq!(
        result.players[0].shot_power, 50,
        "Default shot_power should be 50"
    );
    assert_eq!(
        result.players[0].shot_accuracy, 50,
        "Default shot_accuracy should be 50"
    );
}

#[test]
fn test_player_config_missing_script() {
    // Test that config without script field fails to deserialize
    let toml_without_script = r#"
        [[players]]
        team = "A"
        number = 1
        name = "Test Player"
        reaction_rate = 50
        speed_rate = 50
        start_position = "M42"
    "#;

    let result = SerializableGameConfig::from_toml(toml_without_script);
    assert!(
        result.is_err(),
        "Config without script field should fail to load"
    );
}
