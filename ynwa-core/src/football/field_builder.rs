use crate::field::zones::*;
use crate::field::{Field, FieldBuilder, Zone};
use crate::team::Team;
use std::f32::consts::PI;

// FIFA regulation dimensions (meters)
const DEFAULT_LENGTH: f32 = 100.0;
const DEFAULT_WIDTH: f32 = 60.0;
const GOAL_AREA_LENGTH: f32 = 5.5;
const GOAL_AREA_WIDTH: f32 = 18.32;
const PENALTY_AREA_LENGTH: f32 = 16.5;
const PENALTY_AREA_WIDTH: f32 = 40.32;
const PENALTY_SPOT_DISTANCE: f32 = 11.0;
const CENTER_CIRCLE_RADIUS: f32 = 9.15;
const CORNER_ARC_RADIUS: f32 = 1.0;
const PENALTY_ARC_RADIUS: f32 = 9.15;
const GOAL_DEPTH: f32 = 2.5;
const GOAL_WIDTH: f32 = 7.32;

/// Creates a standard football field (100m x 60m) with all regulation zones
pub fn create_football_field() -> Field {
    create_football_field_with_dimensions(DEFAULT_LENGTH, DEFAULT_WIDTH)
}

/// Creates a football field with custom dimensions and FIFA-proportional zones
pub fn create_football_field_with_dimensions(length: f32, width: f32) -> Field {
    type ZoneSpec = (&'static str, Option<Team>, Box<dyn Fn() -> ZoneGeometry>);
    
    let half_length = length / 2.0;
    let half_width = width / 2.0;

    let zones: Vec<ZoneSpec> = vec![
        (
            "field",
            None,
            Box::new(move || ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, length, width))),
        ),
        (
            "half",
            Some(Team::A),
            Box::new(move || ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, half_length, width))),
        ),
        (
            "half",
            Some(Team::B),
            Box::new(move || ZoneGeometry::Rectangle(Rectangle::from_meters(half_length, 0.0, length, width))),
        ),
        (
            "goal_area",
            Some(Team::A),
            Box::new(move || {
                let z_min = (width - GOAL_AREA_WIDTH) / 2.0;
                let z_max = z_min + GOAL_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, z_min, GOAL_AREA_LENGTH, z_max))
            }),
        ),
        (
            "goal_area",
            Some(Team::B),
            Box::new(move || {
                let z_min = (width - GOAL_AREA_WIDTH) / 2.0;
                let z_max = z_min + GOAL_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    length - GOAL_AREA_LENGTH,
                    z_min,
                    length,
                    z_max,
                ))
            }),
        ),
        (
            "penalty_area",
            Some(Team::A),
            Box::new(move || {
                let z_min = (width - PENALTY_AREA_WIDTH) / 2.0;
                let z_max = z_min + PENALTY_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, z_min, PENALTY_AREA_LENGTH, z_max))
            }),
        ),
        (
            "penalty_area",
            Some(Team::B),
            Box::new(move || {
                let z_min = (width - PENALTY_AREA_WIDTH) / 2.0;
                let z_max = z_min + PENALTY_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    length - PENALTY_AREA_LENGTH,
                    z_min,
                    length,
                    z_max,
                ))
            }),
        ),
        (
            "center_circle",
            None,
            Box::new(move || ZoneGeometry::Circle(Circle::from_meters(half_length, half_width, CENTER_CIRCLE_RADIUS))),
        ),
        (
            "penalty_arc",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    PENALTY_SPOT_DISTANCE,
                    half_width,
                    PENALTY_ARC_RADIUS,
                    -PI / 2.0,
                    PI / 2.0,
                ))
            }),
        ),
        (
            "penalty_arc",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    length - PENALTY_SPOT_DISTANCE,
                    half_width,
                    PENALTY_ARC_RADIUS,
                    PI / 2.0,
                    3.0 * PI / 2.0,
                ))
            }),
        ),
        (
            "corner_arc_bottom",
            Some(Team::A),
            Box::new(move || ZoneGeometry::Arc(Arc::from_radians(0.0, 0.0, CORNER_ARC_RADIUS, 0.0, PI / 2.0))),
        ),
        (
            "corner_arc_top",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    0.0,
                    width,
                    CORNER_ARC_RADIUS,
                    -PI / 2.0,
                    0.0,
                ))
            }),
        ),
        (
            "corner_arc_bottom",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    length,
                    0.0,
                    CORNER_ARC_RADIUS,
                    PI / 2.0,
                    PI,
                ))
            }),
        ),
        (
            "corner_arc_top",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(length, width, CORNER_ARC_RADIUS, PI, 3.0 * PI / 2.0))
            }),
        ),
        (
            "center_spot",
            None,
            Box::new(move || ZoneGeometry::Point(PointZone::from_meters(half_length, half_width))),
        ),
        (
            "penalty_spot",
            Some(Team::A),
            Box::new(move || ZoneGeometry::Point(PointZone::from_meters(PENALTY_SPOT_DISTANCE, half_width))),
        ),
        (
            "penalty_spot",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Point(PointZone::from_meters(
                    length - PENALTY_SPOT_DISTANCE,
                    half_width,
                ))
            }),
        ),
        (
            "goal",
            Some(Team::A),
            Box::new(move || {
                let z_min = (width - GOAL_WIDTH) / 2.0;
                let z_max = z_min + GOAL_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(-GOAL_DEPTH, z_min, 0.0, z_max))
            }),
        ),
        (
            "goal",
            Some(Team::B),
            Box::new(move || {
                let z_min = (width - GOAL_WIDTH) / 2.0;
                let z_max = z_min + GOAL_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(length, z_min, length + GOAL_DEPTH, z_max))
            }),
        ),
    ];

    let mut builder = FieldBuilder::from_meters(width, length);
    for (name, team, zone_fn) in zones {
        builder = builder.with_zone(Zone::new(name, team, zone_fn()));
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;

    #[test]
    fn test_create_football_field() {
        let field = create_football_field();

        assert_eq!(field.length().get::<meter>(), DEFAULT_LENGTH);
        assert_eq!(field.width().get::<meter>(), DEFAULT_WIDTH);

        // Check that required zones exist with correct teams
        assert!(field.get_zone("field", None).is_some());
        assert!(field.get_zone("half", Some(Team::A)).is_some());
        assert!(field.get_zone("half", Some(Team::B)).is_some());
        assert!(field.get_zone("center_circle", None).is_some());
        assert!(field.get_zone("center_spot", None).is_some());
        assert!(field.get_zone("goal_area", Some(Team::A)).is_some());
        assert!(field.get_zone("penalty_area", Some(Team::A)).is_some());
        assert!(field.get_zone("penalty_spot", Some(Team::A)).is_some());
        assert!(field.get_zone("goal", Some(Team::A)).is_some());

        assert_eq!(field.zones().len(), 19);
    }

    #[test]
    fn test_custom_dimensions() {
        let field = create_football_field_with_dimensions(120.0, 80.0);
        assert_eq!(field.length().get::<meter>(), 120.0);
        assert_eq!(field.width().get::<meter>(), 80.0);
    }

    #[test]
    fn test_mini_football_field() {
        // Mini football / futsal field
        let field = create_football_field_with_dimensions(40.0, 20.0);
        assert_eq!(field.length().get::<meter>(), 40.0);
        assert_eq!(field.width().get::<meter>(), 20.0);
        assert_eq!(field.zones().len(), 19);
    }
}
