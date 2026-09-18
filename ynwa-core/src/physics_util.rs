//! Physics utility functions and player speed model.
//!
//! Player speed: `actual_speed = (speed_rate / 100.0) * MAX_SPEED (10.0 m/s)`
//! `speed_rate` range: 10-100, linear dependency.
//!
//! 3D types (`Point3D`, `Velocity3D`) are defined in `field::zones`, not here.

use crate::field::zones::Point3D;
use uom::si::angle::{degree, radian};
use uom::si::f32::{Angle, Length};
use uom::si::length::meter;

/// shot_power=100 → base kick speed = 100/KICK_POWER_DIVISOR m/s
pub const KICK_POWER_DIVISOR: f32 = 5.0;

/// Calculate distance between two 3D points (returns raw f32 in meters)
pub fn distance(a: &Point3D, b: &Point3D) -> f32 {
    let dx = a.x.get::<meter>() - b.x.get::<meter>();
    let dy = a.y.get::<meter>() - b.y.get::<meter>();
    let dz = a.z.get::<meter>() - b.z.get::<meter>();
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Calculate horizontal distance between two points, ignoring the Y axis (meters)
pub fn distance_2d(a: &Point3D, b: &Point3D) -> f32 {
    let dx = a.x.get::<meter>() - b.x.get::<meter>();
    let dz = a.z.get::<meter>() - b.z.get::<meter>();
    (dx * dx + dz * dz).sqrt()
}

/// Calculate distance between two 3D points (returns Length type)
pub fn distance_length(a: &Point3D, b: &Point3D) -> Length {
    Length::new::<meter>(distance(a, b))
}

/// Base kick speed from `shot_power`; the `RngManager` applies the variation.
pub fn kick_speed(shot_power: u32) -> f32 {
    shot_power as f32 / KICK_POWER_DIVISOR
}

/// Maximum kick deviation: accuracy 100 → ±5°, accuracy 10 → ±45°.
pub fn max_kick_deviation(shot_accuracy: u32) -> Angle {
    Angle::new::<degree>(5.0 + (100.0 - shot_accuracy as f32) * 40.0 / 90.0)
}

/// Rotates the normalized direction from `ball` to `target` by `deviation`.
pub fn rotate_kick_direction(target: &Point3D, ball: &Point3D, deviation: Angle) -> (f32, f32) {
    let dx = target.x.get::<meter>() - ball.x.get::<meter>();
    let dz = target.z.get::<meter>() - ball.z.get::<meter>();

    let length = (dx * dx + dz * dz).sqrt();
    if length < 0.001 {
        return (1.0, 0.0);
    }

    let dx_norm = dx / length;
    let dz_norm = dz / length;

    let angle = deviation.get::<radian>();
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();

    (
        dx_norm * cos_angle - dz_norm * sin_angle,
        dx_norm * sin_angle + dz_norm * cos_angle,
    )
}

#[cfg(test)]
#[path = "tests/physics_util_tests.rs"]
mod tests;
