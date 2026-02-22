//! Physics utility functions and player speed model.
//!
//! Player speed: `actual_speed = (speed_rate / 100.0) * MAX_SPEED (10.0 m/s)`
//! `speed_rate` range: 10-100, linear dependency.
//!
//! 3D types (`Point3D`, `Velocity3D`) are defined in `field::zones`, not here.

use crate::field::zones::Point3D;
use uom::si::f32::Length;
use uom::si::length::meter;

/// Kick power conversion factor: shot_power is divided by this to get base m/s
/// shot_power=100 → base velocity = 100/KICK_POWER_DIVISOR m/s
pub const KICK_POWER_DIVISOR: f32 = 5.0;

/// Kick power variation range (±25% around base)
pub const KICK_POWER_VARIATION_MIN: f32 = 0.75; // -25%
pub const KICK_POWER_VARIATION_MAX: f32 = 1.25; // +25%

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

/// Calculate kick velocity: shot_power/KICK_POWER_DIVISOR * (0.75 + rng*0.5) → base ±25% variation
pub fn calculate_kick_velocity(shot_power: u32, rng_value: f32) -> f32 {
    let base_speed = shot_power as f32 / KICK_POWER_DIVISOR;
    let variation = KICK_POWER_VARIATION_MIN
        + rng_value * (KICK_POWER_VARIATION_MAX - KICK_POWER_VARIATION_MIN);
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
#[path = "tests/physics_util_tests.rs"]
mod tests;
