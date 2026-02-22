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
    // sqrt((4-1)^2 + (6-2)^2 + (8-3)^2) = sqrt(9 + 16 + 25) = sqrt(50) ≈ 7.071
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
fn test_kick_velocity_base_calculation() {
    // shot_power=100, no variation (rng=0.5)
    let velocity = calculate_kick_velocity(100, 0.5);
    let expected = 100.0 / KICK_POWER_DIVISOR; // Base speed without variation
    assert!((velocity - expected).abs() < 0.001);

    // shot_power=50, no variation
    let velocity = calculate_kick_velocity(50, 0.5);
    let expected = 50.0 / KICK_POWER_DIVISOR;
    assert!((velocity - expected).abs() < 0.001);
}

#[test]
fn test_kick_velocity_min_variation() {
    // shot_power=100, min variation (rng=0.0)
    let velocity = calculate_kick_velocity(100, 0.0);
    let expected = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MIN;
    assert!((velocity - expected).abs() < 0.001);
}

#[test]
fn test_kick_velocity_max_variation() {
    // shot_power=100, max variation (rng=1.0)
    let velocity = calculate_kick_velocity(100, 1.0);
    let expected = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MAX;
    assert!((velocity - expected).abs() < 0.001);
}

#[test]
fn test_kick_direction_perfect_accuracy_no_deviation() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(60.0, 30.0, 0.0); // Straight along X axis

    // Perfect accuracy (100), no random deviation (0.5)
    let (dx, dz) = calculate_kick_direction_with_accuracy(&target, &ball, 100, 0.5);

    // Should point straight along X axis
    assert!((dx - 1.0).abs() < 0.001);
    assert!(dz.abs() < 0.001);
}

#[test]
fn test_kick_direction_perfect_accuracy_max_deviation() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(60.0, 30.0, 0.0); // Straight along X axis

    // Perfect accuracy (100), max deviation (rng=1.0) → +5 degrees
    let (dx, dz) = calculate_kick_direction_with_accuracy(&target, &ball, 100, 1.0);

    // 5 degrees rotation: cos(5°)≈0.996, sin(5°)≈0.087
    assert!((dx - 0.996).abs() < 0.01);
    assert!((dz - 0.087).abs() < 0.01);
}

#[test]
fn test_kick_direction_poor_accuracy_max_deviation() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(60.0, 30.0, 0.0); // Straight along X axis

    // Poor accuracy (10), max deviation (rng=1.0) → +45 degrees
    let (dx, dz) = calculate_kick_direction_with_accuracy(&target, &ball, 10, 1.0);

    // 45 degrees rotation: cos(45°)≈0.707, sin(45°)≈0.707
    assert!((dx - 0.707).abs() < 0.01);
    assert!((dz - 0.707).abs() < 0.01);
}

#[test]
fn test_kick_direction_degenerate_case() {
    let ball = Point3D::from_meters(50.0, 30.0, 0.0);
    let target = Point3D::from_meters(50.0, 30.0, 0.0); // Same position

    // Should return default direction (1, 0) when target == ball
    let (dx, dz) = calculate_kick_direction_with_accuracy(&target, &ball, 50, 0.5);

    assert!((dx - 1.0).abs() < 0.001);
    assert!(dz.abs() < 0.001);
}
