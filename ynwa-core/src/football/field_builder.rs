use crate::field::zones::*;
use crate::field::{Field, FieldBuilder, Zone};
use crate::team::Team;
use std::f32::consts::PI;

// FIFA regulation dimensions (meters)
// Adjusted to ensure square grid cells: 60m / 26 columns = ~2.3077m per cell
const DEFAULT_WIDTH: f32 = 60.0;
const DEFAULT_LENGTH: f32 = 101.538_46; // 44 rows × 2.3077m
const GOAL_AREA_LENGTH: f32 = 5.5;
const GOAL_AREA_WIDTH: f32 = 18.32;
const PENALTY_AREA_LENGTH: f32 = 16.153_846; // 7 cells × 2.3077m
const PENALTY_AREA_WIDTH: f32 = 41.538_46; // 18 cells × 2.3077m
const PENALTY_SPOT_DISTANCE: f32 = 11.0;
const CENTER_CIRCLE_RADIUS: f32 = 9.15;
const CORNER_ARC_RADIUS: f32 = 1.0;
const PENALTY_ARC_RADIUS: f32 = 9.15;
const GOAL_DEPTH: f32 = 2.5;
const GOAL_WIDTH: f32 = 7.32;

// Grid dimensions for football field
const FOOTBALL_GRID_COLUMNS: u32 = 26; // A-Z
const FOOTBALL_GRID_ROWS: u32 = 44; // Calculated for square cells

/// Creates a standard football field with all regulation zones
pub fn create_football_field() -> Field {
    create_football_field_with_dimensions(
        DEFAULT_WIDTH,
        DEFAULT_LENGTH,
        FOOTBALL_GRID_COLUMNS,
        FOOTBALL_GRID_ROWS,
    )
    .expect("Default football field dimensions should be valid")
}

/// Creates a football field with custom dimensions and FIFA-proportional zones.
/// Returns error if dimensions don't result in square grid cells.
pub fn create_football_field_with_dimensions(
    width: f32,
    length: f32,
    grid_columns: u32,
    grid_rows: u32,
) -> Result<Field, String> {
    // Validate that cells are square (width/columns == length/rows)
    let cell_width = width / grid_columns as f32;
    let cell_height = length / grid_rows as f32;

    // Check if cells are square (tolerance for floating point)
    if (cell_width - cell_height).abs() > 0.01 {
        return Err(format!(
            "Grid cells must be square: cell size from width={:.4}m, from length={:.4}m",
            cell_width, cell_height
        ));
    }

    type ZoneSpec = (&'static str, Option<Team>, Box<dyn Fn() -> ZoneGeometry>);

    let half_length = length / 2.0;
    let half_width = width / 2.0;

    let zones: Vec<ZoneSpec> = vec![
        (
            "field",
            None,
            Box::new(move || {
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, width, length))
            }),
        ),
        (
            "half",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, width, half_length))
            }),
        ),
        (
            "half",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, half_length, width, length))
            }),
        ),
        (
            "goal_area",
            Some(Team::A),
            Box::new(move || {
                let x_min = (width - GOAL_AREA_WIDTH) / 2.0;
                let x_max = x_min + GOAL_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(x_min, 0.0, x_max, GOAL_AREA_LENGTH))
            }),
        ),
        (
            "goal_area",
            Some(Team::B),
            Box::new(move || {
                let x_min = (width - GOAL_AREA_WIDTH) / 2.0;
                let x_max = x_min + GOAL_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    x_min,
                    length - GOAL_AREA_LENGTH,
                    x_max,
                    length,
                ))
            }),
        ),
        (
            "penalty_area",
            Some(Team::A),
            Box::new(move || {
                let x_min = (width - PENALTY_AREA_WIDTH) / 2.0;
                let x_max = x_min + PENALTY_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    x_min,
                    0.0,
                    x_max,
                    PENALTY_AREA_LENGTH,
                ))
            }),
        ),
        (
            "penalty_area",
            Some(Team::B),
            Box::new(move || {
                let x_min = (width - PENALTY_AREA_WIDTH) / 2.0;
                let x_max = x_min + PENALTY_AREA_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    x_min,
                    length - PENALTY_AREA_LENGTH,
                    x_max,
                    length,
                ))
            }),
        ),
        (
            "center_circle",
            None,
            Box::new(move || {
                ZoneGeometry::Circle(Circle::from_meters(
                    half_width,
                    half_length,
                    CENTER_CIRCLE_RADIUS,
                ))
            }),
        ),
        (
            "penalty_arc",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    half_width,
                    PENALTY_SPOT_DISTANCE,
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
                    half_width,
                    length - PENALTY_SPOT_DISTANCE,
                    PENALTY_ARC_RADIUS,
                    PI / 2.0,
                    3.0 * PI / 2.0,
                ))
            }),
        ),
        (
            "corner_arc_bottom",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    0.0,
                    0.0,
                    CORNER_ARC_RADIUS,
                    0.0,
                    PI / 2.0,
                ))
            }),
        ),
        (
            "corner_arc_top",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Arc(Arc::from_radians(
                    width,
                    0.0,
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
                    0.0,
                    length,
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
                ZoneGeometry::Arc(Arc::from_radians(
                    width,
                    length,
                    CORNER_ARC_RADIUS,
                    PI,
                    3.0 * PI / 2.0,
                ))
            }),
        ),
        (
            "center_spot",
            None,
            Box::new(move || ZoneGeometry::Point(PointZone::from_meters(half_width, half_length))),
        ),
        (
            "penalty_spot",
            Some(Team::A),
            Box::new(move || {
                ZoneGeometry::Point(PointZone::from_meters(half_width, PENALTY_SPOT_DISTANCE))
            }),
        ),
        (
            "penalty_spot",
            Some(Team::B),
            Box::new(move || {
                ZoneGeometry::Point(PointZone::from_meters(
                    half_width,
                    length - PENALTY_SPOT_DISTANCE,
                ))
            }),
        ),
        (
            "goal",
            Some(Team::A),
            Box::new(move || {
                let x_min = (width - GOAL_WIDTH) / 2.0;
                let x_max = x_min + GOAL_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(x_min, -GOAL_DEPTH, x_max, 0.0))
            }),
        ),
        (
            "goal",
            Some(Team::B),
            Box::new(move || {
                let x_min = (width - GOAL_WIDTH) / 2.0;
                let x_max = x_min + GOAL_WIDTH;
                ZoneGeometry::Rectangle(Rectangle::from_meters(
                    x_min,
                    length,
                    x_max,
                    length + GOAL_DEPTH,
                ))
            }),
        ),
    ];

    let mut builder = FieldBuilder::from_meters(width, length, grid_columns, grid_rows);
    for (name, team, zone_fn) in zones {
        builder = builder.with_zone(Zone::new(name, team, zone_fn()));
    }
    Ok(builder.build())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;

    #[test]
    fn test_create_football_field() {
        let field = create_football_field();

        assert_eq!(field.width().get::<meter>(), DEFAULT_WIDTH);
        assert!((field.length().get::<meter>() - DEFAULT_LENGTH).abs() < 0.01);

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
    fn test_custom_dimensions_valid() {
        // 52m width with 26 columns = 2m cells, 88m length with 44 rows = 2m cells (square!)
        let result = create_football_field_with_dimensions(52.0, 88.0, 26, 44);
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.width().get::<meter>(), 52.0);
        assert_eq!(field.length().get::<meter>(), 88.0);
    }

    #[test]
    fn test_custom_dimensions_invalid() {
        // 60m width with 26 columns = 2.3077m cells, but 100m length with 44 rows = 2.2727m cells (not square!)
        let result = create_football_field_with_dimensions(60.0, 100.0, 26, 44);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("square"));
    }

    #[test]
    fn test_mini_football_field() {
        // Mini football / futsal field with square cells
        // 20m width / 10 columns = 2m cells, 40m length / 20 rows = 2m cells
        let result = create_football_field_with_dimensions(20.0, 40.0, 10, 20);
        assert!(result.is_ok());
        let field = result.unwrap();
        assert_eq!(field.width().get::<meter>(), 20.0);
        assert_eq!(field.length().get::<meter>(), 40.0);
        assert_eq!(field.zones().len(), 19);
    }
}
