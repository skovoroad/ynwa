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
    ///
    /// # Examples
    /// ```
    /// # use ynwa_core::GridCell;
    /// let cell_a = GridCell::from_literal("A", 1).unwrap();  // col=1
    /// let cell_z = GridCell::from_literal("Z", 1).unwrap();  // col=26
    /// let cell_aa = GridCell::from_literal("AA", 1).unwrap(); // col=27
    /// ```
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
    ///
    /// # Examples
    /// ```
    /// # use ynwa_core::GridCell;
    /// let cell = GridCell::from_notation("A1").unwrap();
    /// assert_eq!(cell.col, 1);
    /// assert_eq!(cell.row, 1);
    ///
    /// let cell2 = GridCell::from_notation("AA10").unwrap();
    /// assert_eq!(cell2.col, 27);
    /// assert_eq!(cell2.row, 10);
    /// ```
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

    /// Flips the cell orientation for the opposite team.
    pub fn flip_orientation(&self, grid_dims: GridDimensions) -> Result<Self, RegionError> {
        // Flip both column and row
        let new_col = grid_dims.columns - self.col + 1;
        let new_row = grid_dims.rows - self.row + 1;

        Self::new(new_col, new_row)
    }

    /// Converts a 1-based column number to Excel-style label (A, B, ..., Z, AA, AB, ...).
    ///
    /// # Examples
    /// ```
    /// # use ynwa_core::GridCell;
    /// assert_eq!(GridCell::column_to_label(1), "A");
    /// assert_eq!(GridCell::column_to_label(26), "Z");
    /// assert_eq!(GridCell::column_to_label(27), "AA");
    /// ```
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

    /// Flips the region orientation for the opposite team.
    /// Returns a new region with both corners flipped.
    pub fn flip_orientation(&self, grid_dims: GridDimensions) -> Result<Self, RegionError> {
        let new_top_left = self.top_left.flip_orientation(grid_dims)?;
        let new_bottom_right = self.bottom_right.flip_orientation(grid_dims)?;

        // Swap corners to maintain top_left <= bottom_right after flip
        Region::new(
            self.team.opposite(),
            new_bottom_right,
            new_top_left,
            grid_dims,
        )
    }

    /// Convert region to grid notation string (e.g., "A1:B2")
    /// This format is used for human-readable configuration files.
    ///
    /// # Examples
    /// ```
    /// # use ynwa_core::{Region, GridCell, GridDimensions, team::Team};
    /// let grid_dims = GridDimensions::new(26, 44);
    /// let region = Region::new(
    ///     Team::A,
    ///     GridCell::new(1, 1).unwrap(),
    ///     GridCell::new(2, 2).unwrap(),
    ///     grid_dims
    /// ).unwrap();
    /// assert_eq!(region.to_grid_notation(), "A1:B2");
    /// ```
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
    /// # Examples
    /// ```
    /// # use ynwa_core::{Region, GridDimensions, team::Team};
    /// let grid_dims = GridDimensions::new(26, 44);
    ///
    /// // Two-cell notation
    /// let region = Region::from_grid_notation("C3:D4", Team::A, grid_dims).unwrap();
    /// assert_eq!(region.top_left.col, 3);
    /// assert_eq!(region.top_left.row, 3);
    ///
    /// // Single-cell notation
    /// let region = Region::from_grid_notation("M42", Team::B, grid_dims).unwrap();
    /// assert_eq!(region.top_left.col, 13);
    /// assert_eq!(region.top_left.row, 42);
    /// assert_eq!(region.bottom_right.col, 13);
    /// assert_eq!(region.bottom_right.row, 42);
    /// ```
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
mod tests {
    use super::*;
    use crate::field::Field;

    #[test]
    fn test_grid_cell_new() {
        let cell = GridCell::new(1, 3).unwrap();
        assert_eq!(cell.col, 1);
        assert_eq!(cell.row, 3);
    }

    #[test]
    fn test_grid_cell_zero_row() {
        let result = GridCell::new(0, 0);
        assert!(matches!(result, Err(RegionError::InvalidRow(0))));
    }

    #[test]
    fn test_grid_cell_from_literal_single_letter() {
        let cell_a = GridCell::from_literal("A", 1).unwrap();
        assert_eq!(cell_a.col, 1); // A=1
        assert_eq!(cell_a.row, 1);

        let cell_b = GridCell::from_literal("B", 2).unwrap();
        assert_eq!(cell_b.col, 2); // B=2

        let cell_z = GridCell::from_literal("Z", 1).unwrap();
        assert_eq!(cell_z.col, 26); // Z=26
    }

    #[test]
    fn test_grid_cell_from_literal_lowercase() {
        let cell = GridCell::from_literal("b", 3).unwrap();
        assert_eq!(cell.col, 2); // b -> B = 2
        assert_eq!(cell.row, 3);
    }

    #[test]
    fn test_grid_cell_from_literal_multi_letter() {
        let cell_aa = GridCell::from_literal("AA", 1).unwrap();
        assert_eq!(cell_aa.col, 27); // Z=26, AA=27

        let cell_ab = GridCell::from_literal("AB", 1).unwrap();
        assert_eq!(cell_ab.col, 28); // AB=28

        let cell_az = GridCell::from_literal("AZ", 1).unwrap();
        assert_eq!(cell_az.col, 52); // 26*2 = 52
    }

    #[test]
    fn test_grid_cell_from_literal_case_insensitive() {
        let cell_lower = GridCell::from_literal("ab", 1).unwrap();
        let cell_upper = GridCell::from_literal("AB", 1).unwrap();
        assert_eq!(cell_lower.col, cell_upper.col);
    }

    #[test]
    fn test_grid_cell_from_literal_empty() {
        let result = GridCell::from_literal("", 1);
        assert!(matches!(result, Err(RegionError::EmptyColumnLabel)));
    }

    #[test]
    fn test_grid_cell_from_literal_invalid() {
        let result = GridCell::from_literal("A1", 1);
        assert!(matches!(result, Err(RegionError::InvalidColumnLabel(_))));
    }

    #[test]
    fn test_grid_cell_from_notation() {
        let cell = GridCell::from_notation("A1").unwrap();
        assert_eq!(cell.col, 1);
        assert_eq!(cell.row, 1);

        let cell2 = GridCell::from_notation("B2").unwrap();
        assert_eq!(cell2.col, 2);
        assert_eq!(cell2.row, 2);

        let cell3 = GridCell::from_notation("Z10").unwrap();
        assert_eq!(cell3.col, 26);
        assert_eq!(cell3.row, 10);

        let cell4 = GridCell::from_notation("AA27").unwrap();
        assert_eq!(cell4.col, 27);
        assert_eq!(cell4.row, 27);
    }

    #[test]
    fn test_grid_cell_from_notation_case_insensitive() {
        let lower = GridCell::from_notation("ab10").unwrap();
        let upper = GridCell::from_notation("AB10").unwrap();
        assert_eq!(lower.col, upper.col);
        assert_eq!(lower.row, upper.row);
    }

    #[test]
    fn test_grid_cell_from_notation_invalid() {
        // Letters after digits
        assert!(GridCell::from_notation("1A").is_err());
        assert!(GridCell::from_notation("A1B").is_err());

        // Missing parts
        assert!(GridCell::from_notation("A").is_err());
        assert!(GridCell::from_notation("1").is_err());
        assert!(GridCell::from_notation("").is_err());

        // Invalid characters
        assert!(GridCell::from_notation("A-1").is_err());
        assert!(GridCell::from_notation("A 1").is_err());
    }

    #[test]
    fn test_column_to_label() {
        assert_eq!(GridCell::column_to_label(1), "A");
        assert_eq!(GridCell::column_to_label(2), "B");
        assert_eq!(GridCell::column_to_label(26), "Z");
        assert_eq!(GridCell::column_to_label(27), "AA");
        assert_eq!(GridCell::column_to_label(28), "AB");
        assert_eq!(GridCell::column_to_label(52), "AZ");
        assert_eq!(GridCell::column_to_label(53), "BA");
        assert_eq!(GridCell::column_to_label(702), "ZZ");
        assert_eq!(GridCell::column_to_label(703), "AAA");
    }

    #[test]
    fn test_grid_cell_flip_orientation() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);

        let cell = GridCell::new(1, 1).unwrap(); // A1 (col=1)
        let flipped = cell.flip_orientation(field.grid_dimensions()).unwrap();

        // Should become Z44 (col=26)
        assert_eq!(flipped.col, 26); // Z = 26
        assert_eq!(flipped.row, 44);
    }

    #[test]
    fn test_region_flip_orientation() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);

        // Region for Team A: B3 to D5 (cols 2-4, rows 3-5)
        let region_a = Region::new(
            Team::A,
            GridCell::from_literal("B", 3).unwrap(),
            GridCell::from_literal("D", 5).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        let flipped = region_a.flip_orientation(field.grid_dimensions()).unwrap();

        // Should flip to Team B
        assert_eq!(flipped.team, Team::B);

        // Corners should be flipped:
        // B (col=2) -> Y (col=25), D (col=4) -> W (col=23)
        // Row 3 -> 42, Row 5 -> 40
        assert_eq!(flipped.top_left.col, 23); // W
        assert_eq!(flipped.top_left.row, 40);
        assert_eq!(flipped.bottom_right.col, 25); // Y
        assert_eq!(flipped.bottom_right.row, 42);
    }

    #[test]
    fn test_region_valid() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3).unwrap(),
            GridCell::from_literal("G", 4).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();
        assert_eq!(region.team, Team::A);
        assert_eq!(region.top_left.col, 2); // B = 2
        assert_eq!(region.bottom_right.col, 7); // G = 7
    }

    #[test]
    fn test_region_inverted_columns() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let result = Region::new(
            Team::A,
            GridCell::from_literal("G", 3).unwrap(),
            GridCell::from_literal("B", 4).unwrap(),
            field.grid_dimensions(),
        );
        assert!(matches!(result, Err(RegionError::InvalidRegion(_))));
    }

    #[test]
    fn test_region_inverted_rows() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let result = Region::new(
            Team::A,
            GridCell::from_literal("B", 4).unwrap(),
            GridCell::from_literal("G", 3).unwrap(),
            field.grid_dimensions(),
        );
        assert!(matches!(result, Err(RegionError::InvalidRegion(_))));
    }

    #[test]
    fn test_region_single_cell() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::B,
            GridCell::from_literal("C", 5).unwrap(),
            GridCell::from_literal("C", 5).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();
        assert_eq!(region.top_left, region.bottom_right);
    }

    #[test]
    fn test_region_column_out_of_bounds() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let result = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(27, 5).unwrap(), // Column 27 > 26 (max for 26 columns)
            field.grid_dimensions(),
        );
        assert!(matches!(result, Err(RegionError::ColumnOutOfBounds { .. })));
    }

    #[test]
    fn test_region_row_out_of_bounds() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let result = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(5, 50).unwrap(), // Row 50 > 44 (field has 44 rows)
            field.grid_dimensions(),
        );
        assert!(matches!(result, Err(RegionError::RowOutOfBounds { .. })));
    }

    #[test]
    fn test_region_contains_point() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);

        // Region B3:D5 (B=2, D=4, so cols 2-4; rows 3-5)
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3).unwrap(),
            GridCell::from_literal("D", 5).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        // Point in the middle of cell C4 should be inside
        let cell_width = 60.0 / 26.0;
        let point_inside = Point3D::from_meters(
            (3.5) * cell_width, // Row 4 center
            0.0,
            (2.5) * cell_width, // Column C (col=3) center
        );
        assert!(region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_inside
        ));

        // Point outside the region
        let point_outside = Point3D::from_meters(1.0, 0.0, 1.0);
        assert!(!region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_outside
        ));
    }

    #[test]
    fn test_region_contains_point_boundaries() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3).unwrap(),
            GridCell::from_literal("D", 5).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        let cell_width = 60.0 / 26.0;

        // Point exactly at min_x, min_z (should be inside, inclusive)
        let point_min = Point3D::from_meters(
            (3.0 - 1.0) * cell_width, // Row 3 min
            0.0,
            (2.0 - 1.0) * cell_width, // Col B min
        );
        assert!(region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_min
        ));

        // Point exactly at max_x boundary (should be outside, exclusive)
        let point_max_x = Point3D::from_meters(
            5.0 * cell_width, // Row 5 max (exclusive)
            0.0,
            2.5 * cell_width,
        );
        assert!(!region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_max_x
        ));

        // Point exactly at max_z boundary (should be outside, exclusive)
        let point_max_z = Point3D::from_meters(
            3.5 * cell_width,
            0.0,
            4.0 * cell_width, // Col D max (exclusive)
        );
        assert!(!region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_max_z
        ));

        // Point just inside boundaries
        let point_inside_edge = Point3D::from_meters(
            (3.0 - 1.0) * cell_width + 0.001,
            0.0,
            (2.0 - 1.0) * cell_width + 0.001,
        );
        assert!(region.contains_point(
            field.grid_dimensions(),
            field.width().get::<meter>(),
            &point_inside_edge
        ));
    }

    #[test]
    fn test_region_center() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);

        // Region B3:D5 (B=2, D=4, so cols 2-4; rows 3-5)
        // Center: col 3 (C), row 4
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3).unwrap(),
            GridCell::from_literal("D", 5).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        let center = region.center(field.grid_dimensions(), field.width().get::<meter>());

        let cell_width = 60.0 / 26.0;
        let expected_z = (2.0 + 0.5) * cell_width; // Col 2 center
        let expected_x = (4.0 - 0.5) * cell_width; // Row 4 center

        assert!((center.x.get::<meter>() - expected_x).abs() < 0.01);
        assert!((center.z.get::<meter>() - expected_z).abs() < 0.01);
        assert_eq!(center.y.get::<meter>(), 0.0);
    }

    #[test]
    fn test_region_center_single_cell() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);

        let region = Region::new(
            Team::A,
            GridCell::from_literal("A", 1).unwrap(),
            GridCell::from_literal("A", 1).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        let center = region.center(field.grid_dimensions(), field.width().get::<meter>());

        let cell_width = 60.0 / 26.0;
        let expected_z = 0.5 * cell_width; // Col 0 center
        let expected_x = 0.5 * cell_width; // Row 1 center

        assert!((center.x.get::<meter>() - expected_x).abs() < 0.01);
        assert!((center.z.get::<meter>() - expected_z).abs() < 0.01);
    }

    #[test]
    fn test_region_to_grid_notation() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        assert_eq!(region.to_grid_notation(), "A1:B2");

        // Test with multi-letter columns
        let region2 = Region::new(
            Team::B,
            GridCell::new(25, 22).unwrap(),
            GridCell::new(26, 24).unwrap(),
            field.grid_dimensions(),
        )
        .unwrap();

        assert_eq!(region2.to_grid_notation(), "Y22:Z24");
    }

    #[test]
    fn test_region_from_grid_notation() {
        let grid_dims = GridDimensions::new(26, 44);

        // Two-cell notation
        let region = Region::from_grid_notation("A1:B2", Team::A, grid_dims).unwrap();
        assert_eq!(region.team, Team::A);
        assert_eq!(region.top_left.col, 1);
        assert_eq!(region.top_left.row, 1);
        assert_eq!(region.bottom_right.col, 2);
        assert_eq!(region.bottom_right.row, 2);

        // Test with multi-letter columns
        let region2 = Region::from_grid_notation("Y22:Z24", Team::B, grid_dims).unwrap();
        assert_eq!(region2.top_left.col, 25);
        assert_eq!(region2.top_left.row, 22);
        assert_eq!(region2.bottom_right.col, 26);
        assert_eq!(region2.bottom_right.row, 24);

        // Single-cell notation
        let region3 = Region::from_grid_notation("M42", Team::A, grid_dims).unwrap();
        assert_eq!(region3.top_left.col, 13);
        assert_eq!(region3.top_left.row, 42);
        assert_eq!(region3.bottom_right.col, 13);
        assert_eq!(region3.bottom_right.row, 42);
    }

    #[test]
    fn test_region_grid_notation_roundtrip() {
        let grid_dims = GridDimensions::new(26, 44);

        let original = Region::new(
            Team::A,
            GridCell::new(3, 5).unwrap(),
            GridCell::new(7, 10).unwrap(),
            grid_dims,
        )
        .unwrap();

        let notation = original.to_grid_notation();
        let parsed = Region::from_grid_notation(&notation, Team::A, grid_dims).unwrap();

        assert_eq!(original.top_left, parsed.top_left);
        assert_eq!(original.bottom_right, parsed.bottom_right);
        assert_eq!(original.team, parsed.team);
    }

    #[test]
    fn test_region_from_grid_notation_invalid() {
        let grid_dims = GridDimensions::new(26, 44);

        // Invalid format
        assert!(Region::from_grid_notation("A1B2", Team::A, grid_dims).is_err());
        assert!(Region::from_grid_notation("A1:", Team::A, grid_dims).is_err());
        assert!(Region::from_grid_notation(":B2", Team::A, grid_dims).is_err());

        // Invalid cells
        assert!(Region::from_grid_notation("A:B2", Team::A, grid_dims).is_err());
        assert!(Region::from_grid_notation("1A:B2", Team::A, grid_dims).is_err());
        assert!(Region::from_grid_notation("A1:B", Team::A, grid_dims).is_err());
    }
}
