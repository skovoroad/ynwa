use crate::game_manager::FootballGameManager;
use ynwa_core::field::zones::{Point3D, Rectangle, ZoneGeometry};
use ynwa_core::field::{Field, FieldBuilder, Zone};
use ynwa_core::game::{BallDef, Decision, Game, GameConfig, GameStage, PlayerDef, RefereeDef};
use ynwa_core::region::{GridCell, Region};
use ynwa_core::system::System;
use ynwa_core::team::Team;
use uom::si::f32::Length;
use uom::si::length::meter;
use std::collections::HashMap;

fn start_regions(r: Region) -> HashMap<String, Region> {
    HashMap::from([("start position".to_string(), r)])
}

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
                start_regions(start_region.clone()),
            ),
            PlayerDef::new(
                Team::A,
                2,
                "Test Player 2".to_string(),
                "function make_decision() return {} end".to_string(),
                start_regions(start_region),
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

    // Players have no current_decision — should not be marked ready
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
    assert!(!game.state.player_states[0].is_ready);
    assert!(!game.state.player_states[1].is_ready);
}

#[test]
fn test_check_player_readiness_when_in_region() {
    let mut game = create_test_game_setup();
    let mut manager = FootballGameManager::new();

    game.state.player_states[0].current_decision = Some(Decision::Stop);

    manager.update(&mut game, 0.0);

    assert!(game.state.player_states[0].is_ready);
    assert!(!game.state.player_states[1].is_ready);
    assert_eq!(game.state.stage, GameStage::Setup("Prepare".to_string()));
}

#[test]
fn test_transition_to_play_when_all_ready() {
    let mut game = create_test_game_setup();
    let mut manager = FootballGameManager::new();

    for player_state in game.state.player_states.iter_mut() {
        player_state.current_decision = Some(Decision::Stop);
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
            start_regions(start_region),
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
            start_regions(start_region.clone()),
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
            assert_eq!(reason, "kick off", "Should transition to kick off setup");
        }
        _ => panic!("Expected Setup stage after goal, got {:?}", game.state.stage),
    }

    assert!(!game.state.player_states[0].is_ready, "Player should not be ready initially");

    game.state.player_states[0].current_decision = Some(Decision::Stop);

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
            start_regions(start_region),
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

    game.state.stage = GameStage::Setup("kick off".to_string());

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

    game.state.stage = GameStage::Setup("kick off".to_string());

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
            start_regions(start_region),
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

/// Standard football field (production orientation) with one player per team.
/// Team A goal: z < 0, Team B goal: z > field_length.
fn create_standard_game() -> Game {
    use crate::field_builder::create_football_field;

    let field = create_football_field();
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(10, 10).unwrap(), GridCell::new(11, 11).unwrap())
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![
            PlayerDef::new(
                Team::A, 1, "A".to_string(),
                "function make_decision() return {} end".to_string(),
                start_regions(start_region.clone()),
            ),
            PlayerDef::new(
                Team::B, 1, "B".to_string(),
                "function make_decision() return {} end".to_string(),
                start_regions(start_region),
            ),
        ],
        ball: BallDef::default(),
        referees: vec![],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play)
}

#[test]
fn test_team_stats_initialized_for_all_teams() {
    let game = create_standard_game();
    assert_eq!(game.state.team_stats[&Team::A].get("score"), 0.0);
    assert_eq!(game.state.team_stats[&Team::B].get("score"), 0.0);
}

#[test]
fn test_goal_increments_score() {
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    // Ball inside Team A goal (z = -0.5, centered on X) — Team B scores
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(field_width / 2.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(-0.5),
    );
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.team_stats[&Team::B].get("score"), 1.0);
    assert_eq!(game.state.team_stats[&Team::A].get("score"), 0.0);

    // Second goal in the same net
    game.state.stage = GameStage::Play;
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(field_width / 2.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(-0.5),
    );
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.team_stats[&Team::B].get("score"), 2.0);
}

#[test]
fn test_goal_scored_in_standard_field_credits_correct_team() {
    // Ball in Team B's goal (z > field_length) → Team A scores.
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_length = game.config().field.length().get::<meter>();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(field_width / 2.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(field_length + 0.5),
    );
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.team_stats[&Team::A].get("score"), 1.0, "Team A should score when ball enters Team B's goal");
    assert_eq!(game.state.team_stats[&Team::B].get("score"), 0.0);
}

#[test]
fn test_goal_in_team_a_net_scores_for_team_b() {
    // Ball in Team A's goal (z < 0) → Team B scores.
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(field_width / 2.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(-0.5),
    );
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.team_stats[&Team::B].get("score"), 1.0, "Team B should score when ball enters Team A's goal");
    assert_eq!(game.state.team_stats[&Team::A].get("score"), 0.0);
}

// --- restart_position / restart_team from handle_event ---

fn set_ball_pos(game: &mut Game, x: f32, z: f32) {
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(x),
        Length::new::<meter>(0.0),
        Length::new::<meter>(z),
    );
}

#[test]
fn test_touchline_sets_restart_position_and_team() {
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.last_possessing_team = Some(Team::A);
    set_ball_pos(&mut game, -(0.12 + 0.01), 30.0); // over left sideline

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("throw_in".to_string()));
    let rp = game.state.restart_position.expect("restart_position must be set");
    assert!((rp.z.get::<meter>() - 30.0).abs() < 0.01);
    assert_eq!(game.state.restart_team, Some(Team::B)); // opposite of last_team A
    let _ = field_width;
}

#[test]
fn test_goal_line_attacking_team_gives_goal_kick() {
    // Team B attacked toward z=0 (Team A's goal) and last touched — goal kick for Team A
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.last_possessing_team = Some(Team::B);
    // Ball outside goalposts near z=0
    set_ball_pos(&mut game, 0.5, -(0.12 + 0.01));

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("goal_kick".to_string()));
    let rp = game.state.restart_position.expect("restart_position must be set");
    assert!((rp.x.get::<meter>() - field_width / 2.0).abs() < 0.01);
    assert!((rp.z.get::<meter>() - 5.5).abs() < 0.01);
    assert_eq!(game.state.restart_team, Some(Team::A));
}

#[test]
fn test_goal_line_defending_team_gives_corner() {
    // Team A (defending) last touched past z=0 — corner for Team B (attacking)
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.last_possessing_team = Some(Team::A);
    set_ball_pos(&mut game, 0.5, -(0.12 + 0.01));

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("corner".to_string()));
    let rp = game.state.restart_position.expect("restart_position must be set");
    // Nearest corner to (0.5, ~0) is (0, 0)
    assert!((rp.x.get::<meter>()).abs() < 0.01);
    assert!((rp.z.get::<meter>()).abs() < 0.01);
    assert_eq!(game.state.restart_team, Some(Team::B)); // attacking team takes corner
    let _ = field_width;
}

#[test]
fn test_goal_line_far_end_attacking_team_gives_goal_kick() {
    // Team A attacked toward z=field_length and last touched — goal kick for Team B
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_length = game.config().field.length().get::<meter>();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.last_possessing_team = Some(Team::A);
    set_ball_pos(&mut game, field_width - 0.5, field_length + 0.12 + 0.01);

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("goal_kick".to_string()));
    let rp = game.state.restart_position.expect("restart_position must be set");
    assert!((rp.x.get::<meter>() - field_width / 2.0).abs() < 0.01);
    assert!((rp.z.get::<meter>() - (field_length - 5.5)).abs() < 0.01);
    assert_eq!(game.state.restart_team, Some(Team::B));
}

#[test]
fn test_goal_line_far_end_defending_team_gives_corner() {
    // Team B (defending) last touched past z=field_length — corner for Team A (attacking)
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_length = game.config().field.length().get::<meter>();
    let field_width = game.config().field.width().get::<meter>();

    game.state.ball_state.last_possessing_team = Some(Team::B);
    set_ball_pos(&mut game, field_width - 0.5, field_length + 0.12 + 0.01);

    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("corner".to_string()));
    let rp = game.state.restart_position.expect("restart_position must be set");
    // Nearest corner to (field_width - 0.5, ~field_length) is (field_width, field_length)
    assert!((rp.x.get::<meter>() - field_width).abs() < 0.01);
    assert!((rp.z.get::<meter>() - field_length).abs() < 0.01);
    assert_eq!(game.state.restart_team, Some(Team::A)); // attacking team takes corner
}

#[test]
fn test_setup_tick_places_ball_at_restart_position() {
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();

    game.state.ball_state.last_possessing_team = Some(Team::A);
    let field_width = game.config().field.width().get::<meter>();
    set_ball_pos(&mut game, 0.5, -(0.12 + 0.01)); // corner scenario: defending Team A last touched

    manager.update(&mut game, 0.0); // Play tick → corner Setup

    // Manually move ball away to verify Setup tick restores restart_position
    set_ball_pos(&mut game, 99.0, 99.0);
    manager.update(&mut game, 0.0); // Setup tick

    let ball_x = game.state.ball_state.position.x.get::<meter>();
    let ball_z = game.state.ball_state.position.z.get::<meter>();
    // Corner (0, 0) was set as restart_position
    assert!(ball_x.abs() < 0.01, "ball x should be at corner: {ball_x}");
    assert!(ball_z.abs() < 0.01, "ball z should be at corner: {ball_z}");
    let _ = field_width;
}

#[test]
fn test_kick_off_has_no_restart_position() {
    let mut game = create_standard_game();
    let mut manager = FootballGameManager::new();
    let field_width = game.config().field.width().get::<meter>();

    set_ball_pos(&mut game, field_width / 2.0, -0.5);
    manager.update(&mut game, 0.0);

    assert_eq!(game.state.stage, GameStage::Setup("kick off".to_string()));
    assert!(game.state.restart_position.is_none());
}
