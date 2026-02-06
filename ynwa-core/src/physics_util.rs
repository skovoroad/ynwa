use crate::field::zones::Point3D;
use uom::si::f32::Length;
use uom::si::length::meter;

/// Calculate distance between two 3D points (returns raw f32 in meters)
pub fn distance(a: &Point3D, b: &Point3D) -> f32 {
    let dx = a.x.get::<meter>() - b.x.get::<meter>();
    let dy = a.y.get::<meter>() - b.y.get::<meter>();
    let dz = a.z.get::<meter>() - b.z.get::<meter>();
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Calculate distance between two 3D points (returns Length type)
pub fn distance_length(a: &Point3D, b: &Point3D) -> Length {
    Length::new::<meter>(distance(a, b))
}

/// Calculate kick velocity: shot_power/10 * (0.75 + rng*0.5) → base ±25% variation
pub fn calculate_kick_velocity(shot_power: u32, rng_value: f32) -> f32 {
    let base_speed = shot_power as f32 / 10.0; // shot_power=100 → 10 m/s
    let variation = 0.75 + rng_value * 0.5; // 0.75 to 1.25 (±25%)
    base_speed * variation
}

/// Calculate kick direction with accuracy-based deviation (100→±5°, 10→±45°)
/// Returns normalized (dx, dz) direction vector
pub fn calculate_kick_direction_with_accuracy(
    target: &Point3D,
    ball_position: &Point3D,
    shot_accuracy: u32,
    rng_value: f32,
) -> (f32, f32) {
    // Calculate base direction vector
    let dx = target.x.get::<meter>() - ball_position.x.get::<meter>();
    let dz = target.z.get::<meter>() - ball_position.z.get::<meter>();
    
    // Normalize
    let length = (dx * dx + dz * dz).sqrt();
    if length < 0.001 {
        // Degenerate case: target == ball position, kick in default direction
        return (1.0, 0.0);
    }
    
    let dx_norm = dx / length;
    let dz_norm = dz / length;
    
    // Calculate max deviation in degrees based on accuracy
    // accuracy=100 → 5 degrees, accuracy=10 → 45 degrees
    let max_deviation_degrees = 5.0 + (100.0 - shot_accuracy as f32) * (45.0 - 5.0) / 90.0;
    
    // Convert to radians and apply random deviation
    // rng_value: 0.5 = no deviation, 0.0 = max negative, 1.0 = max positive
    let deviation_radians = (rng_value - 0.5) * 2.0 * max_deviation_degrees.to_radians();
    
    // Apply rotation to direction vector
    let cos_angle = deviation_radians.cos();
    let sin_angle = deviation_radians.sin();
    
    let dx_rotated = dx_norm * cos_angle - dz_norm * sin_angle;
    let dz_rotated = dx_norm * sin_angle + dz_norm * cos_angle;
    
    (dx_rotated, dz_rotated)
}

#[cfg(test)]
mod tests {
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
        // shot_power=100, no variation (rng=0.5) → 10.0 m/s
        let velocity = calculate_kick_velocity(100, 0.5);
        assert!((velocity - 10.0).abs() < 0.001);
        
        // shot_power=50, no variation → 5.0 m/s
        let velocity = calculate_kick_velocity(50, 0.5);
        assert!((velocity - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_kick_velocity_min_variation() {
        // shot_power=100, min variation (rng=0.0) → 7.5 m/s (-25%)
        let velocity = calculate_kick_velocity(100, 0.0);
        assert!((velocity - 7.5).abs() < 0.001);
    }

    #[test]
    fn test_kick_velocity_max_variation() {
        // shot_power=100, max variation (rng=1.0) → 12.5 m/s (+25%)
        let velocity = calculate_kick_velocity(100, 1.0);
        assert!((velocity - 12.5).abs() < 0.001);
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
}
