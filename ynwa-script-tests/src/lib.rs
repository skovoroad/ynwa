// Test helpers for script testing
// This module is used by integration tests

use ynwa_core::field::{Field, FieldBuilder, Zone};
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef};
use ynwa_core::region::{GridCell, Region};
use ynwa_core::team::Team;

/// Create a simple test game with one player using the given script
pub fn create_test_game_with_script(script: &str) -> Game {
    create_test_game_with_script_and_stage(script, GameStage::Play)
}

/// Create a simple test game with one player using the given script and specific stage
pub fn create_test_game_with_script_and_stage(script: &str, stage: GameStage) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = Region::new(
        Team::A,
        GridCell::new(10, 10).unwrap(),
        GridCell::new(11, 11).unwrap(),
        grid_dims,
    )
    .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            50,
            50,
            50,
            50,
            50,
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, stage)
}

/// Load a test script from ynwa-scripts/test-scripts/
pub fn load_test_script(name: &str) -> String {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");
    let script_path = workspace_root.join(format!("ynwa-scripts/test-scripts/{}", name));
    std::fs::read_to_string(&script_path)
        .unwrap_or_else(|e| panic!("Failed to load test script {}: {}", name, e))
}

/// Create a test game with preambles loaded
pub fn create_test_game_with_preambles(script: &str) -> Game {
    create_test_game_with_preambles_and_stage(script, GameStage::Play)
}

/// Create a test game with preambles loaded and specific stage
pub fn create_test_game_with_preambles_and_stage(script: &str, stage: GameStage) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = Region::new(
        Team::A,
        GridCell::new(10, 10).unwrap(),
        GridCell::new(11, 11).unwrap(),
        grid_dims,
    )
    .unwrap();

    // Load preambles from files (CARGO_MANIFEST_DIR points to ynwa-script-tests)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");
    let core_path = workspace_root.join("ynwa-scripts/preambles/core.lua");
    let stdlib_path = workspace_root.join("ynwa-scripts/preambles/stdlib.lua");

    let core_preamble = std::fs::read_to_string(&core_path)
        .unwrap_or_else(|e| panic!("Failed to load core preamble from {:?}: {}", core_path, e));
    let stdlib_preamble = std::fs::read_to_string(&stdlib_path).unwrap_or_else(|e| {
        panic!(
            "Failed to load stdlib preamble from {:?}: {}",
            stdlib_path, e
        )
    });

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            50,
            50,
            50,
            50,
            50,
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig {
            core_preamble,
            stdlib_preamble,
            team_a_preamble: String::new(),
            team_b_preamble: String::new(),
        },
    };

    Game::with_stage(config, stage)
}

/// Create a test game with preambles and custom zones
pub fn create_test_game_with_preambles_and_zones(script: &str, zones: Vec<Zone>) -> Game {
    let mut builder = FieldBuilder::from_meters(100.0, 60.0, 26, 44);
    for zone in zones {
        builder = builder.with_zone(zone);
    }
    let field = builder.build();
    
    let grid_dims = field.grid_dimensions();
    let start_region = Region::new(
        Team::A,
        GridCell::new(10, 10).unwrap(),
        GridCell::new(11, 11).unwrap(),
        grid_dims,
    )
    .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");
    let core_path = workspace_root.join("ynwa-scripts/preambles/core.lua");
    let stdlib_path = workspace_root.join("ynwa-scripts/preambles/stdlib.lua");

    let core_preamble = std::fs::read_to_string(&core_path)
        .unwrap_or_else(|e| panic!("Failed to load core preamble from {:?}: {}", core_path, e));
    let stdlib_preamble = std::fs::read_to_string(&stdlib_path).unwrap_or_else(|e| {
        panic!(
            "Failed to load stdlib preamble from {:?}: {}",
            stdlib_path, e
        )
    });

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            50,
            50,
            50,
            50,
            50,
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig {
            core_preamble,
            stdlib_preamble,
            team_a_preamble: String::new(),
            team_b_preamble: String::new(),
        },
    };

    Game::with_stage(config, GameStage::Play)
}
