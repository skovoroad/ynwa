use crate::field::zones::Point3D;
use crate::game::{Decision, DecisionTarget};
use crate::region::GridDimensions;

/// Resolves the target point of a `Decision::Run` variant into a `Point3D`.
/// Returns `None` for non-Run decisions.
pub fn resolve_target_point(
    decision: &Decision,
    field_width: f32,
    grid_dims: GridDimensions,
) -> Option<Point3D> {
    match decision {
        Decision::Run(target) => Some(match target {
            DecisionTarget::Point(p) => *p,
            DecisionTarget::GridCell(cell) => {
                crate::region::Region::new(*cell, *cell).center(grid_dims, field_width)
            }
            DecisionTarget::Region(r) => r.center(grid_dims, field_width),
        }),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/decision_util_tests.rs"]
mod tests;
