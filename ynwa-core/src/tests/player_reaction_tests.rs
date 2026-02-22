use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, GameStage, PlayerDef, RefereeDef, Decision, DecisionTarget};
use crate::region::GridCell;
use crate::team::Team;

fn create_test_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(1, 1).unwrap()).unwrap();

    let players = vec![
        PlayerDef::new(
            Team::A,
            1,
            "Player 1".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region.clone(),
        )
        .with_reaction_rate(100),
        PlayerDef::new(
            Team::A,
            2,
            "Player 2".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region.clone(),
        )
        .with_reaction_rate(55),
        PlayerDef::new(
            Team::A,
            3,
            "Player 3".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region.clone(),
        )
        .with_reaction_rate(10),
    ];

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, crate::game::GameStage::Play)
}

#[test]
fn test_reaction_interval() {
    assert!((PlayerReactionSystem::reaction_interval(100) - 0.5).abs() < 0.01);
    assert!((PlayerReactionSystem::reaction_interval(10) - 3.0).abs() < 0.01);
    assert!((PlayerReactionSystem::reaction_interval(55) - 1.75).abs() < 0.01);
}

#[test]
fn test_reaction_interval_clamping() {
    assert!((PlayerReactionSystem::reaction_interval(0) - 3.0).abs() < 0.01);
    assert!((PlayerReactionSystem::reaction_interval(150) - 0.5).abs() < 0.01);
}

#[test]
fn test_update_sets_needs_decision_flag() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    assert!(game.state.player_states[0].needs_decision);
    assert!(game.state.player_states[1].needs_decision);
    assert!(game.state.player_states[2].needs_decision);

    for player_state in &mut game.state.player_states {
        player_state.needs_decision = false;
    }

    system.update(&mut game, 0.5);
    assert!(game.state.player_states[0].needs_decision);
    assert!(!game.state.player_states[1].needs_decision);
    assert!(!game.state.player_states[2].needs_decision);
}

#[test]
fn test_update_respects_different_reaction_rates() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    for player_state in &mut game.state.player_states {
        player_state.needs_decision = false;
    }

    system.update(&mut game, 2.0);

    assert!(game.state.player_states[0].needs_decision);
    assert!(game.state.player_states[1].needs_decision);
    assert!(!game.state.player_states[2].needs_decision);
}

#[test]
fn test_update_uses_last_decision_time() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    game.state.player_states[0].last_decision_time = 1.0;
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 1.4);
    assert!(!game.state.player_states[0].needs_decision);

    system.update(&mut game, 1.6);
    assert!(game.state.player_states[0].needs_decision);
}

#[test]
fn test_update_does_not_clear_needs_decision_flag() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    game.state.player_states[0].needs_decision = true;
    game.state.player_states[0].last_decision_time = 0.0;

    system.update(&mut game, 0.6);
    assert!(game.state.player_states[0].needs_decision);
}

fn make_setup_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(1, 1).unwrap())
        .unwrap();

    let players = vec![PlayerDef::new(
        Team::A,
        1,
        "Player 1".to_string(),
        "function make_decision() return {} end".to_string(),
        start_region,
    )
    .with_reaction_rate(100)];

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Setup("start".to_string()))
}

#[test]
fn test_setup_sets_needs_decision_when_no_current_decision() {
    // Player has no decision yet → must request one
    let mut game = make_setup_game();
    let mut system = PlayerReactionSystem::new();

    game.state.player_states[0].current_decision = None;
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 0.0);

    assert!(game.state.player_states[0].needs_decision);
}

#[test]
fn test_setup_does_not_set_needs_decision_when_decision_exists() {
    // Player already has a decision (moving to position) → do not re-request
    let mut game = make_setup_game();
    let mut system = PlayerReactionSystem::new();

    let cell = GridCell::new(1, 1).unwrap();
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::GridCell(cell)));
    game.state.player_states[0].needs_decision = false;

    system.update(&mut game, 0.0);

    assert!(!game.state.player_states[0].needs_decision);
}

#[test]
fn test_setup_does_not_reset_needs_decision_already_true() {
    // needs_decision was already true (e.g. set by previous tick), must stay true
    let mut game = make_setup_game();
    let mut system = PlayerReactionSystem::new();

    game.state.player_states[0].current_decision = None;
    game.state.player_states[0].needs_decision = true;

    system.update(&mut game, 0.0);

    assert!(game.state.player_states[0].needs_decision);
}

#[test]
fn test_setup_ignores_reaction_rate_interval() {
    // Even a slow player (reaction_rate=10, interval=3s) must get a decision request
    // immediately during setup if they have no current decision.
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(1, 1).unwrap())
        .unwrap();

    let players = vec![PlayerDef::new(
        Team::A,
        1,
        "Slow Player".to_string(),
        "function make_decision() return {} end".to_string(),
        start_region,
    )
    .with_reaction_rate(10)]; // slowest rate: interval = 3.0s

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };
    let mut game = Game::with_stage(config, GameStage::Setup("start".to_string()));
    let mut system = PlayerReactionSystem::new();

    game.state.player_states[0].current_decision = None;
    game.state.player_states[0].needs_decision = false;
    game.state.player_states[0].last_decision_time = 0.0;

    // Tick at t=0.1 — far too early for Play-stage interval, but Setup must fire anyway
    system.update(&mut game, 0.1);

    assert!(game.state.player_states[0].needs_decision);
}
