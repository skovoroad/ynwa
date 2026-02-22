//! Coordinate orientation transformations for team perspectives.
//!
//! The field has a canonical "display orientation" (Team A's perspective: left-to-right).
//! Team B plays from the opposite side (right-to-left in their perspective).
//! These functions convert coordinates between team perspectives and display orientation.
//!
//! Applied at system boundaries in `ScriptedDecisionMaker`:
//! - Input: context for Team B has flipped coordinates (scripts see field from same side as Team A)
//! - Output: decisions from Team B are flipped back to display orientation

use crate::field::zones::Point3D;
use crate::region::{GridCell, GridDimensions, Region, RegionError};
#[allow(unused_imports)] // Used in doctests
use crate::team::Team;
use uom::si::f32::Length;
use uom::si::length::meter;

/// Flips a grid cell's orientation for the opposite team.
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
pub fn flip_point_orientation(point: &Point3D, field_width: f32, field_length: f32) -> Point3D {
    Point3D {
        x: Length::new::<meter>(field_width - point.x.get::<meter>()),
        y: point.y, // height doesn't change
        z: Length::new::<meter>(field_length - point.z.get::<meter>()),
    }
}

#[cfg(test)]
#[path = "tests/orientation_tests.rs"]
mod tests;
