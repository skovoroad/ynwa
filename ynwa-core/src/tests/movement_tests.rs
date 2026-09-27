//! Tests for the decision-to-movement helpers shared by `ActionSystem` and replay.

use super::{calculate_target_point, calculate_velocity};
use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::game::{
    BallDef, DecisionTarget, Game, GameConfig, GameStage, PlayerDef, RefereeDef, ScriptingConfig,
    REGION_START_POSITION,
};
use crate::region::{GridCell, Region};
use crate::team::Team;
use crate::test_utils::deterministic_rng;
use std::collections::HashMap;
use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

fn game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 16);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(12, 8).unwrap(), GridCell::new(12, 8).unwrap())
        .unwrap();

    let player = PlayerDef::new(
        Team::A,
        1,
        "Runner".to_string(),
        String::new(),
        HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    );

    let config = GameConfig {
        field,
        players: vec![player],
        ball: BallDef {
            initial_position: Point3D::from_meters(10.0, 0.0, 20.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    };

    Game::with_stage(config, GameStage::Play, deterministic_rng())
}

#[test]
fn point_target_is_used_as_is() {
    let game = game();
    let point = Point3D::from_meters(1.0, 2.0, 3.0);

    assert_eq!(
        calculate_target_point(&DecisionTarget::Point(point), &game),
        point
    );
}

#[test]
fn grid_cell_target_resolves_to_cell_center() {
    let game = game();
    let cell = GridCell::new(12, 8).unwrap();
    let expected = Region::new(cell, cell).center(
        game.config().field.grid_dimensions(),
        game.config().field.width().get::<meter>(),
    );

    assert_eq!(
        calculate_target_point(&DecisionTarget::GridCell(cell), &game),
        expected
    );
}

#[test]
fn region_target_resolves_to_region_center() {
    let game = game();
    let grid_dims = game.config().field.grid_dimensions();
    let region = grid_dims
        .create_region(GridCell::new(3, 4).unwrap(), GridCell::new(6, 8).unwrap())
        .unwrap();
    let expected = region.center(grid_dims, game.config().field.width().get::<meter>());

    assert_eq!(
        calculate_target_point(&DecisionTarget::Region(region), &game),
        expected
    );
}

#[test]
fn ball_target_resolves_to_current_ball_position() {
    let game = game();

    assert_eq!(
        calculate_target_point(&DecisionTarget::Ball, &game),
        game.state.ball_state.position
    );
}

#[test]
fn velocity_points_to_target_at_full_speed_for_rate_100() {
    let from = Point3D::from_meters(0.0, 0.0, 0.0);
    let to = Point3D::from_meters(10.0, 0.0, 0.0);

    let velocity = calculate_velocity(&from, &to, 100);

    assert!((velocity.x.get::<meter_per_second>() - 10.0).abs() < 1e-4);
    assert!(velocity.y.get::<meter_per_second>().abs() < 1e-6);
    assert!(velocity.z.get::<meter_per_second>().abs() < 1e-6);
}

#[test]
fn velocity_scales_linearly_with_speed_rate() {
    let from = Point3D::from_meters(0.0, 0.0, 0.0);
    let to = Point3D::from_meters(0.0, 0.0, 10.0);

    let velocity = calculate_velocity(&from, &to, 50);

    assert!((velocity.z.get::<meter_per_second>() - 5.0).abs() < 1e-4);
}

#[test]
fn velocity_is_zero_when_target_is_within_stop_threshold() {
    let from = Point3D::from_meters(0.0, 0.0, 0.0);
    let to = Point3D::from_meters(0.4, 0.0, 0.0);

    assert_eq!(calculate_velocity(&from, &to, 100), Velocity3D::default());
}
