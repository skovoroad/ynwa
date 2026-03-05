use crate::events::{
    check_events, check_game_end, check_goal, check_goal_line, check_touchline, FootballEvent,
    BALL_RADIUS, GAME_DURATION,
};
use crate::field_builder::{FIELD_WIDTH, GOAL_DEPTH, GOAL_WIDTH};
use ynwa_core::field::zones::{Point3D, Rectangle, ZoneGeometry};
use ynwa_core::field::{FieldBuilder, Zone};
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef};
use ynwa_core::region::GridCell;
use ynwa_core::team::Team;
use uom::si::f32::Length;
use uom::si::length::meter;

// Derived from production constants: goal centered on the field width
const GOAL_X_MIN: f32 = (FIELD_WIDTH - GOAL_WIDTH) / 2.0;
const GOAL_X_MAX: f32 = GOAL_X_MIN + GOAL_WIDTH;
const GOAL_X_CENTER: f32 = FIELD_WIDTH / 2.0;
const BALL_R: f32 = BALL_RADIUS;

// Test field length — intentionally round number, distinct from DEFAULT_LENGTH (101.538m)
const TEST_FIELD_LENGTH: f32 = 100.0;
const TEAM_B_GOAL_Z: f32 = TEST_FIELD_LENGTH; // z where Team B goal line sits

/// Standard football field: goals at z=0 (Team A) and z=TEST_FIELD_LENGTH (Team B), centered on X axis
fn create_test_game() -> Game {
    let field = FieldBuilder::from_meters(FIELD_WIDTH, TEST_FIELD_LENGTH, 26, 44)
        .with_zone(Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(GOAL_X_MIN, -GOAL_DEPTH, GOAL_X_MAX, 0.0)),
        ))
        .with_zone(Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(GOAL_X_MIN, TEAM_B_GOAL_Z, GOAL_X_MAX, TEAM_B_GOAL_Z + GOAL_DEPTH)),
        ))
        .build();

    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A, 1, "Test".to_string(), String::new(), start_region,
        )],
        ball: BallDef::default(),
        referees: vec![],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play)
}

fn set_ball(game: &mut Game, x: f32, z: f32) {
    game.state.ball_state.position = Point3D::new(
        Length::new::<meter>(x),
        Length::new::<meter>(0.0),
        Length::new::<meter>(z),
    );
}

// --- check_goal ---

#[test]
fn test_goal_team_a_center() {
    // Ball fully inside goal, center of goal mouth
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_team_b_center() {
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_goal_not_scored_ball_on_line() {
    // Ball center on the goal line, radius sticks into the field — not fully crossed
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, 0.0);
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_not_scored_ball_partially_crossed() {
    // Ball center 5cm past the line but radius=11cm, so ball not fully past
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, -0.05);
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_scored_ball_exactly_fully_crossed() {
    // Ball center exactly one radius past the line — ball fully crossed
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, -BALL_R);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_near_post_center_inside() {
    // Ball center 1cm inside the post — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN + 0.01, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_near_post_center_on_post() {
    // Ball center exactly on the post line — goal (inner edge counts)
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_near_post_center_outside() {
    // Ball center 1cm outside the post — not a goal even if ball touches post
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 0.01, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_far_post_center_inside() {
    // Team A goal, right post (GOAL_X_MAX): ball center 1cm inside — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX - 0.01, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_far_post_center_on_post() {
    // Team A goal, right post: ball center exactly on post — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_goal_far_post_center_outside() {
    // Team A goal, right post: ball center 1cm outside — not a goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX + 0.01, -(BALL_R + 0.01));
    assert_eq!(check_goal(&game), None);
}

// Team B goal (z > TEAM_B_GOAL_Z) — mirror tests for the opposite end

#[test]
fn test_goal_team_b_near_post_center_inside() {
    // Team B goal, left post (GOAL_X_MIN): ball center 1cm inside — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN + 0.01, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_goal_team_b_near_post_center_on_post() {
    // Team B goal, left post: ball center exactly on post — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_goal_team_b_near_post_center_outside() {
    // Team B goal, left post: ball center 1cm outside — not a goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 0.01, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_team_b_far_post_center_inside() {
    // Team B goal, right post (GOAL_X_MAX): ball center 1cm inside — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX - 0.01, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_goal_team_b_far_post_center_on_post() {
    // Team B goal, right post: ball center exactly on post — goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

#[test]
fn test_goal_team_b_far_post_center_outside() {
    // Team B goal, right post: ball center 1cm outside — not a goal
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX + 0.01, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_team_b_not_scored_ball_partially_crossed() {
    // Ball center 5cm past the Team B goal line but radius=11cm — not fully crossed
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, TEAM_B_GOAL_Z + 0.05);
    assert_eq!(check_goal(&game), None);
}

#[test]
fn test_goal_team_b_scored_ball_exactly_fully_crossed() {
    // Ball center exactly one radius past the Team B goal line — fully crossed
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, TEAM_B_GOAL_Z + BALL_R);
    assert_eq!(check_goal(&game), Some(FootballEvent::Goal(Team::B)));
}

// --- check_goal_line ---

#[test]
fn test_goal_line_near_outside_post() {
    // Ball crosses z=0 outside the goalposts — GoalLine
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 1.0, -(BALL_R + 0.01));
    assert!(matches!(check_goal_line(&game), Some(FootballEvent::GoalLine(_, _))));
}

#[test]
fn test_goal_line_far_outside_post() {
    // Ball crosses z=TEAM_B_GOAL_Z outside the goalposts — GoalLine
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MAX + 1.0, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert!(matches!(check_goal_line(&game), Some(FootballEvent::GoalLine(_, _))));
}

#[test]
fn test_goal_line_not_fired_between_goalposts() {
    // Ball crosses z=0 between the goalposts — not GoalLine (check_goal handles it)
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, -(BALL_R + 0.01));
    assert_eq!(check_goal_line(&game), None);
}

#[test]
fn test_goal_line_team_b_not_fired_between_goalposts() {
    // Ball crosses z=TEAM_B_GOAL_Z between the goalposts — not GoalLine (check_goal handles it)
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert_eq!(check_goal_line(&game), None);
}

#[test]
fn test_goal_line_team_b_near_outside_post() {
    // Ball crosses z=TEAM_B_GOAL_Z just outside the near post — GoalLine
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 1.0, TEAM_B_GOAL_Z + BALL_R + 0.01);
    assert!(matches!(check_goal_line(&game), Some(FootballEvent::GoalLine(_, _))));
}

#[test]
fn test_goal_line_not_fired_ball_on_line_in_play() {
    // Ball touching the line but not past it — no event
    let mut game = create_test_game();
    set_ball(&mut game, 5.0, BALL_R); // z - R = 0.0, not < 0
    assert_eq!(check_goal_line(&game), None);
}

// --- check_touchline ---

#[test]
fn test_touchline_left() {
    let mut game = create_test_game();
    set_ball(&mut game, -(BALL_R + 0.01), 50.0);
    assert!(matches!(check_touchline(&game), Some(FootballEvent::Touchline(_, _))));
}

#[test]
fn test_touchline_right() {
    let mut game = create_test_game();
    set_ball(&mut game, FIELD_WIDTH + BALL_R + 0.01, 50.0);
    assert!(matches!(check_touchline(&game), Some(FootballEvent::Touchline(_, _))));
}

#[test]
fn test_touchline_ball_on_line_in_play() {
    // Ball center at exactly BALL_R from line — still in play
    let mut game = create_test_game();
    set_ball(&mut game, BALL_R, 50.0); // x - R = 0.0, not < 0
    assert_eq!(check_touchline(&game), None);
}

// --- check_game_end ---

#[test]
fn test_game_end() {
    let mut game = create_test_game();
    game.state.elapsed_time = GAME_DURATION;
    assert_eq!(check_game_end(&game), Some(FootballEvent::GameEnd));
}

#[test]
fn test_game_not_ended() {
    let mut game = create_test_game();
    game.state.elapsed_time = GAME_DURATION - 0.1;
    assert_eq!(check_game_end(&game), None);
}

// --- check_events priority ---

#[test]
fn test_check_events_priority_goal_over_game_end() {
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_CENTER, -(BALL_R + 0.01));
    game.state.elapsed_time = GAME_DURATION;
    assert_eq!(check_events(&game), Some(FootballEvent::Goal(Team::A)));
}

#[test]
fn test_check_events_priority_game_end_over_goal_line() {
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 1.0, -(BALL_R + 0.01)); // outside post = GoalLine
    game.state.elapsed_time = GAME_DURATION;
    assert_eq!(check_events(&game), Some(FootballEvent::GameEnd));
}

#[test]
fn test_check_events_no_event() {
    let mut game = create_test_game();
    set_ball(&mut game, 30.0, 50.0);
    game.state.elapsed_time = 30.0;
    assert_eq!(check_events(&game), None);
}

// --- last_possessing_team in out-of-bounds events ---

fn set_last_team(game: &mut Game, team: Option<Team>) {
    game.state.ball_state.last_possessing_team = team;
}

#[test]
fn test_touchline_carries_last_team() {
    let mut game = create_test_game();
    set_ball(&mut game, -(BALL_R + 0.01), 50.0);
    set_last_team(&mut game, Some(Team::B));
    assert_eq!(check_touchline(&game), Some(FootballEvent::Touchline(game.state.ball_state.position, Team::B)));
}

#[test]
fn test_touchline_default_team_when_none() {
    let mut game = create_test_game();
    set_ball(&mut game, -(BALL_R + 0.01), 50.0);
    set_last_team(&mut game, None);
    assert!(matches!(check_touchline(&game), Some(FootballEvent::Touchline(_, Team::A))));
}

#[test]
fn test_goal_line_carries_last_team() {
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 1.0, -(BALL_R + 0.01));
    set_last_team(&mut game, Some(Team::B));
    assert!(matches!(check_goal_line(&game), Some(FootballEvent::GoalLine(_, Team::B))));
}

#[test]
fn test_goal_line_default_team_when_none() {
    let mut game = create_test_game();
    set_ball(&mut game, GOAL_X_MIN - 1.0, -(BALL_R + 0.01));
    set_last_team(&mut game, None);
    assert!(matches!(check_goal_line(&game), Some(FootballEvent::GoalLine(_, Team::A))));
}
