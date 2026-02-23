use crate::field::{zones::Point3D, Field};
use crate::region::{GridCell, GridDimensions, Region, RegionError};
use uom::si::length::meter;

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
    let region_a = field.grid_dimensions().create_region(GridCell::from_literal("B", 3).unwrap(), GridCell::from_literal("D", 5).unwrap())
    .unwrap();

    let flipped = region_a.flip_orientation(field.grid_dimensions()).unwrap();

    // Should flip to Team B

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
    let region = field.grid_dimensions().create_region(GridCell::from_literal("B", 3).unwrap(), GridCell::from_literal("G", 4).unwrap())
    .unwrap();
    assert_eq!(region.top_left.col, 2); // B = 2
    assert_eq!(region.bottom_right.col, 7); // G = 7
}

#[test]
fn test_region_inverted_columns() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let result = field.grid_dimensions().create_region(GridCell::from_literal("G", 3).unwrap(), GridCell::from_literal("B", 4).unwrap());
    assert!(matches!(result, Err(RegionError::InvalidRegion(_))));
}

#[test]
fn test_region_inverted_rows() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let result = field.grid_dimensions().create_region(GridCell::from_literal("B", 4).unwrap(), GridCell::from_literal("G", 3).unwrap());
    assert!(matches!(result, Err(RegionError::InvalidRegion(_))));
}

#[test]
fn test_region_single_cell() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let region = field.grid_dimensions().create_region(GridCell::from_literal("C", 5).unwrap(), GridCell::from_literal("C", 5).unwrap())
    .unwrap();
    assert_eq!(region.top_left, region.bottom_right);
}

#[test]
fn test_region_column_out_of_bounds() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let result = field.grid_dimensions().create_region(
        GridCell::new(1, 1).unwrap(),
        GridCell::new(27, 5).unwrap(), // Column 27 > 26 (max for 26 columns)
    );
    assert!(matches!(result, Err(RegionError::ColumnOutOfBounds { .. })));
}

#[test]
fn test_region_row_out_of_bounds() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let result = field.grid_dimensions().create_region(
        GridCell::new(1, 1).unwrap(),
        GridCell::new(5, 50).unwrap(), // Row 50 > 44 (field has 44 rows)
    );
    assert!(matches!(result, Err(RegionError::RowOutOfBounds { .. })));
}

#[test]
fn test_region_contains_point() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);

    // Region B3:D5 (B=2, D=4, so cols 2-4; rows 3-5)
    let region = field.grid_dimensions().create_region(GridCell::from_literal("B", 3).unwrap(), GridCell::from_literal("D", 5).unwrap())
    .unwrap();

    // Point in the middle of cell C4 should be inside
    let cell_width = 60.0 / 26.0;
    let point_inside = Point3D::from_meters(
        (2.5) * cell_width, // Column C (col=3) center → X
        0.0,
        (3.5) * cell_width, // Row 4 center → Z
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
    let region = field.grid_dimensions().create_region(GridCell::from_literal("B", 3).unwrap(), GridCell::from_literal("D", 5).unwrap())
    .unwrap();

    let cell_width = 60.0 / 26.0;

    // Point exactly at min_x, min_z (should be inside, inclusive)
    let point_min = Point3D::from_meters(
        (2.0 - 1.0) * cell_width, // Col B (col=2) min → X
        0.0,
        (3.0 - 1.0) * cell_width, // Row 3 min → Z
    );
    assert!(region.contains_point(
        field.grid_dimensions(),
        field.width().get::<meter>(),
        &point_min
    ));

    // Point exactly at max_x boundary (should be outside, exclusive)
    let point_max_x = Point3D::from_meters(
        4.0 * cell_width, // Col D (col=4) max (exclusive) → X
        0.0,
        3.5 * cell_width,
    );
    assert!(!region.contains_point(
        field.grid_dimensions(),
        field.width().get::<meter>(),
        &point_max_x
    ));

    // Point exactly at max_z boundary (should be outside, exclusive)
    let point_max_z = Point3D::from_meters(
        2.5 * cell_width,
        0.0,
        5.0 * cell_width, // Row 5 max (exclusive) → Z
    );
    assert!(!region.contains_point(
        field.grid_dimensions(),
        field.width().get::<meter>(),
        &point_max_z
    ));

    // Point just inside boundaries
    let point_inside_edge = Point3D::from_meters(
        (2.0 - 1.0) * cell_width + 0.001,
        0.0,
        (3.0 - 1.0) * cell_width + 0.001,
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
    let region = field.grid_dimensions().create_region(GridCell::from_literal("B", 3).unwrap(), GridCell::from_literal("D", 5).unwrap())
    .unwrap();

    let center = region.center(field.grid_dimensions(), field.width().get::<meter>());

    let cell_width = 60.0 / 26.0;
    // col → X: cols B(2)..D(4): min=(2-1)*cw=1*cw, max=4*cw, center=2.5*cw
    // row → Z: rows 3..5:       min=(3-1)*cw=2*cw, max=5*cw, center=3.5*cw
    let expected_x = 2.5 * cell_width;
    let expected_z = 3.5 * cell_width;

    assert!((center.x.get::<meter>() - expected_x).abs() < 0.01);
    assert!((center.z.get::<meter>() - expected_z).abs() < 0.01);
    assert_eq!(center.y.get::<meter>(), 0.0);
}

#[test]
fn test_region_center_single_cell() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);

    let region = field.grid_dimensions().create_region(GridCell::from_literal("A", 1).unwrap(), GridCell::from_literal("A", 1).unwrap())
    .unwrap();

    let center = region.center(field.grid_dimensions(), field.width().get::<meter>());

    let cell_width = 60.0 / 26.0;
    let expected_x = 0.5 * cell_width; // Col A (col=1) center → X
    let expected_z = 0.5 * cell_width; // Row 1 center → Z

    assert!((center.x.get::<meter>() - expected_x).abs() < 0.01);
    assert!((center.z.get::<meter>() - expected_z).abs() < 0.01);
}

#[test]
fn test_region_to_grid_notation() {
    let field = Field::from_meters(60.0, 100.0, 26, 44);
    let region = field.grid_dimensions().create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
    .unwrap();

    assert_eq!(region.to_grid_notation(), "A1:B2");

    // Test with multi-letter columns
    let region2 = field.grid_dimensions().create_region(GridCell::new(25, 22).unwrap(), GridCell::new(26, 24).unwrap())
    .unwrap();

    assert_eq!(region2.to_grid_notation(), "Y22:Z24");
}

#[test]
fn test_region_from_grid_notation() {
    let grid_dims = GridDimensions::new(26, 44);

    // Two-cell notation
    let region = Region::from_grid_notation("A1:B2", grid_dims).unwrap();
    assert_eq!(region.top_left.col, 1);
    assert_eq!(region.top_left.row, 1);
    assert_eq!(region.bottom_right.col, 2);
    assert_eq!(region.bottom_right.row, 2);

    // Test with multi-letter columns
    let region2 = Region::from_grid_notation("Y22:Z24", grid_dims).unwrap();
    assert_eq!(region2.top_left.col, 25);
    assert_eq!(region2.top_left.row, 22);
    assert_eq!(region2.bottom_right.col, 26);
    assert_eq!(region2.bottom_right.row, 24);

    // Single-cell notation
    let region3 = Region::from_grid_notation("M42", grid_dims).unwrap();
    assert_eq!(region3.top_left.col, 13);
    assert_eq!(region3.top_left.row, 42);
    assert_eq!(region3.bottom_right.col, 13);
    assert_eq!(region3.bottom_right.row, 42);
}

#[test]
fn test_region_grid_notation_roundtrip() {
    let grid_dims = GridDimensions::new(26, 44);

    let original = grid_dims.create_region(GridCell::new(3, 5).unwrap(), GridCell::new(7, 10).unwrap())
    .unwrap();

    let notation = original.to_grid_notation();
    let parsed = Region::from_grid_notation(&notation, grid_dims).unwrap();

    assert_eq!(original.top_left, parsed.top_left);
    assert_eq!(original.bottom_right, parsed.bottom_right);
}

#[test]
fn test_region_from_grid_notation_invalid() {
    let grid_dims = GridDimensions::new(26, 44);

    // Invalid format
    assert!(Region::from_grid_notation("A1B2", grid_dims).is_err());
    assert!(Region::from_grid_notation("A1:", grid_dims).is_err());
    assert!(Region::from_grid_notation(":B2", grid_dims).is_err());

    // Invalid cells
    assert!(Region::from_grid_notation("A:B2", grid_dims).is_err());
    assert!(Region::from_grid_notation("1A:B2", grid_dims).is_err());
    assert!(Region::from_grid_notation("A1:B", grid_dims).is_err());
}
