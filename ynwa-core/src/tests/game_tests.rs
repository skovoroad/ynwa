use super::*;
use crate::region::GridCell;

fn create_test_config() -> GameConfig {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region_a1 = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
    .unwrap();

    let start_region_a2 = grid_dims.create_region(GridCell::new(3, 3).unwrap(), GridCell::new(4, 4).unwrap())
    .unwrap();

    let start_region_b = grid_dims.create_region(GridCell::new(20, 20).unwrap(), GridCell::new(21, 21).unwrap())
    .unwrap();

    let attack_region_a1 = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
    .unwrap();

    let attack_region_a2 = grid_dims.create_region(GridCell::new(3, 3).unwrap(), GridCell::new(4, 4).unwrap())
    .unwrap();

    let attack_region_b = grid_dims.create_region(GridCell::new(20, 20).unwrap(), GridCell::new(21, 21).unwrap())
    .unwrap();

    let defence_region_a1 = grid_dims.create_region(GridCell::new(1, 3).unwrap(), GridCell::new(2, 4).unwrap())
    .unwrap();

    let defence_region_a2 = grid_dims.create_region(GridCell::new(3, 5).unwrap(), GridCell::new(4, 6).unwrap())
    .unwrap();

    let defence_region_b = grid_dims.create_region(GridCell::new(20, 22).unwrap(), GridCell::new(21, 23).unwrap())
    .unwrap();

    GameConfig {
        field,
        players: vec![
            PlayerDef::new(
                Team::A,
                1,
                "Player A1".to_string(),
                "function make_decision() return {} end".to_string(),
                start_region_a1,
            )
            .with_attack_position(attack_region_a1)
            .with_defence_position(defence_region_a1),
            PlayerDef::new(
                Team::A,
                2,
                "Player A2".to_string(),
                "function make_decision() return {} end".to_string(),
                start_region_a2,
            )
            .with_attack_position(attack_region_a2)
            .with_defence_position(defence_region_a2),
            PlayerDef::new(
                Team::B,
                1,
                "Player B1".to_string(),
                "function make_decision() return {} end".to_string(),
                start_region_b,
            )
            .with_attack_position(attack_region_b)
            .with_defence_position(defence_region_b),
        ],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

#[test]
fn test_state_indices_match_config() {
    let config = create_test_config();
    let player_count = config.players.len();

    let game = Game::new(config);

    assert_eq!(game.state().player_states.len(), player_count);
}

#[test]
fn test_step_updates_time() {
    let config = create_test_config();
    let mut game = Game::new(config);

    game.step(0.016);
    assert!((game.state().elapsed_time - 0.016).abs() < 0.001);
}

#[test]
fn test_player_initial_position_from_start_region() {
    let config = create_test_config();
    let game = Game::with_stage(config, GameStage::Play);

    let cell_width =
        game.config().field.width().get::<meter>() / game.config().field.grid_columns() as f32;

    let expected_a1_x = 1.0 * cell_width;
    let expected_a1_z = 1.0 * cell_width;
    let expected_a2_x = 3.0 * cell_width;
    let expected_a2_z = 3.0 * cell_width;
    let expected_b1_x = 20.0 * cell_width;
    let expected_b1_z = 20.0 * cell_width;

    assert_eq!(game.state().player_states.len(), 3);

    let tolerance = 0.01;

    assert!(
        (game.state().player_states[0].position.x.get::<meter>() - expected_a1_x).abs() < tolerance
    );
    assert!(
        (game.state().player_states[0].position.z.get::<meter>() - expected_a1_z).abs() < tolerance
    );
    assert!(
        (game.state().player_states[1].position.x.get::<meter>() - expected_a2_x).abs() < tolerance
    );
    assert!(
        (game.state().player_states[1].position.z.get::<meter>() - expected_a2_z).abs() < tolerance
    );
    assert!(
        (game.state().player_states[2].position.x.get::<meter>() - expected_b1_x).abs() < tolerance
    );
    assert!(
        (game.state().player_states[2].position.z.get::<meter>() - expected_b1_z).abs() < tolerance
    );

    for player_state in &game.state().player_states {
        assert_eq!(player_state.position.y.get::<meter>(), 0.0);
    }
}

#[test]
fn test_player_initial_position_in_setup_stage() {
    let config = create_test_config();
    let field_length = config.field.length().get::<meter>();
    let game = Game::with_stage(config, GameStage::Setup("start".to_string()));

    let expected_x = -5.0;
    let expected_z = field_length / 2.0;

    for player_state in &game.state().player_states {
        assert!(
            (player_state.position.x.get::<meter>() - expected_x).abs() < 0.01,
            "Setup X should be -5, got {}",
            player_state.position.x.get::<meter>()
        );
        assert!(
            (player_state.position.z.get::<meter>() - expected_z).abs() < 0.01,
            "Setup Z should be field_length/2, got {}",
            player_state.position.z.get::<meter>()
        );
    }
}
