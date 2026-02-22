use crate::field::zones::Point3D;
use crate::orientation::{flip_grid_cell_orientation, flip_point_orientation, flip_region_orientation};
use crate::region::{GridCell, GridDimensions, Region};
use crate::team::Team;
use uom::si::length::meter;

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
