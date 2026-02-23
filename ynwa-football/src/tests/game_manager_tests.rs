use crate::game_manager::FootballGameManager;
use ynwa_core::field::zones::{Point3D, Rectangle, ZoneGeometry};
use ynwa_core::field::{Field, FieldBuilder, Zone};
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef};
use ynwa_core::region::GridCell;
use ynwa_core::system::System;
use ynwa_core::team::Team;
use uom::si::f32::Length;
use uom::si::length::meter;

fn create_test_game_setup() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![
            PlayerDef::new(
                Team::A,
                1,
                "Test Player 1".to_string(),
                "function make_decision() return {} end".to_string(),
                start_region.clone(),
            ),
            PlayerDef::new(
                Team::A,
                2,
                "Test Player 2".to_string(),
                "function make_decision() return {} end".to_string(),
                start_region,
            ),
        ],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Setup("Prepare".to_string()))
}

#[test]
fn test_players_start_at_edge_in_setup() {
    let game = create_test_game_setup();

    let field_length = game.config().field.length().get::<meter>();
    let expected_x = -5.0;
    let expected_z = field_length / 2.0;

    for (idx, player_state) in game.state.player_states.iter().enumerate() {
        assert!(
            (player_state.position.x.get::<meter>() - expected_x).abs() < 0.01,
            "Player {} X: {} vs expected {}",
            idx,
            player_state.position.x.get::<meter>(),
            expected_x
        );
        assert!(
            (player_state.position.z.get::<meter>() - expected_z).abs() < 0.01,
            "Player {} Z: {} vs expected {}",
            idx,
            player_state.position.z.get::<meter>(),
            expected_z
        );
        assert!(!player_state.is_ready);
    }
}

#[test]
fn test_check_player_readiness_when_not_in_region() {
    let mut game = create_test_game_setup();
    let mut manager = FootballGameManager::new();

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
    assert!(!game.state.player_states[0].is_ready);
    assert!(!game.state.player_states[1].is_ready);
}

#[test]
fn test_check_player_readiness_when_in_region() {
    let mut game = create_test_game_setup();
    let mut manager = FootballGameManager::new();

    let start_region = game.config().players[0].regions["start position"].clone();
    let center = start_region.center(
        game.config().field.grid_dimensions(),
        game.config().field.width().get::<meter>(),
    );
    game.state.player_states[0].position = center;

    manager.update(&mut game, 0.0);

    assert!(game.state.player_states[0].is_ready);
    assert!(!game.state.player_states[1].is_ready);
    assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
}

#[test]
fn test_transition_to_play_when_all_ready() {
    let mut game = create_test_game_setup();
    let mut manager = FootballGameManager::new();

    let centers: Vec<_> = game
        .config()
        .players
        .iter()
        .map(|player_def| {
            let start_region = &player_def.regions["start position"];
            start_region.center(
                game.config().field.grid_dimensions(),
                game.config().field.width().get::<meter>(),
            )
        })
        .collect();

    for (idx, player_state) in game.state.player_states.iter_mut().enumerate() {
        player_state.position = centers[idx].clone();
    }

    manager.update(&mut game, 0.0);

    assert!(game.state.player_states[0].is_ready);
    assert!(game.state.player_states[1].is_ready);
    assert_eq!(game.state.stage, GameStage::Play);
}

#[test]
fn test_no_updates_in_play_stage() {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef {
            initial_position: Point3D::new(
                Length::new::<meter>(50.0),
                Length::new::<meter>(0.0),
                Length::new::<meter>(30.0),
            ),
        },
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Play);
    let mut manager = FootballGameManager::new();

    let initial_stage = game.state.stage.clone();
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, initial_stage);
}

#[test]
fn test_game_resumes_after_event_triggered_setup() {
    let field = FieldBuilder::from_meters(60.0, 100.0, 26, 44)
        .with_zone(Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.32, 0.0, 32.68)),
        ))
        .with_zone(Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(100.0, 27.32, 102.0, 32.68)),
        ))
        .build();

    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region.clone(),
        )],
        ball: BallDef {
            initial_position: Point3D::new(
                Length::new::<meter>(50.0),
                Length::new::<meter>(0.0),
                Length::new::<meter>(30.0),
            ),
        },
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Play);
    let mut manager = FootballGameManager::new();

    assert_eq!(game.state.stage, GameStage::Play);
    manager.update(&mut game, 0.0);
    assert_eq!(game.state.stage, GameStage::Play, "Should remain in Play");

    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-1.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    manager.update(&mut game, 0.0);
    match &game.state.stage {
        GameStage::Setup(reason) => {
            assert_eq!(reason, "after_goal", "Should transition to after_goal setup");
        }
        _ => panic!("Expected Setup stage after goal, got {:?}", game.state.stage),
    }

    assert!(!game.state.player_states[0].is_ready, "Player should not be ready initially");

    let center = start_region.center(
        game.config().field.grid_dimensions(),
        game.config().field.width().get::<meter>(),
    );
    game.state.player_states[0].position = center;

    manager.update(&mut game, 0.0);
    assert!(game.state.player_states[0].is_ready, "Player should be marked ready");

    assert_eq!(
        game.state.stage,
        GameStage::Play,
        "Game should resume Play immediately after all players ready"
    );
}

#[test]
fn test_ball_resets_to_initial_position_in_setup() {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let initial_ball_position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef {
            initial_position: initial_ball_position.clone(),
        },
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Play);

    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(10.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(5.0),
    );
    game.state.ball_state.velocity =
        ynwa_core::field::zones::Velocity3D::from_meters_per_second(2.0, 1.0, 3.0);

    assert_ne!(
        game.state.ball_state.position.x.get::<meter>(),
        initial_ball_position.x.get::<meter>()
    );

    game.state.stage = GameStage::Setup("after_goal".to_string());

    let mut manager = FootballGameManager::new();
    manager.update(&mut game, 0.0);

    assert_eq!(
        game.state.ball_state.position.x.get::<meter>(),
        initial_ball_position.x.get::<meter>(),
        "Ball X position should be reset"
    );
    assert_eq!(
        game.state.ball_state.position.y.get::<meter>(),
        initial_ball_position.y.get::<meter>(),
        "Ball Y position should be reset"
    );
    assert_eq!(
        game.state.ball_state.position.z.get::<meter>(),
        initial_ball_position.z.get::<meter>(),
        "Ball Z position should be reset"
    );

    use uom::si::velocity::meter_per_second;
    assert_eq!(game.state.ball_state.velocity.x.get::<meter_per_second>(), 0.0);
    assert_eq!(game.state.ball_state.velocity.y.get::<meter_per_second>(), 0.0);
    assert_eq!(game.state.ball_state.velocity.z.get::<meter_per_second>(), 0.0);
}

#[test]
fn test_ball_ownership_resets_in_setup_stage() {
    let mut game = create_test_game_setup();

    game.state.stage = GameStage::Play;
    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.last_possessing_team = Some(Team::A);

    assert_eq!(game.state.ball_state.possessed_by, Some(0));
    assert_eq!(game.state.ball_state.last_possessing_team, Some(Team::A));

    game.state.stage = GameStage::Setup("after_goal".to_string());

    let mut manager = FootballGameManager::new();
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.ball_state.possessed_by, None);
    assert_eq!(game.state.ball_state.last_possessing_team, None);
}

#[test]
fn test_handle_event_clears_decision_so_setup_position_is_requested() {
    let field = FieldBuilder::from_meters(60.0, 100.0, 26, 44)
        .with_zone(Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.32, 0.0, 32.68)),
        ))
        .build();

    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
        )
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        )],
        ball: BallDef {
            initial_position: Point3D::new(
                Length::new::<meter>(50.0),
                Length::new::<meter>(0.0),
                Length::new::<meter>(30.0),
            ),
        },
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    let mut game = Game::with_stage(config, GameStage::Play);
    let mut manager = FootballGameManager::new();

    game.state.player_states[0].current_decision = Some(ynwa_core::game::Decision::Stop);
    game.state.player_states[0].needs_decision = false;

    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-1.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    manager.update(&mut game, 0.0);

    assert!(
        matches!(game.state.stage, GameStage::Setup(_)),
        "Expected Setup stage"
    );
    assert!(
        game.state.player_states[0].current_decision.is_none(),
        "current_decision must be None after Play→Setup transition"
    );
    assert!(
        game.state.player_states[0].needs_decision,
        "needs_decision must be true after Play→Setup transition"
    );
}
