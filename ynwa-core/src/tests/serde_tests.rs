//! Round-trip tests for serialization of core configuration types.

use crate::field::zones::{Arc, Circle, Point3D, PointZone, Rectangle, Velocity3D, ZoneGeometry};
use crate::field::{Field, FieldBuilder, Zone};
use crate::game::{
    BallDef, Decision, DecisionTarget, GameConfig, GameStage, PlayerDef, RefereeDef,
    ScriptingConfig, REGION_START_POSITION,
};
use crate::region::{GridCell, GridDimensions, Region};
use crate::team::Team;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::{HashMap, HashSet};
use std::f32::consts::PI;
use uom::si::angle::degree;
use uom::si::f32::{Angle, Length};
use uom::si::length::meter;

fn round_trip<T: Serialize + DeserializeOwned>(value: &T) -> T {
    let json = serde_json::to_string(value).expect("serialize to JSON");
    serde_json::from_str(&json).expect("deserialize from JSON")
}

fn assert_round_trip<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    assert_eq!(round_trip(&value), value);
}

/// `Angle` keeps radians internally while the record keeps degrees, so a round trip may differ
/// in the last bits; everything else has to survive bit-exactly.
fn assert_angle_close(actual: Angle, expected: Angle) {
    let diff = (actual.get::<degree>() - expected.get::<degree>()).abs();
    assert!(
        diff <= 1e-3,
        "angle {} != {}",
        actual.get::<degree>(),
        expected.get::<degree>()
    );
}

fn cell(col: u32, row: u32) -> GridCell {
    GridCell::new(col, row).expect("valid cell")
}

fn test_region() -> Region {
    Region::new(cell(1, 2), cell(3, 4))
}

fn test_field() -> Field {
    FieldBuilder::from_meters(68.0, 104.6, 26, 40)
        .with_zone(Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 30.0, 2.0, 38.0)),
        ))
        .with_zone(Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(66.0, 30.0, 68.0, 38.0)),
        ))
        .with_zone(Zone::new(
            "center_circle",
            None,
            ZoneGeometry::Circle(Circle::from_meters(34.0, 52.3, 9.15)),
        ))
        .with_zone(Zone::new(
            "kick_off",
            None,
            ZoneGeometry::Point(PointZone::from_meters(34.0, 52.3)),
        ))
        .build()
}

fn test_player_def(team: Team, number: u32) -> PlayerDef {
    PlayerDef::new(
        team,
        number,
        format!("Player {:?}{}", team, number),
        "function make_decision() return { action = 'stop' } end".to_string(),
        HashMap::from([
            (REGION_START_POSITION.to_string(), test_region()),
            (
                "attack".to_string(),
                Region::new(cell(10, 10), cell(12, 12)),
            ),
        ]),
    )
    .with_reaction_rate(70)
    .with_speed_rate(80)
    .with_shot_power(90)
    .with_set_piece_roles(HashSet::from([
        "goal kick own".to_string(),
        "corner own".to_string(),
    ]))
}

fn test_game_config() -> GameConfig {
    GameConfig {
        field: test_field(),
        players: vec![test_player_def(Team::A, 9), test_player_def(Team::B, 1)],
        ball: BallDef {
            initial_position: Point3D::on_ground(34.0, 52.3),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig {
            core_preamble: "core".to_string(),
            stdlib_preamble: "stdlib".to_string(),
            team_a_preamble: "team a".to_string(),
            team_b_preamble: "team b".to_string(),
        },
    }
}

#[test]
fn point3d_is_serialized_in_meters() {
    let point = Point3D::from_meters(1.5, 2.0, -3.25);

    assert_eq!(
        serde_json::to_value(point).unwrap(),
        serde_json::json!({"x": 1.5, "y": 2.0, "z": -3.25})
    );
    assert_round_trip(point);
}

#[test]
fn velocity3d_is_serialized_in_meters_per_second() {
    let velocity = Velocity3D::from_meters_per_second(0.5, 0.0, -1.25);

    assert_eq!(
        serde_json::to_value(velocity).unwrap(),
        serde_json::json!({"x": 0.5, "y": 0.0, "z": -1.25})
    );
    assert_round_trip(velocity);
}

#[test]
fn rectangle_round_trip() {
    assert_round_trip(Rectangle::from_meters(0.0, 0.0, 16.5, 40.0));
}

#[test]
fn circle_round_trip() {
    let circle = Circle::from_meters(34.0, 52.3, 9.15);

    let json: serde_json::Value =
        serde_json::from_str(&serde_json::to_string(&circle).unwrap()).unwrap();
    assert!((json["radius"].as_f64().unwrap() - 9.15).abs() < 1e-3);
    assert_round_trip(circle);
}

#[test]
fn arc_is_serialized_in_degrees() {
    let arc = Arc::new(
        Point3D::on_ground(10.0, 10.0),
        Length::new::<meter>(5.0),
        Angle::new::<degree>(30.0),
        Angle::new::<degree>(120.0),
    );

    let json = serde_json::to_value(&arc).unwrap();
    assert_eq!(json["radius"], serde_json::json!(5.0));
    assert!((json["start_angle"].as_f64().unwrap() - 30.0).abs() < 1e-3);
    assert!((json["end_angle"].as_f64().unwrap() - 120.0).abs() < 1e-3);

    let restored = round_trip(&arc);
    assert_eq!(restored.center, arc.center);
    assert_eq!(restored.radius, arc.radius);
    assert_angle_close(restored.start_angle, arc.start_angle);
    assert_angle_close(restored.end_angle, arc.end_angle);
}

#[test]
fn arc_round_trip_from_radians() {
    let arc = Arc::from_radians(0.0, 0.0, 5.0, 0.0, PI / 2.0);

    let restored = round_trip(&arc);
    assert_eq!(restored.center, arc.center);
    assert_eq!(restored.radius, arc.radius);
    assert_angle_close(restored.start_angle, arc.start_angle);
    assert_angle_close(restored.end_angle, arc.end_angle);
}

#[test]
fn point_zone_round_trip() {
    assert_round_trip(PointZone::from_meters(1.0, 2.0));
}

#[test]
fn zone_geometry_variants_round_trip() {
    let geometries = [
        ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 10.0, 20.0)),
        ZoneGeometry::Circle(Circle::from_meters(5.0, 5.0, 3.0)),
        ZoneGeometry::Point(PointZone::from_meters(1.0, 2.0)),
    ];

    for geometry in geometries {
        assert_round_trip(geometry);
    }
}

#[test]
fn zone_round_trips_with_and_without_team() {
    let zones = [
        Zone::new(
            "penalty_area",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 16.5, 40.0)),
        ),
        Zone::new(
            "center_circle",
            None,
            ZoneGeometry::Circle(Circle::from_meters(34.0, 52.3, 9.15)),
        ),
    ];

    for zone in zones {
        assert_round_trip(zone);
    }
}

#[test]
fn field_round_trip_preserves_dimensions_and_zones() {
    let field = test_field();

    let restored = round_trip(&field);

    assert_eq!(restored, field);
    assert_eq!(restored.zones().len(), 4);
    assert!(restored.get_zone("goal", Some(Team::B)).is_some());
}

#[test]
fn field_serialization_is_independent_of_zone_insertion_order() {
    let forward = test_field();
    let backward = FieldBuilder::from_meters(68.0, 104.6, 26, 40)
        .with_zone(Zone::new(
            "kick_off",
            None,
            ZoneGeometry::Point(PointZone::from_meters(34.0, 52.3)),
        ))
        .with_zone(Zone::new(
            "center_circle",
            None,
            ZoneGeometry::Circle(Circle::from_meters(34.0, 52.3, 9.15)),
        ))
        .with_zone(Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(66.0, 30.0, 68.0, 38.0)),
        ))
        .with_zone(Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 30.0, 2.0, 38.0)),
        ))
        .build();

    assert_eq!(
        serde_json::to_string(&forward).unwrap(),
        serde_json::to_string(&backward).unwrap()
    );
}

#[test]
fn grid_dimensions_round_trip() {
    assert_round_trip(GridDimensions::new(26, 40));
}

#[test]
fn grid_cells_round_trip() {
    for cell in [cell(1, 1), cell(26, 40), cell(33, 7)] {
        assert_round_trip(cell);
    }
}

#[test]
fn regions_round_trip() {
    let regions = [
        Region::new(cell(1, 1), cell(2, 2)),
        Region::new(cell(15, 22), cell(15, 22)),
    ];

    for region in regions {
        assert_round_trip(region);
    }
}

#[test]
fn teams_round_trip() {
    for team in [Team::A, Team::B] {
        assert_round_trip(team);
    }
}

#[test]
fn game_stages_round_trip() {
    let stages = [
        GameStage::Play,
        GameStage::Setup("throw in".to_string()),
        GameStage::Setup("kick off".to_string()),
        GameStage::GameOver,
    ];

    for stage in stages {
        assert_round_trip(stage);
    }
}

#[test]
fn decisions_round_trip() {
    let decisions = [
        Decision::Run(DecisionTarget::Region(test_region())),
        Decision::Run(DecisionTarget::GridCell(cell(13, 22))),
        Decision::Run(DecisionTarget::Point(Point3D::on_ground(10.0, 20.0))),
        Decision::Run(DecisionTarget::Ball),
        Decision::Stop,
        Decision::Kick(Point3D::from_meters(34.0, 0.0, 52.3)),
    ];

    for decision in decisions {
        assert_round_trip(decision);
    }
}

#[test]
fn player_def_round_trip() {
    assert_round_trip(test_player_def(Team::A, 9));
}

#[test]
fn player_def_serialization_is_independent_of_collection_order() {
    let forward = test_player_def(Team::B, 2);

    let mut regions = HashMap::new();
    regions.insert(
        "attack".to_string(),
        Region::new(cell(10, 10), cell(12, 12)),
    );
    regions.insert(REGION_START_POSITION.to_string(), test_region());
    let backward = PlayerDef::new(
        Team::B,
        2,
        "Player B2".to_string(),
        "function make_decision() return { action = 'stop' } end".to_string(),
        regions,
    )
    .with_reaction_rate(70)
    .with_speed_rate(80)
    .with_shot_power(90)
    .with_set_piece_roles(HashSet::from([
        "corner own".to_string(),
        "goal kick own".to_string(),
    ]));

    let json = serde_json::to_string(&forward).unwrap();
    assert_eq!(json, serde_json::to_string(&backward).unwrap());
    assert!(json.find("\"attack\"").unwrap() < json.find("\"start\"").unwrap());
    assert!(json.find("\"corner own\"").unwrap() < json.find("\"goal kick own\"").unwrap());
}

#[test]
fn ball_def_round_trip() {
    assert_round_trip(BallDef {
        initial_position: Point3D::on_ground(34.0, 52.3),
    });
}

#[test]
fn referee_def_round_trip() {
    assert_round_trip(RefereeDef::default());
}

#[test]
fn scripting_config_round_trip() {
    assert_round_trip(test_game_config().scripting);
}

#[test]
fn game_config_round_trip() {
    assert_round_trip(test_game_config());
}
