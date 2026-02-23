use super::*;
use crate::field::Field;
use crate::game::{BallState, Decision, DecisionTarget};
use crate::region::GridCell;
use uom::si::length::meter;

fn make_field() -> Field {
    Field::from_meters(100.0, 60.0, 10, 6)
}

fn make_ball_state() -> BallState {
    BallState::default()
}

#[test]
fn test_resolve_target_point_from_point() {
    let field = make_field();
    let target = crate::field::zones::Point3D::from_meters(30.0, 0.0, 20.0);
    let decision = Decision::Run(DecisionTarget::Point(target));

    let result = resolve_target_point(&decision, field.width().get::<meter>(), field.grid_dimensions(), &make_ball_state());

    assert!(result.is_some());
    let p = result.unwrap();
    assert!((p.x.get::<meter>() - 30.0).abs() < 0.001);
    assert!((p.z.get::<meter>() - 20.0).abs() < 0.001);
}

#[test]
fn test_resolve_target_point_from_grid_cell() {
    let field = make_field();
    let cell = GridCell::new(1, 1).unwrap();
    let decision = Decision::Run(DecisionTarget::GridCell(cell));

    let result = resolve_target_point(&decision, field.width().get::<meter>(), field.grid_dimensions(), &make_ball_state());

    // Cell (1,1) center must be inside the field
    assert!(result.is_some());
    let p = result.unwrap();
    assert!(p.x.get::<meter>() > 0.0);
    assert!(p.z.get::<meter>() > 0.0);
}

#[test]
fn test_resolve_target_point_from_region() {
    let field = make_field();
    let grid_dims = field.grid_dimensions();
    let cell_a = GridCell::new(1, 1).unwrap();
    let cell_b = GridCell::new(2, 2).unwrap();
    let region = grid_dims.create_region(cell_a, cell_b).unwrap();
    let decision = Decision::Run(DecisionTarget::Region(region));

    let result = resolve_target_point(&decision, field.width().get::<meter>(), grid_dims, &make_ball_state());

    assert!(result.is_some());
}

#[test]
fn test_resolve_target_point_stop_returns_none() {
    let field = make_field();
    let decision = Decision::Stop;

    let result = resolve_target_point(&decision, field.width().get::<meter>(), field.grid_dimensions(), &make_ball_state());

    assert!(result.is_none());
}

#[test]
fn test_resolve_target_point_kick_returns_none() {
    let field = make_field();
    let target = crate::field::zones::Point3D::from_meters(50.0, 0.0, 30.0);
    let decision = Decision::Kick(target);

    let result = resolve_target_point(&decision, field.width().get::<meter>(), field.grid_dimensions(), &make_ball_state());

    assert!(result.is_none());
}

#[test]
fn test_resolve_target_point_grid_cell_center_consistent_with_region() {
    // A single-cell region and a GridCell decision for the same cell
    // must resolve to the same point.
    let field = make_field();
    let grid_dims = field.grid_dimensions();
    let cell = GridCell::new(3, 2).unwrap();

    let decision_cell = Decision::Run(DecisionTarget::GridCell(cell));
    let region = grid_dims.create_region(cell, cell).unwrap();
    let decision_region = Decision::Run(DecisionTarget::Region(region));

    let point_from_cell = resolve_target_point(&decision_cell, field.width().get::<meter>(), grid_dims, &make_ball_state()).unwrap();
    let point_from_region = resolve_target_point(&decision_region, field.width().get::<meter>(), grid_dims, &make_ball_state()).unwrap();

    assert!((point_from_cell.x.get::<meter>() - point_from_region.x.get::<meter>()).abs() < 0.001);
    assert!((point_from_cell.z.get::<meter>() - point_from_region.z.get::<meter>()).abs() < 0.001);
}

#[test]
fn test_resolve_target_point_ball_returns_ball_position() {
    let field = make_field();
    let mut ball_state = make_ball_state();
    ball_state.position = crate::field::zones::Point3D::from_meters(42.0, 0.0, 37.5);

    let decision = Decision::Run(DecisionTarget::Ball);
    let result = resolve_target_point(&decision, field.width().get::<meter>(), field.grid_dimensions(), &ball_state);

    assert!(result.is_some());
    let p = result.unwrap();
    assert!((p.x.get::<meter>() - 42.0).abs() < 0.001);
    assert!((p.z.get::<meter>() - 37.5).abs() < 0.001);
}