//! Field area addressing through a grid coordinate system.
//!
//! Key types:
//! - `GridDimensions { columns, rows }` - grid size
//! - `GridCell { col, row }` - single cell, 1-based (A=1, B=2, ..., Z=26, AA=27, ...)
//! - `Region { team, top_left, bottom_right }` - rectangular area
//!
//! Grid notation format: `"A1:B2"` (TopLeft:BottomRight)

use crate::field::zones::Point3D;
use crate::team::Team;
use std::fmt;
use uom::si::length::meter;

/// Grid dimensions for field regions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridDimensions {
    /// Number of columns (1-based: A=1, B=2, ..., Z=26, AA=27, etc.)
    pub columns: u32,
    /// Number of rows (1-based)
    pub rows: u32,
}

impl GridDimensions {
    /// Creates new grid dimensions
    pub fn new(columns: u32, rows: u32) -> Self {
        Self { columns, rows }
    }
}

/// Errors that can occur when creating or manipulating grid cells and regions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegionError {
    /// Row must be positive (1-based)
    InvalidRow(u32),
    /// Column must be positive (1-based)
    InvalidColumn(u32),
    /// Column label contains invalid characters
    InvalidColumnLabel(String),
    /// Column label is empty
    EmptyColumnLabel,
    /// Column exceeds field grid bounds
    ColumnOutOfBounds { col: u32, max: u32 },
    /// Row exceeds field grid bounds
    RowOutOfBounds { row: u32, max: u32 },
    /// Region has inverted corners or other invalid configuration
    InvalidRegion(String),
}

impl fmt::Display for RegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RegionError::InvalidRow(row) => write!(f, "Row must be positive, got {}", row),
            RegionError::InvalidColumn(col) => write!(f, "Column must be positive, got {}", col),
            RegionError::InvalidColumnLabel(label) => {
                write!(
                    f,
                    "Column label must contain only letters A-Z, got '{}'",
                    label
                )
            }
            RegionError::EmptyColumnLabel => write!(f, "Column label must not be empty"),
            RegionError::ColumnOutOfBounds { col, max } => {
                write!(f, "Column {} exceeds field grid columns {}", col, max)
            }
            RegionError::RowOutOfBounds { row, max } => {
                write!(f, "Row {} exceeds field grid rows {}", row, max)
            }
            RegionError::InvalidRegion(msg) => write!(f, "Invalid region: {}", msg),
        }
    }
}

impl std::error::Error for RegionError {}

/// A grid cell on the field, addressed by column (1-based) and row (1-based)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridCell {
    /// Column index (1-based: A=1, B=2, ..., Z=26, AA=27, etc.)
    pub col: u32,
    /// Row number (1-based)
    pub row: u32,
}

impl GridCell {
    /// Creates a new grid cell from numeric column and row indices.
    /// Both are 1-based.
    pub fn new(col: u32, row: u32) -> Result<Self, RegionError> {
        if row == 0 {
            return Err(RegionError::InvalidRow(row));
        }
        if col == 0 {
            return Err(RegionError::InvalidColumn(col));
        }
        Ok(Self { col, row })
    }

    /// Creates a grid cell from string notation (A, B, ..., Z, AA, AB, ...).
    /// Case-insensitive.
    pub fn from_literal(label: &str, row: u32) -> Result<Self, RegionError> {
        if label.is_empty() {
            return Err(RegionError::EmptyColumnLabel);
        }
        if row == 0 {
            return Err(RegionError::InvalidRow(row));
        }

        let mut col: u32 = 0;
        for ch in label.chars() {
            let ch_upper = ch.to_ascii_uppercase();
            if !ch_upper.is_ascii_uppercase() {
                return Err(RegionError::InvalidColumnLabel(label.to_string()));
            }

            // Column encoding: A=1, B=2, ..., Z=26, AA=27
            col = col * 26 + (ch_upper as u32 - 'A' as u32 + 1);
        }

        Self::new(col, row)
    }

    /// Parse cell notation like "A1", "B2", "AA10" into GridCell.
    /// Letters must come before digits.
    pub fn from_notation(notation: &str) -> Result<Self, RegionError> {
        let mut col_str = String::new();
        let mut row_str = String::new();
        let mut seen_digit = false;

        for ch in notation.chars() {
            if ch.is_ascii_alphabetic() {
                if seen_digit {
                    return Err(RegionError::InvalidRegion(format!(
                        "Letters must come before digits in cell notation '{}'",
                        notation
                    )));
                }
                col_str.push(ch);
            } else if ch.is_ascii_digit() {
                seen_digit = true;
                row_str.push(ch);
            } else {
                return Err(RegionError::InvalidRegion(format!(
                    "Invalid character '{}' in cell notation '{}'",
                    ch, notation
                )));
            }
        }

        if col_str.is_empty() {
            return Err(RegionError::InvalidRegion(format!(
                "Missing column in cell notation '{}'",
                notation
            )));
        }
        if row_str.is_empty() {
            return Err(RegionError::InvalidRegion(format!(
                "Missing row in cell notation '{}'",
                notation
            )));
        }

        let row = row_str.parse::<u32>().map_err(|_| {
            RegionError::InvalidRegion(format!(
                "Invalid row number '{}' in cell '{}'",
                row_str, notation
            ))
        })?;

        Self::from_literal(&col_str, row)
    }

    /// Convenience wrapper for [`crate::orientation::flip_grid_cell_orientation`].
    pub fn flip_orientation(&self, grid_dims: GridDimensions) -> Result<Self, RegionError> {
        crate::orientation::flip_grid_cell_orientation(self, grid_dims)
    }

    /// Converts a 1-based column number to Excel-style label (A, B, ..., Z, AA, AB, ...).
    pub fn column_to_label(col: u32) -> String {
        let mut result = String::new();
        let mut n = col;

        while n > 0 {
            let remainder = (n - 1) % 26;
            result.push((b'A' + remainder as u8) as char);
            n = (n - 1) / 26;
        }

        result.chars().rev().collect()
    }
}

/// Rectangular region on the field, defined by two grid cells.
/// Regions are team-specific because players view the field from different sides.
///
/// Example: Region from B3 to G4 (B=2, G=7; so cols 2-7, rows 3-4)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Region {
    /// The team this region is defined for (affects perspective)
    pub team: Team,
    /// Top-left corner of the region
    pub top_left: GridCell,
    /// Bottom-right corner of the region
    pub bottom_right: GridCell,
}

impl Region {
    /// Creates a region with validation that cells are within grid bounds.
    pub fn new(
        team: Team,
        top_left: GridCell,
        bottom_right: GridCell,
        grid_dims: GridDimensions,
    ) -> Result<Self, RegionError> {
        if top_left.col > bottom_right.col {
            return Err(RegionError::InvalidRegion(
                "Top-left column must be <= bottom-right column".to_string(),
            ));
        }
        if top_left.row > bottom_right.row {
            return Err(RegionError::InvalidRegion(
                "Top-left row must be <= bottom-right row".to_string(),
            ));
        }

        // Validate cells are within grid bounds (1-based)
        if top_left.col == 0 || top_left.col > grid_dims.columns {
            return Err(RegionError::ColumnOutOfBounds {
                col: top_left.col,
                max: grid_dims.columns,
            });
        }
        if bottom_right.col == 0 || bottom_right.col > grid_dims.columns {
            return Err(RegionError::ColumnOutOfBounds {
                col: bottom_right.col,
                max: grid_dims.columns,
            });
        }
        if top_left.row == 0 || top_left.row > grid_dims.rows {
            return Err(RegionError::RowOutOfBounds {
                row: top_left.row,
                max: grid_dims.rows,
            });
        }
        if bottom_right.row == 0 || bottom_right.row > grid_dims.rows {
            return Err(RegionError::RowOutOfBounds {
                row: bottom_right.row,
                max: grid_dims.rows,
            });
        }

        Ok(Self {
            team,
            top_left,
            bottom_right,
        })
    }

    /// Creates a region without validation. Use with caution.
    /// This is useful when parsing from user scripts where validation will happen later.
    pub fn new_unchecked(team: Team, top_left: GridCell, bottom_right: GridCell) -> Self {
        Self {
            team,
            top_left,
            bottom_right,
        }
    }

    /// Checks if a point falls within this region (ignores Y/height).
    pub fn contains_point(
        &self,
        grid_dims: GridDimensions,
        field_width_meters: f32,
        point: &Point3D,
    ) -> bool {
        let cell_width = field_width_meters / grid_dims.columns as f32;

        // Calculate region boundaries in meters (columns are 1-based)
        let min_z = (self.top_left.col - 1) as f32 * cell_width;
        let max_z = self.bottom_right.col as f32 * cell_width;
        let min_x = (self.top_left.row - 1) as f32 * cell_width;
        let max_x = self.bottom_right.row as f32 * cell_width;

        // Check if point is within boundaries
        let point_x = point.x.get::<meter>();
        let point_z = point.z.get::<meter>();

        point_x >= min_x && point_x < max_x && point_z >= min_z && point_z < max_z
    }

    /// Returns the center point of the region (Y=0, ground level).
    pub fn center(&self, grid_dims: GridDimensions, field_width_meters: f32) -> Point3D {
        let cell_width = field_width_meters / grid_dims.columns as f32;

        // Calculate region boundaries in meters (columns are 1-based)
        let min_z = (self.top_left.col - 1) as f32 * cell_width;
        let max_z = self.bottom_right.col as f32 * cell_width;
        let min_x = (self.top_left.row - 1) as f32 * cell_width;
        let max_x = self.bottom_right.row as f32 * cell_width;

        // Center is the midpoint of boundaries
        let center_x = (min_x + max_x) / 2.0;
        let center_z = (min_z + max_z) / 2.0;

        Point3D::from_meters(center_x, 0.0, center_z)
    }

    /// Convenience wrapper for [`crate::orientation::flip_region_orientation`].
    pub fn flip_orientation(&self, grid_dims: GridDimensions) -> Result<Self, RegionError> {
        crate::orientation::flip_region_orientation(self, grid_dims)
    }

    /// Convert region to grid notation string (e.g., "A1:B2")
    /// This format is used for human-readable configuration files.
    ///
    pub fn to_grid_notation(&self) -> String {
        format!(
            "{}{}:{}{}",
            GridCell::column_to_label(self.top_left.col),
            self.top_left.row,
            GridCell::column_to_label(self.bottom_right.col),
            self.bottom_right.row
        )
    }

    /// Parse region from grid notation string (e.g., "A1:B2" or "A1" for single cell)
    ///
    pub fn from_grid_notation(
        notation: &str,
        team: Team,
        grid_dims: GridDimensions,
    ) -> Result<Self, RegionError> {
        // Split by colon
        let parts: Vec<&str> = notation.split(':').collect();

        let (top_left, bottom_right) = match parts.len() {
            1 => {
                // Single cell notation: "A1"
                let cell = GridCell::from_notation(parts[0]).map_err(|e| {
                    RegionError::InvalidRegion(format!("Invalid cell '{}': {}", parts[0], e))
                })?;
                (cell, cell)
            }
            2 => {
                // Two-cell notation: "A1:B2"
                let top_left = GridCell::from_notation(parts[0]).map_err(|e| {
                    RegionError::InvalidRegion(format!(
                        "Invalid top-left cell '{}': {}",
                        parts[0], e
                    ))
                })?;

                let bottom_right = GridCell::from_notation(parts[1]).map_err(|e| {
                    RegionError::InvalidRegion(format!(
                        "Invalid bottom-right cell '{}': {}",
                        parts[1], e
                    ))
                })?;

                (top_left, bottom_right)
            }
            _ => {
                return Err(RegionError::InvalidRegion(format!(
                    "Invalid grid notation '{}'. Expected format: 'A1' or 'A1:B2'",
                    notation
                )));
            }
        };

        Region::new(team, top_left, bottom_right, grid_dims)
    }
}

#[cfg(test)]
#[path = "tests/region_tests.rs"]
mod tests;
