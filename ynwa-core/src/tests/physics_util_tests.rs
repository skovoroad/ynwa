use super::*;

#[test]
fn test_distance_zero() {
    let point = Point3D::from_meters(5.0, 3.0, 2.0);
    assert_eq!(distance(&point, &point), 0.0);
}

#[test]
fn test_distance_horizontal() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(10.0, 0.0, 0.0);
    assert!((distance(&a, &b) - 10.0).abs() < 0.001);
}

#[test]
fn test_distance_vertical() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(0.0, 10.0, 0.0);
    assert!((distance(&a, &b) - 10.0).abs() < 0.001);
}

#[test]
fn test_distance_diagonal() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(3.0, 4.0, 0.0);
    assert!((distance(&a, &b) - 5.0).abs() < 0.001); // 3-4-5 triangle
}

#[test]
fn test_distance_3d() {
    let a = Point3D::from_meters(1.0, 2.0, 3.0);
    let b = Point3D::from_meters(4.0, 6.0, 8.0);
    assert!((distance(&a, &b) - 7.071).abs() < 0.01);
}

#[test]
fn test_distance_symmetric() {
    let a = Point3D::from_meters(1.0, 2.0, 3.0);
    let b = Point3D::from_meters(4.0, 5.0, 6.0);
    assert_eq!(distance(&a, &b), distance(&b, &a));
}

#[test]
fn test_distance_length_returns_length_type() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(10.0, 0.0, 0.0);
    let dist = distance_length(&a, &b);
    assert!((dist.get::<meter>() - 10.0).abs() < 0.001);
}

#[test]
fn test_distance_2d_zero() {
    let point = Point3D::from_meters(5.0, 3.0, 2.0);
    assert_eq!(distance_2d(&point, &point), 0.0);
}

#[test]
fn test_distance_2d_along_x() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(10.0, 0.0, 0.0);
    assert!((distance_2d(&a, &b) - 10.0).abs() < 0.001);
}

#[test]
fn test_distance_2d_along_z() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(0.0, 0.0, 10.0);
    assert!((distance_2d(&a, &b) - 10.0).abs() < 0.001);
}

#[test]
fn test_distance_2d_ignores_y() {
    let a = Point3D::from_meters(5.0, 0.0, 5.0);
    let b = Point3D::from_meters(5.0, 100.0, 5.0);
    assert_eq!(distance_2d(&a, &b), 0.0);
}

#[test]
fn test_distance_2d_differs_from_3d_when_y_differs() {
    let a = Point3D::from_meters(0.0, 0.0, 0.0);
    let b = Point3D::from_meters(3.0, 4.0, 0.0);
    assert!((distance(&a, &b) - 5.0).abs() < 0.001);
    assert!((distance_2d(&a, &b) - 3.0).abs() < 0.001);
}

#[test]
fn test_distance_2d_diagonal() {
    let a = Point3D::from_meters(0.0, 99.0, 0.0);
    let b = Point3D::from_meters(3.0, 0.0, 4.0);
    assert!((distance_2d(&a, &b) - 5.0).abs() < 0.001);
}

#[test]
fn test_distance_2d_symmetric() {
    let a = Point3D::from_meters(1.0, 2.0, 3.0);
    let b = Point3D::from_meters(4.0, 5.0, 6.0);
    assert_eq!(distance_2d(&a, &b), distance_2d(&b, &a));
}

#[test]
fn test_kick_speed_from_power() {
    assert_eq!(kick_speed(100), 20.0);
    assert_eq!(kick_speed(50), 10.0);
    assert_eq!(kick_speed(0), 0.0);
}

#[test]
fn test_max_kick_deviation_bounds() {
    assert_eq!(max_kick_deviation(100).get::<degree>(), 5.0);
    assert_eq!(max_kick_deviation(10).get::<degree>(), 45.0);
    assert_eq!(max_kick_deviation(55).get::<degree>(), 25.0);
}

#[test]
fn test_rotate_kick_direction_no_deviation() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(60.0, 30.0, 0.0);

    let (dx, dz) = rotate_kick_direction(&target, &ball, Angle::new::<degree>(0.0));

    assert!((dx - 1.0).abs() < 0.001);
    assert!(dz.abs() < 0.001);
}

#[test]
fn test_rotate_kick_direction_along_z() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(50.0, 30.0, 10.0);

    let (dx, dz) = rotate_kick_direction(&target, &ball, Angle::new::<degree>(0.0));

    assert!(dx.abs() < 0.001);
    assert!((dz - 1.0).abs() < 0.001);
}

#[test]
fn test_rotate_kick_direction_45_degrees() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(60.0, 30.0, 0.0);

    let (dx, dz) = rotate_kick_direction(&target, &ball, Angle::new::<degree>(45.0));

    assert!((dx - 0.707).abs() < 0.01);
    assert!((dz - 0.707).abs() < 0.01);
}

#[test]
fn test_rotate_kick_direction_degenerate_target_equals_ball() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);

    let (dx, dz) = rotate_kick_direction(&ball, &ball, Angle::new::<degree>(30.0));

    assert_eq!((dx, dz), (1.0, 0.0));
}
