use super::*;
use crate::field::zones::{Rectangle, ZoneGeometry};
use crate::field::{FieldBuilder, Zone};
use crate::game::{BallDef, GameConfig, GameStage, PlayerDef, RefereeDef};
use crate::region::{GridCell, Region};
use uom::si::f32::Length;
use uom::si::length::meter;

fn create_test_game() -> Game {
    // Standard football field: length=100m (X axis), width=60m (Z axis)
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
    let start_region = Region::new(
        Team::A,
        GridCell::new(1, 1).unwrap(),
        GridCell::new(2, 2).unwrap(),
        grid_dims,
    )
    .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(Team::A, 1, "Test".to_string(), String::new(), start_region,
        )],
        ball: BallDef {
            initial_position: Point3D::new(
                Length::new::<meter>(50.0), // Center of field length
                Length::new::<meter>(0.0),
                Length::new::<meter>(30.0), // Center of field width
            ),
        },
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play)
}

#[test]
fn test_goal_team_a() {
    let mut game = create_test_game();
    // Place ball completely inside Team A goal
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-1.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal(&game);
    assert_eq!(event, Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_team_b() {
    let mut game = create_test_game();
    // Place ball completely inside Team B goal
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(101.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal(&game);
    assert_eq!(event, Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_ball_on_goal_line_not_a_goal() {
    let mut game = create_test_game();
    // Ball center on goal line, but not completely inside
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(0.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal(&game);
    assert_eq!(event, None);
}

#[test]
fn test_ball_partially_in_goal_not_a_goal() {
    let mut game = create_test_game();
    // Ball partially crosses goal line but not completely inside goal
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-0.05), // Only 5cm inside, ball radius is 11cm
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal(&game);
    assert_eq!(event, None);
}

#[test]
fn test_touchline_left() {
    let mut game = create_test_game();
    // Ball completely over left sideline (z < 0)
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(-0.12), // Beyond ball radius
    );

    let event = check_touchline(&game);
    assert!(matches!(event, Some(FootballEvent::Touchline(_))));
}

#[test]
fn test_touchline_right() {
    let mut game = create_test_game();
    // Field width is 60m, ball completely over right sideline
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(60.12), // Beyond ball radius (60 + 0.12 > 60 + 0.11)
    );

    let event = check_touchline(&game);
    assert!(matches!(event, Some(FootballEvent::Touchline(_))));
}

#[test]
fn test_ball_on_touchline_not_out() {
    let mut game = create_test_game();
    // Ball touching but not completely over touchline
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(0.11), // Center at ball radius from line - still in play
    );

    let event = check_touchline(&game);
    assert_eq!(event, None);
}

#[test]
fn test_goal_line_near() {
    let mut game = create_test_game();
    // Ball completely over near goal line (x < 0)
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-0.12),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal_line(&game);
    assert!(matches!(event, Some(FootballEvent::GoalLine(_))));
}

#[test]
fn test_goal_line_far() {
    let mut game = create_test_game();
    // Ball completely over far goal line (x > field_length)
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(100.12),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );

    let event = check_goal_line(&game);
    assert!(matches!(event, Some(FootballEvent::GoalLine(_))));
}

#[test]
fn test_game_end() {
    let mut game = create_test_game();
    game.state.elapsed_time = GAME_DURATION;

    let event = check_game_end(&game);
    assert_eq!(event, Some(FootballEvent::GameEnd));
}

#[test]
fn test_game_not_ended() {
    let mut game = create_test_game();
    game.state.elapsed_time = 59.9;

    let event = check_game_end(&game);
    assert_eq!(event, None);
}

#[test]
fn test_check_events_priority_goal() {
    let mut game = create_test_game();
    // Ball in goal AND time expired - goal has priority
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(-1.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );
    game.state.elapsed_time = GAME_DURATION;

    let event = check_events(&game);
    assert_eq!(event, Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_check_events_priority_game_end() {
    let mut game = create_test_game();
    // Time expired AND ball out - game end has priority
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(-0.12),
    );
    game.state.elapsed_time = GAME_DURATION;

    let event = check_events(&game);
    assert_eq!(event, Some(FootballEvent::GameEnd));
}

#[test]
fn test_check_events_no_event() {
    let mut game = create_test_game();
    // Ball in play, time not expired
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(50.0),
        Length::new::<meter>(0.0),
        Length::new::<meter>(30.0),
    );
    game.state.elapsed_time = 30.0;

    let event = check_events(&game);
    assert_eq!(event, None);
}
