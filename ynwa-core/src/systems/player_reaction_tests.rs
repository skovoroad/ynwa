use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
use crate::region::{GridCell, Region};
use crate::team::Team;

fn create_test_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();

    let start_region = Region::new(
        Team::A,
        GridCell::new(1, 1).unwrap(),
        GridCell::new(1, 1).unwrap(),
        grid_dims,
    )
    .unwrap();

    let players = vec![
        PlayerDef::new(Team::A, 1, "Player 1".to_string(), "function make_decision() return {} end".to_string(), start_region.clone(),
        )
        .with_reaction_rate(100),
        PlayerDef::new(Team::A, 2, "Player 2".to_string(), "function make_decision() return {} end".to_string(), start_region.clone(),
        )
        .with_reaction_rate(55),
        PlayerDef::new(Team::A, 3, "Player 3".to_string(), "function make_decision() return {} end".to_string(), start_region.clone(),
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

#[test]
fn test_setup_stage_uses_max_frequency() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    game.state.stage = crate::game::GameStage::Setup("kickoff".to_string());

    for player_state in &mut game.state.player_states {
        player_state.needs_decision = false;
        player_state.last_decision_time = 0.0;
    }

    system.update(&mut game, 0.5);

    assert!(game.state.player_states[0].needs_decision);
    assert!(game.state.player_states[1].needs_decision);
    assert!(game.state.player_states[2].needs_decision);
}

#[test]
fn test_play_stage_respects_individual_rates() {
    let mut game = create_test_game();
    let mut system = PlayerReactionSystem::new();

    game.state.stage = crate::game::GameStage::Play;

    for player_state in &mut game.state.player_states {
        player_state.needs_decision = false;
        player_state.last_decision_time = 0.0;
    }

    system.update(&mut game, 0.5);

    assert!(game.state.player_states[0].needs_decision);
    assert!(!game.state.player_states[1].needs_decision);
    assert!(!game.state.player_states[2].needs_decision);
}
