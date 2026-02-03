use crate::field::zones::Point3D;
use uom::si::f32::Length;
use uom::si::length::meter;

pub fn distance(a: &Point3D, b: &Point3D) -> f32 {
    let dx = a.x.get::<meter>() - b.x.get::<meter>();
    let dy = a.y.get::<meter>() - b.y.get::<meter>();
    let dz = a.z.get::<meter>() - b.z.get::<meter>();
    (dx * dx + dy * dy + dz * dz).sqrt()
}

pub fn distance_length(a: &Point3D, b: &Point3D) -> Length {
    Length::new::<meter>(distance(a, b))
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
}
