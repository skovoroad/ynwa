pub mod zones;

use crate::team::Team;
use std::collections::HashMap;
use uom::si::f32::Length;
use uom::si::length::meter;
use zones::ZoneGeometry;

/// Zone on the field with geometry and optional team ownership.
///
/// Note: `name` and `team` are stored both in the HashMap key and in this struct.
/// This is intentional for:
/// - O(1) lookup by (name, team) key
/// - Self-contained Zone objects that can be passed around with full context
/// - Memory overhead is negligible (~100-200 bytes total for 19 zones)
#[derive(Debug, Clone, PartialEq)]
pub struct Zone {
    pub name: String,
    pub team: Option<Team>,
    pub geometry: ZoneGeometry,
}

impl Zone {
    pub fn new(name: impl Into<String>, team: Option<Team>, geometry: ZoneGeometry) -> Self {
        Self {
            name: name.into(),
            team,
            geometry,
        }
    }
}

/// Playing field with dimensions and zones.
///
/// Zones are stored in a HashMap with (name, team) as the key for O(1) lookup.
/// The name and team are also stored in the Zone struct itself to make zones
/// self-contained and easy to pass to rendering/physics systems.
///
/// Grid dimensions define how the field is divided for region addressing.
#[derive(Debug, Clone)]
pub struct Field {
    width: Length,
    length: Length,
    grid_dims: crate::region::GridDimensions,
    zones: HashMap<(String, Option<Team>), Zone>,
}

impl Field {
    pub fn new(width: Length, length: Length, grid_dims: crate::region::GridDimensions) -> Self {
        Self {
            width,
            length,
            grid_dims,
            zones: HashMap::new(),
        }
    }

    pub fn from_meters(width: f32, length: f32, grid_columns: u32, grid_rows: u32) -> Self {
        Self::new(
            Length::new::<meter>(width),
            Length::new::<meter>(length),
            crate::region::GridDimensions::new(grid_columns, grid_rows),
        )
    }

    pub fn width(&self) -> Length {
        self.width
    }

    pub fn length(&self) -> Length {
        self.length
    }

    pub fn grid_columns(&self) -> u32 {
        self.grid_dims.columns
    }

    pub fn grid_rows(&self) -> u32 {
        self.grid_dims.rows
    }

    pub fn grid_dimensions(&self) -> crate::region::GridDimensions {
        self.grid_dims
    }

    /// Returns the width of a single grid cell in meters.
    /// Assumes square cells (width of field divided by number of columns).
    pub fn cell_size(&self) -> f32 {
        self.width.get::<meter>() / self.grid_dims.columns as f32
    }

    pub fn add_zone(&mut self, zone: Zone) {
        self.zones.insert((zone.name.clone(), zone.team), zone);
    }

    pub fn get_zone(&self, name: &str, team: Option<Team>) -> Option<&Zone> {
        self.zones.get(&(name.to_string(), team))
    }

    pub fn zones(&self) -> &HashMap<(String, Option<Team>), Zone> {
        &self.zones
    }
}

pub struct FieldBuilder {
    field: Field,
}

impl FieldBuilder {
    pub fn new(width: Length, length: Length, grid_columns: u32, grid_rows: u32) -> Self {
        Self {
            field: Field::new(
                width,
                length,
                crate::region::GridDimensions::new(grid_columns, grid_rows),
            ),
        }
    }

    pub fn from_meters(width: f32, length: f32, grid_columns: u32, grid_rows: u32) -> Self {
        Self {
            field: Field::from_meters(width, length, grid_columns, grid_rows),
        }
    }

    pub fn with_zone(mut self, zone: Zone) -> Self {
        self.field.add_zone(zone);
        self
    }

    pub fn build(self) -> Field {
        self.field
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zones::Rectangle;

    #[test]
    fn test_field_dimensions() {
        let field = Field::from_meters(100.0, 60.0, 26, 26);
        assert_eq!(field.width().get::<meter>(), 100.0);
        assert_eq!(field.length().get::<meter>(), 60.0);
        assert_eq!(field.grid_columns(), 26);
        assert_eq!(field.grid_rows(), 26);
    }

    #[test]
    fn test_field_cell_size() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let cell_size = field.cell_size();
        assert!((cell_size - (60.0 / 26.0)).abs() < 0.001);
    }

    #[test]
    fn test_field_builder_with_zones() {
        let zone_a = Zone::new(
            "penalty_area",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 16.5, 40.0)),
        );
        let zone_b = Zone::new(
            "penalty_area",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(83.5, 0.0, 100.0, 40.0)),
        );

        let field = FieldBuilder::from_meters(100.0, 60.0, 26, 44)
            .with_zone(zone_a)
            .with_zone(zone_b)
            .build();

        assert_eq!(field.zones().len(), 2);
        assert!(field.get_zone("penalty_area", Some(Team::A)).is_some());
        assert!(field.get_zone("penalty_area", Some(Team::B)).is_some());
    }

    #[test]
    fn test_zone_lookup_by_name_and_team() {
        let mut field = Field::from_meters(100.0, 60.0, 26, 44);

        let zone_team_a = Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 26.84, 2.5, 33.16)),
        );
        let zone_team_b = Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(97.5, 26.84, 100.0, 33.16)),
        );

        field.add_zone(zone_team_a);
        field.add_zone(zone_team_b);

        // Both teams have "goal" zones, but they're different
        assert!(field.get_zone("goal", Some(Team::A)).is_some());
        assert!(field.get_zone("goal", Some(Team::B)).is_some());

        // Non-existent combinations should return None
        assert!(field.get_zone("goal", None).is_none());
        assert!(field.get_zone("nonexistent", Some(Team::A)).is_none());
    }

    #[test]
    fn test_zone_without_team() {
        let mut field = Field::from_meters(100.0, 60.0, 26, 44);

        let center_circle = Zone::new(
            "center_circle",
            None,
            ZoneGeometry::Rectangle(Rectangle::from_meters(40.0, 25.0, 60.0, 35.0)),
        );

        field.add_zone(center_circle);

        assert!(field.get_zone("center_circle", None).is_some());
        assert!(field.get_zone("center_circle", Some(Team::A)).is_none());
        assert!(field.get_zone("center_circle", Some(Team::B)).is_none());
    }
}
