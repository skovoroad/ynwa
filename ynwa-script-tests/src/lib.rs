// Test helpers for script testing
// This module is used by integration tests

use ynwa_core::field::Field;
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef};
use ynwa_core::region::GridCell;
use ynwa_core::team::Team;

/// Create a simple test game with one player using the given script
pub fn create_test_game_with_script(script: &str) -> Game {
    create_test_game_with_script_and_stage(script, GameStage::Play)
}

/// Create a simple test game with one player using the given script and specific stage
pub fn create_test_game_with_script_and_stage(script: &str, stage: GameStage) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(10, 10).unwrap(), GridCell::new(11, 11).unwrap())
    .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, stage)
}

/// Sets `needs_decision = true` for all players, bypassing PlayerReactionSystem timing logic.
pub fn request_decisions_for_all(game: &mut ynwa_core::game::Game) {
    for state in game.state.player_states.iter_mut() {
        state.needs_decision = true;
    }
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

    let start_region = grid_dims.create_region(GridCell::new(10, 10).unwrap(), GridCell::new(11, 11).unwrap())
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

/// Create a test game with core, stdlib, and team A preambles loaded, at a specific stage
pub fn create_test_game_with_full_preambles_and_stage(script: &str, stage: GameStage) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");
    let core_path = workspace_root.join("ynwa-scripts/preambles/core.lua");
    let stdlib_path = workspace_root.join("ynwa-scripts/preambles/stdlib.lua");
    let team_a_path = workspace_root.join("ynwa-scripts/team-libs/team_a.lua");

    let core_preamble = std::fs::read_to_string(&core_path)
        .unwrap_or_else(|e| panic!("Failed to load core preamble: {}", e));
    let stdlib_preamble = std::fs::read_to_string(&stdlib_path)
        .unwrap_or_else(|e| panic!("Failed to load stdlib preamble: {}", e));
    let team_a_preamble = std::fs::read_to_string(&team_a_path)
        .unwrap_or_else(|e| panic!("Failed to load team_a preamble: {}", e));

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            script.to_string(),
            start_region,
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig {
            core_preamble,
            stdlib_preamble,
            team_a_preamble,
            team_b_preamble: String::new(),
        },
    };

    Game::with_stage(config, stage)
}

/// Create a test game using a full football field (with all zones including goals)
/// and core + stdlib preambles. Required for tests that access GAME_DATA.zones.goal_*.
pub fn create_test_game_football_field_with_preambles(script: &str) -> Game {
    let field = ynwa_football::field_builder::create_football_field();
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");
    let core_path = workspace_root.join("ynwa-scripts/preambles/core.lua");
    let stdlib_path = workspace_root.join("ynwa-scripts/preambles/stdlib.lua");

    let core_preamble = std::fs::read_to_string(&core_path)
        .unwrap_or_else(|e| panic!("Failed to load core preamble: {}", e));
    let stdlib_preamble = std::fs::read_to_string(&stdlib_path)
        .unwrap_or_else(|e| panic!("Failed to load stdlib preamble: {}", e));

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
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

/// Create a test game with core, stdlib, and both team preambles loaded,
/// using a football field with goals. Players are provided by the caller.
pub fn create_test_game_with_all_preambles(players: Vec<PlayerDef>) -> Game {
    let field = ynwa_football::field_builder::create_football_field();

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .expect("Failed to get workspace root");

    let load = |rel: &str| -> String {
        std::fs::read_to_string(workspace_root.join(rel))
            .unwrap_or_else(|e| panic!("Failed to load {}: {}", rel, e))
    };

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig {
            core_preamble:   load("ynwa-scripts/preambles/core.lua"),
            stdlib_preamble: load("ynwa-scripts/preambles/stdlib.lua"),
            team_a_preamble: load("ynwa-scripts/team-libs/team_a.lua"),
            team_b_preamble: load("ynwa-scripts/team-libs/team_b.lua"),
        },
    };

    Game::with_stage(config, GameStage::Play)
}
