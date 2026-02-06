//! Coordinate orientation transformations for team perspectives.
//!
//! The field has a canonical "display orientation" (Team A's perspective: left-to-right).
//! Team B plays from the opposite side (right-to-left in their perspective).
//! These functions convert coordinates between team perspectives and display orientation.

use crate::field::zones::Point3D;
use crate::region::{GridCell, GridDimensions, Region, RegionError};
#[allow(unused_imports)] // Used in doctests
use crate::team::Team;
use uom::si::f32::Length;
use uom::si::length::meter;

/// Flips a grid cell's orientation for the opposite team.
///
/// # Example
/// ```
/// use ynwa_core::{GridCell, GridDimensions, orientation::flip_grid_cell_orientation};
///
/// let grid_dims = GridDimensions::new(26, 44);
/// let cell = GridCell::new(1, 1).unwrap();
/// let flipped = flip_grid_cell_orientation(&cell, grid_dims).unwrap();
/// assert_eq!(flipped, GridCell::new(26, 44).unwrap()); // A1 -> Z44
/// ```
pub fn flip_grid_cell_orientation(
    cell: &GridCell,
    grid_dims: GridDimensions,
) -> Result<GridCell, RegionError> {
    let new_col = grid_dims.columns - cell.col + 1;
    let new_row = grid_dims.rows - cell.row + 1;

    GridCell::new(new_col, new_row).map_err(|e| {
        RegionError::InvalidRegion(format!("Failed to flip grid cell orientation: {}", e))
    })
}

/// Flips a region's orientation for the opposite team.
/// Swaps team and corners to maintain invariants.
///
/// # Example
/// ```
/// use ynwa_core::{GridCell, GridDimensions, Region, orientation::flip_region_orientation};
/// use ynwa_core::team::Team;
///
/// let grid_dims = GridDimensions::new(26, 44);
/// let region = Region::new(
///     Team::A,
///     GridCell::new(1, 1).unwrap(),
///     GridCell::new(2, 2).unwrap(),
///     grid_dims
/// ).unwrap();
///
/// let flipped = flip_region_orientation(&region, grid_dims).unwrap();
/// assert_eq!(flipped.team, Team::B);
/// ```
pub fn flip_region_orientation(
    region: &Region,
    grid_dims: GridDimensions,
) -> Result<Region, RegionError> {
    let new_top_left = flip_grid_cell_orientation(&region.top_left, grid_dims)?;
    let new_bottom_right = flip_grid_cell_orientation(&region.bottom_right, grid_dims)?;

    // Swap corners to maintain top_left <= bottom_right after flip
    Region::new(
        region.team.opposite(),
        new_bottom_right,
        new_top_left,
        grid_dims,
    )
}

/// Flips a point's orientation for the opposite team.
/// Y coordinate (height) remains unchanged.
///
/// # Example
/// ```
/// use ynwa_core::{Point3D, orientation::flip_point_orientation};
/// use uom::si::length::meter;
///
/// let point = Point3D::from_meters(20.0, 1.0, 15.0);
/// let flipped = flip_point_orientation(&point, 100.0, 60.0);
///
/// assert_eq!(flipped.x.get::<meter>(), 80.0);
/// assert_eq!(flipped.y.get::<meter>(), 1.0); // unchanged
/// assert_eq!(flipped.z.get::<meter>(), 45.0);
/// ```
pub fn flip_point_orientation(point: &Point3D, field_width: f32, field_length: f32) -> Point3D {
    Point3D {
        x: Length::new::<meter>(field_width - point.x.get::<meter>()),
        y: point.y, // height doesn't change
        z: Length::new::<meter>(field_length - point.z.get::<meter>()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flip_grid_cell_corner() {
        let grid_dims = GridDimensions::new(26, 44);
        let cell = GridCell::new(1, 1).unwrap();
        let flipped = flip_grid_cell_orientation(&cell, grid_dims).unwrap();
        assert_eq!(flipped, GridCell::new(26, 44).unwrap());
    }

    #[test]
    fn test_flip_grid_cell_center() {
        let grid_dims = GridDimensions::new(26, 44);
        let cell = GridCell::new(13, 22).unwrap(); // center
        let flipped = flip_grid_cell_orientation(&cell, grid_dims).unwrap();
        assert_eq!(flipped, GridCell::new(14, 23).unwrap()); // slightly off-center due to even dimensions
    }

    #[test]
    fn test_flip_grid_cell_double_flip() {
        let grid_dims = GridDimensions::new(26, 44);
        let cell = GridCell::new(5, 10).unwrap();
        let flipped_once = flip_grid_cell_orientation(&cell, grid_dims).unwrap();
        let flipped_twice = flip_grid_cell_orientation(&flipped_once, grid_dims).unwrap();
        assert_eq!(flipped_twice, cell);
    }

    #[test]
    fn test_flip_region_orientation() {
        let grid_dims = GridDimensions::new(26, 44);
        let region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();

        let flipped = flip_region_orientation(&region, grid_dims).unwrap();

        // Team should be opposite
        assert_eq!(flipped.team, Team::B);

        // Coordinates should be flipped and swapped
        // Original: top_left=(1,1), bottom_right=(2,2)
        // After flip: (1,1)->(26,44), (2,2)->(25,43)
        // After swap to maintain order: top_left=(25,43), bottom_right=(26,44)
        assert_eq!(flipped.top_left, GridCell::new(25, 43).unwrap());
        assert_eq!(flipped.bottom_right, GridCell::new(26, 44).unwrap());
    }

    #[test]
    fn test_flip_region_double_flip() {
        let grid_dims = GridDimensions::new(26, 44);
        let region = Region::new(
            Team::A,
            GridCell::new(5, 10).unwrap(),
            GridCell::new(8, 15).unwrap(),
            grid_dims,
        )
        .unwrap();

        let flipped_once = flip_region_orientation(&region, grid_dims).unwrap();
        let flipped_twice = flip_region_orientation(&flipped_once, grid_dims).unwrap();

        assert_eq!(flipped_twice.team, region.team);
        assert_eq!(flipped_twice.top_left, region.top_left);
        assert_eq!(flipped_twice.bottom_right, region.bottom_right);
    }

    #[test]
    fn test_flip_point_corner() {
        // Point at (0, 0, 0) on 100x60 field should flip to (100, 0, 60)
        let point = Point3D::from_meters(0.0, 0.0, 0.0);
        let flipped = flip_point_orientation(&point, 100.0, 60.0);

        assert_eq!(flipped.x.get::<meter>(), 100.0);
        assert_eq!(flipped.y.get::<meter>(), 0.0); // height unchanged
        assert_eq!(flipped.z.get::<meter>(), 60.0);
    }

    #[test]
    fn test_flip_point_center() {
        // Point at center (50, 1, 30) on 100x60 field should flip to center (50, 1, 30)
        let point = Point3D::from_meters(50.0, 1.0, 30.0);
        let flipped = flip_point_orientation(&point, 100.0, 60.0);

        assert_eq!(flipped.x.get::<meter>(), 50.0);
        assert_eq!(flipped.y.get::<meter>(), 1.0);
        assert_eq!(flipped.z.get::<meter>(), 30.0);
    }

    #[test]
    fn test_flip_point_arbitrary() {
        // Point at (20, 2, 15) on 100x60 field should flip to (80, 2, 45)
        let point = Point3D::from_meters(20.0, 2.0, 15.0);
        let flipped = flip_point_orientation(&point, 100.0, 60.0);

        assert_eq!(flipped.x.get::<meter>(), 80.0);
        assert_eq!(flipped.y.get::<meter>(), 2.0);
        assert_eq!(flipped.z.get::<meter>(), 45.0);
    }

    #[test]
    fn test_flip_point_double_flip() {
        // Double flip should return to original point
        let point = Point3D::from_meters(25.5, 1.5, 33.3);
        let flipped_once = flip_point_orientation(&point, 100.0, 60.0);
        let flipped_twice = flip_point_orientation(&flipped_once, 100.0, 60.0);

        assert!((flipped_twice.x.get::<meter>() - point.x.get::<meter>()).abs() < 0.001);
        assert!((flipped_twice.y.get::<meter>() - point.y.get::<meter>()).abs() < 0.001);
        assert!((flipped_twice.z.get::<meter>() - point.z.get::<meter>()).abs() < 0.001);
    }
}
