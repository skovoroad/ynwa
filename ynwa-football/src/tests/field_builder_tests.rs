/// Tests for `field_builder` zone geometry and its consistency with `region` coordinates.
///
/// Verifies that football-specific zones (halves, penalty areas, goals, center circle)
/// are correctly placed on the X/Z axes:
///
///   X axis  = field WIDTH  (horizontal, short side, 60 m)
///   Z axis  = field LENGTH (vertical,   long  side, 101.538 m)
///   col (letter A-Z) → X axis   (26 columns span the width)
///   row (number 1-44) → Z axis  (44 rows   span the length)
///   Team A goal at Z = 0,   Team B goal at Z = field_length
///
/// Cross-layer tests (Layer 3) ensure that `region.center()` and zone geometry
/// agree: e.g. cell A1 falls inside `half_a`, cell Z44 inside `half_b`.

use crate::field_builder::create_football_field;
use ynwa_core::field::zones::ZoneGeometry;
use ynwa_core::region::GridCell;
use uom::si::length::meter;

fn football_field() -> ynwa_core::field::Field {
    create_football_field()
}

// ── Layer 1: Field dimensions ─────────────────────────────────────────────────

#[test]
fn test_field_x_is_width_z_is_length() {
    let field = football_field();
    let w = field.width().get::<meter>();
    let l = field.length().get::<meter>();

    assert!(w < l, "Field width ({w:.2}) must be shorter than length ({l:.2}): X=width, Z=length");
    assert!((w - 60.0).abs() < 0.1, "Field width must be ~60 m (X axis), got {w}");
    assert!((l - 101.538).abs() < 0.1, "Field length must be ~101.5 m (Z axis), got {l}");
}

// ── Layer 2: field_builder zone geometry ─────────────────────────────────────

#[test]
fn test_half_a_zone_spans_z_axis() {
    let field = football_field();
    let zone = field
        .get_zone("half", Some(ynwa_core::team::Team::A))
        .expect("half_a zone must exist");

    let rect = match &zone.geometry {
        ZoneGeometry::Rectangle(r) => r,
        other => panic!("half_a should be a Rectangle, got {:?}", other),
    };

    let w = field.width().get::<meter>();
    let l = field.length().get::<meter>();

    assert!((rect.min.x.get::<meter>() - 0.0).abs() < 0.1, "half_a min_x must be 0, got {}", rect.min.x.get::<meter>());
    assert!((rect.max.x.get::<meter>() - w).abs() < 0.1, "half_a max_x must be field_width={w:.2}, got {}", rect.max.x.get::<meter>());
    assert!((rect.min.z.get::<meter>() - 0.0).abs() < 0.1, "half_a min_z must be 0 (Team A goal end), got {}", rect.min.z.get::<meter>());
    assert!((rect.max.z.get::<meter>() - l / 2.0).abs() < 0.5, "half_a max_z must be field_length/2={:.2}, got {}", l / 2.0, rect.max.z.get::<meter>());
}

#[test]
fn test_penalty_area_a_is_wide_not_deep() {
    let field = football_field();
    let zone = field
        .get_zone("penalty_area", Some(ynwa_core::team::Team::A))
        .expect("penalty_area_a must exist");

    let rect = match &zone.geometry {
        ZoneGeometry::Rectangle(r) => r,
        other => panic!("penalty_area_a should be Rectangle, got {:?}", other),
    };

    let x_span = rect.max.x.get::<meter>() - rect.min.x.get::<meter>();
    let z_span = rect.max.z.get::<meter>() - rect.min.z.get::<meter>();

    assert!(x_span > z_span, "penalty_area_a must be wider (X={x_span:.2}) than deep (Z={z_span:.2})");
    assert!(rect.min.z.get::<meter>().abs() < 0.1, "penalty_area_a must start at Z=0, got {}", rect.min.z.get::<meter>());
}

#[test]
fn test_penalty_area_b_is_at_far_end() {
    let field = football_field();
    let zone = field
        .get_zone("penalty_area", Some(ynwa_core::team::Team::B))
        .expect("penalty_area_b must exist");

    let rect = match &zone.geometry {
        ZoneGeometry::Rectangle(r) => r,
        other => panic!("penalty_area_b should be Rectangle, got {:?}", other),
    };

    let l = field.length().get::<meter>();
    assert!((rect.max.z.get::<meter>() - l).abs() < 0.1, "penalty_area_b must reach Z=field_length={l:.2}, got {}", rect.max.z.get::<meter>());
}

#[test]
fn test_center_circle_is_at_field_center() {
    let field = football_field();
    let zone = field.get_zone("center_circle", None).expect("center_circle must exist");

    let circle = match &zone.geometry {
        ZoneGeometry::Circle(c) => c,
        other => panic!("center_circle should be Circle, got {:?}", other),
    };

    let cx = circle.center.x.get::<meter>();
    let cz = circle.center.z.get::<meter>();
    let w = field.width().get::<meter>();
    let l = field.length().get::<meter>();

    assert!((cx - w / 2.0).abs() < 0.5, "center_circle X must be field_width/2={:.2}, got {cx:.2}", w / 2.0);
    assert!((cz - l / 2.0).abs() < 0.5, "center_circle Z must be field_length/2={:.2}, got {cz:.2}", l / 2.0);
}

#[test]
fn test_field_zone_covers_entire_field() {
    let field = football_field();
    let w = field.width().get::<meter>();
    let l = field.length().get::<meter>();

    let rect = match &field.get_zone("field", None).expect("field zone must exist").geometry {
        ZoneGeometry::Rectangle(r) => r.clone(),
        _ => panic!("field zone must be Rectangle"),
    };

    assert!((rect.min.x.get::<meter>() - 0.0).abs() < 0.01);
    assert!((rect.min.z.get::<meter>() - 0.0).abs() < 0.01);
    assert!((rect.max.x.get::<meter>() - w).abs() < 0.01, "field zone max_x must equal width={w:.2}, got {}", rect.max.x.get::<meter>());
    assert!((rect.max.z.get::<meter>() - l).abs() < 0.01, "field zone max_z must equal length={l:.2}, got {}", rect.max.z.get::<meter>());
}

#[test]
fn test_halves_are_adjacent_and_cover_full_length() {
    let field = football_field();
    let l = field.length().get::<meter>();

    let rect_a = match &field.get_zone("half", Some(ynwa_core::team::Team::A)).unwrap().geometry {
        ZoneGeometry::Rectangle(r) => r.clone(),
        _ => panic!(),
    };
    let rect_b = match &field.get_zone("half", Some(ynwa_core::team::Team::B)).unwrap().geometry {
        ZoneGeometry::Rectangle(r) => r.clone(),
        _ => panic!(),
    };

    assert!((rect_a.max.z.get::<meter>() - rect_b.min.z.get::<meter>()).abs() < 0.01, "half_a.max_z must equal half_b.min_z (midline): {:.4} vs {:.4}", rect_a.max.z.get::<meter>(), rect_b.min.z.get::<meter>());
    assert!((rect_a.min.z.get::<meter>() - 0.0).abs() < 0.01, "half_a must start at Z=0");
    assert!((rect_b.max.z.get::<meter>() - l).abs() < 0.01, "half_b must end at Z=field_length={l:.2}");
}

#[test]
fn test_goals_are_centred_on_x_axis() {
    let field = football_field();
    let w = field.width().get::<meter>();

    for team in [ynwa_core::team::Team::A, ynwa_core::team::Team::B] {
        let rect = match &field
            .get_zone("goal", Some(team))
            .unwrap_or_else(|| panic!("goal_{team:?} must exist"))
            .geometry
        {
            ZoneGeometry::Rectangle(r) => r.clone(),
            _ => panic!("goal must be Rectangle"),
        };

        let margin_left = rect.min.x.get::<meter>();
        let margin_right = w - rect.max.x.get::<meter>();

        assert!((margin_left - margin_right).abs() < 0.1, "goal_{team:?} must be centred on X: left={margin_left:.4}, right={margin_right:.4}");
    }
}

// ── Layer 3: cross-layer invariants (region ↔ field_builder) ─────────────────

fn point_in_rect(cx: f32, cz: f32, rect: &ynwa_core::field::zones::Rectangle) -> bool {
    cx >= rect.min.x.get::<meter>()
        && cx < rect.max.x.get::<meter>()
        && cz >= rect.min.z.get::<meter>()
        && cz < rect.max.z.get::<meter>()
}

#[test]
fn test_cell_a1_is_in_half_a_not_half_b() {
    let field = football_field();
    let w = field.width().get::<meter>();

    let region = field.grid_dimensions()
        .create_region(GridCell::from_notation("A1").unwrap(), GridCell::from_notation("A1").unwrap())
        .unwrap();
    let center = region.center(field.grid_dimensions(), w);
    let cx = center.x.get::<meter>();
    let cz = center.z.get::<meter>();

    let half_a = match &field.get_zone("half", Some(ynwa_core::team::Team::A)).unwrap().geometry { ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!() };
    let half_b = match &field.get_zone("half", Some(ynwa_core::team::Team::B)).unwrap().geometry { ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!() };

    assert!(point_in_rect(cx, cz, &half_a), "Cell A1 center ({cx:.4}, {cz:.4}) must lie inside half_a (Z near 0)");
    assert!(!point_in_rect(cx, cz, &half_b), "Cell A1 center ({cx:.4}, {cz:.4}) must NOT lie inside half_b");
}

#[test]
fn test_cell_z44_is_in_half_b_not_half_a() {
    let field = football_field();
    let w = field.width().get::<meter>();

    let region = field.grid_dimensions()
        .create_region(GridCell::from_notation("Z44").unwrap(), GridCell::from_notation("Z44").unwrap())
        .unwrap();
    let center = region.center(field.grid_dimensions(), w);
    let cx = center.x.get::<meter>();
    let cz = center.z.get::<meter>();

    let half_a = match &field.get_zone("half", Some(ynwa_core::team::Team::A)).unwrap().geometry { ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!() };
    let half_b = match &field.get_zone("half", Some(ynwa_core::team::Team::B)).unwrap().geometry { ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!() };

    assert!(point_in_rect(cx, cz, &half_b), "Cell Z44 center ({cx:.4}, {cz:.4}) must lie inside half_b");
    assert!(!point_in_rect(cx, cz, &half_a), "Cell Z44 center ({cx:.4}, {cz:.4}) must NOT lie inside half_a");
}

#[test]
fn test_cell_m1_is_in_penalty_area_a() {
    let field = football_field();
    let w = field.width().get::<meter>();

    let region = field.grid_dimensions()
        .create_region(GridCell::from_notation("M1").unwrap(), GridCell::from_notation("M1").unwrap())
        .unwrap();
    let center = region.center(field.grid_dimensions(), w);
    let cx = center.x.get::<meter>();
    let cz = center.z.get::<meter>();

    let pa = match &field.get_zone("penalty_area", Some(ynwa_core::team::Team::A)).expect("penalty_area_a must exist").geometry {
        ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!()
    };

    assert!(point_in_rect(cx, cz, &pa), "Cell M1 center ({cx:.4}, {cz:.4}) must be inside penalty_area_a (min_x={:.4}, max_x={:.4}, min_z={:.4}, max_z={:.4})", pa.min.x.get::<meter>(), pa.max.x.get::<meter>(), pa.min.z.get::<meter>(), pa.max.z.get::<meter>());
}

#[test]
fn test_cell_m44_is_in_penalty_area_b() {
    let field = football_field();
    let w = field.width().get::<meter>();

    let region = field.grid_dimensions()
        .create_region(GridCell::from_notation("M44").unwrap(), GridCell::from_notation("M44").unwrap())
        .unwrap();
    let center = region.center(field.grid_dimensions(), w);
    let cx = center.x.get::<meter>();
    let cz = center.z.get::<meter>();

    let pb = match &field.get_zone("penalty_area", Some(ynwa_core::team::Team::B)).expect("penalty_area_b must exist").geometry {
        ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!()
    };

    assert!(point_in_rect(cx, cz, &pb), "Cell M44 center ({cx:.4}, {cz:.4}) must be inside penalty_area_b (min_x={:.4}, max_x={:.4}, min_z={:.4}, max_z={:.4})", pb.min.x.get::<meter>(), pb.max.x.get::<meter>(), pb.min.z.get::<meter>(), pb.max.z.get::<meter>());
}

#[test]
fn test_cell_a1_not_in_penalty_area_a() {
    let field = football_field();
    let w = field.width().get::<meter>();

    let region = field.grid_dimensions()
        .create_region(GridCell::from_notation("A1").unwrap(), GridCell::from_notation("A1").unwrap())
        .unwrap();
    let center = region.center(field.grid_dimensions(), w);
    let cx = center.x.get::<meter>();
    let cz = center.z.get::<meter>();

    let pa = match &field.get_zone("penalty_area", Some(ynwa_core::team::Team::A)).expect("penalty_area_a must exist").geometry {
        ZoneGeometry::Rectangle(r) => r.clone(), _ => panic!()
    };

    assert!(!point_in_rect(cx, cz, &pa), "Corner cell A1 ({cx:.4}, {cz:.4}) must NOT be in penalty_area_a");
}
