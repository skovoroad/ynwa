use uom::si::angle::radian;
use uom::si::f32::{Angle, Length, Velocity};
use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

/// 3D point using Y-up coordinate system:
/// - X: field width (left-right)
/// - Y: height (up)
/// - Z: field length (team A to team B)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Point3D {
    pub x: Length,
    pub y: Length,
    pub z: Length,
}

/// 3D velocity vector using Y-up coordinate system:
/// - X: velocity along field width (left-right)
/// - Y: velocity along height (up-down)
/// - Z: velocity along field length (team A to team B)
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Velocity3D {
    pub x: Velocity,
    pub y: Velocity,
    pub z: Velocity,
}

impl Point3D {
    pub fn new(x: Length, y: Length, z: Length) -> Self {
        Self { x, y, z }
    }

    pub fn from_meters(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: Length::new::<meter>(x),
            y: Length::new::<meter>(y),
            z: Length::new::<meter>(z),
        }
    }

    /// Create a point on the field surface (y = 0)
    pub fn on_ground(x: f32, z: f32) -> Self {
        Self::from_meters(x, 0.0, z)
    }
}

impl Velocity3D {
    pub fn new(x: Velocity, y: Velocity, z: Velocity) -> Self {
        Self { x, y, z }
    }

    pub fn from_meters_per_second(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: Velocity::new::<meter_per_second>(x),
            y: Velocity::new::<meter_per_second>(y),
            z: Velocity::new::<meter_per_second>(z),
        }
    }

    pub fn zero() -> Self {
        Self::from_meters_per_second(0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Rectangle {
    pub min: Point3D,
    pub max: Point3D,
}

impl Rectangle {
    pub fn new(min: Point3D, max: Point3D) -> Self {
        assert!(
            min.x < max.x && min.z < max.z,
            "Rectangle min corner must be less than max corner: min=({}, {}), max=({}, {})",
            min.x.get::<meter>(),
            min.z.get::<meter>(),
            max.x.get::<meter>(),
            max.z.get::<meter>()
        );
        Self { min, max }
    }

    /// Create a rectangle on the ground (y = 0)
    pub fn from_meters(x_min: f32, z_min: f32, x_max: f32, z_max: f32) -> Self {
        Self::new(
            Point3D::on_ground(x_min, z_min),
            Point3D::on_ground(x_max, z_max),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Circle {
    pub center: Point3D,
    pub radius: Length,
}

impl Circle {
    pub fn new(center: Point3D, radius: Length) -> Self {
        assert!(
            radius.get::<meter>() > 0.0,
            "Circle radius must be positive: {}",
            radius.get::<meter>()
        );
        Self { center, radius }
    }

    /// Create a circle on the ground (y = 0)
    pub fn from_meters(x: f32, z: f32, radius: f32) -> Self {
        Self::new(Point3D::on_ground(x, z), Length::new::<meter>(radius))
    }
}

/// Arc segment - part of a circle
#[derive(Debug, Clone, PartialEq)]
pub struct Arc {
    pub center: Point3D,
    pub radius: Length,
    pub start_angle: Angle,
    pub end_angle: Angle,
}

impl Arc {
    pub fn new(center: Point3D, radius: Length, start_angle: Angle, end_angle: Angle) -> Self {
        assert!(
            radius.get::<meter>() > 0.0,
            "Arc radius must be positive: {}",
            radius.get::<meter>()
        );
        assert!(
            start_angle.get::<radian>() != end_angle.get::<radian>(),
            "Arc start and end angles must be different: start={}, end={}",
            start_angle.get::<radian>(),
            end_angle.get::<radian>()
        );
        Self {
            center,
            radius,
            start_angle,
            end_angle,
        }
    }

    /// Create an arc on the ground (y = 0)
    pub fn from_radians(
        x: f32,
        z: f32,
        radius: f32,
        start_angle_rad: f32,
        end_angle_rad: f32,
    ) -> Self {
        Self::new(
            Point3D::on_ground(x, z),
            Length::new::<meter>(radius),
            Angle::new::<radian>(start_angle_rad),
            Angle::new::<radian>(end_angle_rad),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PointZone {
    pub position: Point3D,
}

impl PointZone {
    pub fn new(position: Point3D) -> Self {
        Self { position }
    }

    /// Create a point zone on the ground (y = 0)
    pub fn from_meters(x: f32, z: f32) -> Self {
        Self {
            position: Point3D::on_ground(x, z),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ZoneGeometry {
    Rectangle(Rectangle),
    Circle(Circle),
    Arc(Arc),
    Point(PointZone),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    // Rectangle tests
    #[test]
    fn test_rectangle_valid() {
        let rect = Rectangle::from_meters(0.0, 0.0, 10.0, 20.0);
        assert_eq!(rect.min.x.get::<meter>(), 0.0);
        assert_eq!(rect.max.z.get::<meter>(), 20.0);
    }

    #[test]
    #[should_panic(expected = "Rectangle min corner must be less than max corner")]
    fn test_rectangle_inverted_x() {
        Rectangle::from_meters(10.0, 0.0, 0.0, 10.0); // x_min > x_max
    }

    #[test]
    #[should_panic(expected = "Rectangle min corner must be less than max corner")]
    fn test_rectangle_inverted_z() {
        Rectangle::from_meters(0.0, 10.0, 10.0, 0.0); // z_min > z_max
    }

    #[test]
    #[should_panic(expected = "Rectangle min corner must be less than max corner")]
    fn test_rectangle_equal_corners() {
        Rectangle::from_meters(5.0, 5.0, 5.0, 5.0); // min == max (degenerate)
    }

    // Circle tests
    #[test]
    fn test_circle_valid() {
        let circle = Circle::from_meters(5.0, 5.0, 3.0);
        assert_eq!(circle.radius.get::<meter>(), 3.0);
    }

    #[test]
    #[should_panic(expected = "Circle radius must be positive")]
    fn test_circle_zero_radius() {
        Circle::from_meters(5.0, 5.0, 0.0);
    }

    #[test]
    #[should_panic(expected = "Circle radius must be positive")]
    fn test_circle_negative_radius() {
        Circle::from_meters(5.0, 5.0, -3.0);
    }

    // Arc tests
    #[test]
    fn test_arc_valid() {
        let arc = Arc::from_radians(10.0, 10.0, 5.0, 0.0, PI / 2.0);
        assert_eq!(arc.radius.get::<meter>(), 5.0);
        assert_eq!(arc.start_angle.get::<radian>(), 0.0);
        assert_eq!(arc.end_angle.get::<radian>(), PI / 2.0);
    }

    #[test]
    #[should_panic(expected = "Arc radius must be positive")]
    fn test_arc_zero_radius() {
        Arc::from_radians(0.0, 0.0, 0.0, 0.0, PI);
    }

    #[test]
    #[should_panic(expected = "Arc radius must be positive")]
    fn test_arc_negative_radius() {
        Arc::from_radians(0.0, 0.0, -5.0, 0.0, PI);
    }

    #[test]
    #[should_panic(expected = "Arc start and end angles must be different")]
    fn test_arc_equal_angles() {
        Arc::from_radians(0.0, 0.0, 5.0, PI / 2.0, PI / 2.0);
    }

    #[test]
    fn test_arc_reverse_direction() {
        // Reverse direction (end < start) - valid for arcs
        let arc = Arc::from_radians(0.0, 0.0, 5.0, PI, 0.0);
        assert_eq!(arc.start_angle.get::<radian>(), PI);
        assert_eq!(arc.end_angle.get::<radian>(), 0.0);
    }

    #[test]
    fn test_arc_full_circle_minus_epsilon() {
        // Almost full circle - valid
        let arc = Arc::from_radians(0.0, 0.0, 5.0, 0.0, 2.0 * PI - 0.001);
        assert!(arc.start_angle.get::<radian>() < arc.end_angle.get::<radian>());
    }

    // Point tests (any coordinates are valid)
    #[test]
    fn test_point_zone_any_coordinates() {
        let p1 = PointZone::from_meters(0.0, 0.0);
        let p2 = PointZone::from_meters(-100.0, 50.0);
        let p3 = PointZone::from_meters(f32::MAX, f32::MIN);

        assert_eq!(p1.position.x.get::<meter>(), 0.0);
        assert_eq!(p2.position.x.get::<meter>(), -100.0);
        assert!(p3.position.x.get::<meter>().is_finite());
    }
}
