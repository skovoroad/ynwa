use crate::team::Team;
use crate::field::Field;
use crate::field::zones::Point3D;
use uom::si::length::meter;

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
    pub fn new(col: u32, row: u32) -> Self {
        assert!(row > 0, "Row must be positive");
        Self { col, row }
    }

    /// Creates a grid cell from string notation (A, B, ..., Z, AA, AB, ...).
    /// Case-insensitive.
    /// 
    /// # Examples
    /// ```
    /// # use ynwa_core::GridCell;
    /// let cell_a = GridCell::from_literal("A", 1);  // col=1
    /// let cell_z = GridCell::from_literal("Z", 1);  // col=26
    /// let cell_aa = GridCell::from_literal("AA", 1); // col=27
    /// ```
    pub fn from_literal(label: &str, row: u32) -> Self {
        assert!(!label.is_empty(), "Column label must not be empty");
        assert!(row > 0, "Row must be positive");
        
        let mut col: u32 = 0;
        for ch in label.chars() {
            let ch_upper = ch.to_ascii_uppercase();
            assert!(ch_upper.is_ascii_uppercase(), 
                    "Column label must contain only letters A-Z");
            
            // Column encoding: A=1, B=2, ..., Z=26, AA=27
            col = col * 26 + (ch_upper as u32 - 'A' as u32 + 1);
        }
        
        Self::new(col, row)
    }

    /// Flips the cell orientation for the opposite team.
    pub fn flip_orientation(&self, field: &Field) -> Self {
        let grid_cols = field.grid_columns();
        let grid_rows = field.grid_rows();
        
        // Flip both column and row
        let new_col = grid_cols - self.col + 1;
        let new_row = grid_rows - self.row + 1;
        
        Self::new(new_col, new_row)
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
    /// Creates a region with validation that cells are within field grid bounds.
    pub fn new(team: Team, top_left: GridCell, bottom_right: GridCell, field: &Field) -> Self {
        assert!(top_left.col <= bottom_right.col,
                "Top-left column must be <= bottom-right column");
        assert!(top_left.row <= bottom_right.row,
                "Top-left row must be <= bottom-right row");
        
        // Validate cells are within field grid bounds (1-based)
        assert!(top_left.col > 0 && top_left.col <= field.grid_columns(),
                "Top-left column {} must be between 1 and {}", 
                top_left.col, field.grid_columns());
        assert!(bottom_right.col > 0 && bottom_right.col <= field.grid_columns(),
                "Bottom-right column {} must be between 1 and {}", 
                bottom_right.col, field.grid_columns());
        assert!(top_left.row > 0 && top_left.row <= field.grid_rows(),
                "Top-left row {} must be between 1 and {}", 
                top_left.row, field.grid_rows());
        assert!(bottom_right.row > 0 && bottom_right.row <= field.grid_rows(),
                "Bottom-right row {} must be between 1 and {}", 
                bottom_right.row, field.grid_rows());
        
        Self {
            team,
            top_left,
            bottom_right,
        }
    }

    /// Checks if a point falls within this region (ignores Y/height).
    pub fn contains_point(&self, field: &Field, point: &Point3D) -> bool {
        let field_width = field.width().get::<meter>();
        let grid_cols = field.grid_columns();
        let cell_width = field_width / grid_cols as f32;
        
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
    pub fn center(&self, field: &Field) -> Point3D {
        let field_width = field.width().get::<meter>();
        let grid_cols = field.grid_columns();
        let cell_width = field_width / grid_cols as f32;
        
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;

    #[test]
    fn test_grid_cell_new() {
        let cell = GridCell::new(1, 3);
        assert_eq!(cell.col, 1);
        assert_eq!(cell.row, 3);
    }

    #[test]
    #[should_panic(expected = "Row must be positive")]
    fn test_grid_cell_zero_row() {
        GridCell::new(0, 0);
    }

    #[test]
    fn test_grid_cell_from_literal_single_letter() {
        let cell_a = GridCell::from_literal("A", 1);
        assert_eq!(cell_a.col, 1); // A=1
        assert_eq!(cell_a.row, 1);

        let cell_b = GridCell::from_literal("B", 2);
        assert_eq!(cell_b.col, 2); // B=2

        let cell_z = GridCell::from_literal("Z", 1);
        assert_eq!(cell_z.col, 26); // Z=26
    }

    #[test]
    fn test_grid_cell_from_literal_lowercase() {
        let cell = GridCell::from_literal("b", 3);
        assert_eq!(cell.col, 2); // b -> B = 2
        assert_eq!(cell.row, 3);
    }

    #[test]
    fn test_grid_cell_from_literal_multi_letter() {
        let cell_aa = GridCell::from_literal("AA", 1);
        assert_eq!(cell_aa.col, 27); // Z=26, AA=27

        let cell_ab = GridCell::from_literal("AB", 1);
        assert_eq!(cell_ab.col, 28); // AB=28

        let cell_az = GridCell::from_literal("AZ", 1);
        assert_eq!(cell_az.col, 52); // 26*2 = 52
    }

    #[test]
    fn test_grid_cell_from_literal_case_insensitive() {
        let cell_lower = GridCell::from_literal("ab", 1);
        let cell_upper = GridCell::from_literal("AB", 1);
        assert_eq!(cell_lower.col, cell_upper.col);
    }

    #[test]
    #[should_panic(expected = "Column label must not be empty")]
    fn test_grid_cell_from_literal_empty() {
        GridCell::from_literal("", 1);
    }

    #[test]
    #[should_panic(expected = "Column label must contain only letters A-Z")]
    fn test_grid_cell_from_literal_invalid() {
        GridCell::from_literal("A1", 1);
    }

    #[test]
    fn test_grid_cell_flip_orientation() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        
        let cell = GridCell::new(1, 1); // A1 (col=1)
        let flipped = cell.flip_orientation(&field);
        
        // Should become Z44 (col=26)
        assert_eq!(flipped.col, 26); // Z = 26
        assert_eq!(flipped.row, 44);
    }

    #[test]
    fn test_region_valid() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3),
            GridCell::from_literal("G", 4),
            &field,
        );
        assert_eq!(region.team, Team::A);
        assert_eq!(region.top_left.col, 2); // B = 2
        assert_eq!(region.bottom_right.col, 7); // G = 7
    }

    #[test]
    #[should_panic(expected = "Top-left column must be <= bottom-right column")]
    fn test_region_inverted_columns() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        Region::new(
            Team::A,
            GridCell::from_literal("G", 3),
            GridCell::from_literal("B", 4),
            &field,
        );
    }

    #[test]
    #[should_panic(expected = "Top-left row must be <= bottom-right row")]
    fn test_region_inverted_rows() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        Region::new(
            Team::A,
            GridCell::from_literal("B", 4),
            GridCell::from_literal("G", 3),
            &field,
        );
    }

    #[test]
    fn test_region_single_cell() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let region = Region::new(
            Team::B,
            GridCell::from_literal("C", 5),
            GridCell::from_literal("C", 5),
            &field,
        );
        assert_eq!(region.top_left, region.bottom_right);
    }

    #[test]
    #[should_panic(expected = "must be between 1 and")]
    fn test_region_column_out_of_bounds() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        Region::new(
            Team::A,
            GridCell::new(1, 1),
            GridCell::new(27, 5), // Column 27 > 26 (max for 26 columns)
            &field,
        );
    }

    #[test]
    #[should_panic(expected = "must be between 1 and")]
    fn test_region_row_out_of_bounds() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        Region::new(
            Team::A,
            GridCell::new(1, 1),
            GridCell::new(5, 50), // Row 50 > 44 (field has 44 rows)
            &field,
        );
    }

    #[test]
    fn test_region_contains_point() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        
        // Region B3:D5 (B=2, D=4, so cols 2-4; rows 3-5)
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3),
            GridCell::from_literal("D", 5),
            &field,
        );
        
        // Point in the middle of cell C4 should be inside
        let cell_width = 60.0 / 26.0;
        let point_inside = Point3D::from_meters(
            (3.5) * cell_width,  // Row 4 center
            0.0,
            (2.5) * cell_width,  // Column C (col=3) center
        );
        assert!(region.contains_point(&field, &point_inside));
        
        // Point outside the region
        let point_outside = Point3D::from_meters(1.0, 0.0, 1.0);
        assert!(!region.contains_point(&field, &point_outside));
    }

    #[test]
    fn test_region_center() {
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        
        // Region B3:D5 (B=2, D=4, so cols 2-4; rows 3-5)
        // Center: col 3 (C), row 4
        let region = Region::new(
            Team::A,
            GridCell::from_literal("B", 3),
            GridCell::from_literal("D", 5),
            &field,
        );
        
        let center = region.center(&field);
        
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
            GridCell::from_literal("A", 1),
            GridCell::from_literal("A", 1),
            &field,
        );
        
        let center = region.center(&field);
        
        let cell_width = 60.0 / 26.0;
        let expected_z = 0.5 * cell_width; // Col 0 center
        let expected_x = 0.5 * cell_width; // Row 1 center
        
        assert!((center.x.get::<meter>() - expected_x).abs() < 0.01);
        assert!((center.z.get::<meter>() - expected_z).abs() < 0.01);
    }
}
