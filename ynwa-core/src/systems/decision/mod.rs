mod decision_parser;
mod decision_system;
mod scripted_decision_maker;

pub use decision_system::{DecisionError, DecisionMaker, DecisionSystem, PlaceholderDecisionMaker};
pub use scripted_decision_maker::ScriptedDecisionMaker;

use crate::game::{Decision, DecisionTarget};
use crate::orientation::flip_point_orientation;
use crate::region::{GridDimensions, Region};
use crate::team::Team;

/// Converts a decision from team-local orientation to display orientation (Team A perspective).
/// Team A decisions pass through unchanged. Applied after receiving decision from DecisionMaker.
fn convert_decision_to_display_orientation(
    decision: &Decision,
    team: Team,
    field_width: f32,
    field_height: f32,
    grid_dimensions: GridDimensions,
) -> Decision {
    if team == Team::A {
        return decision.clone();
    }

    match decision {
        Decision::Run(target) => {
            let flipped_target = match target {
                DecisionTarget::Region(region) => {
                    let flipped = region.flip_orientation(grid_dimensions).unwrap();
                    // flip_orientation swaps team; force Team::A for display orientation
                    let display_region =
                        Region::new_unchecked(Team::A, flipped.top_left, flipped.bottom_right);
                    DecisionTarget::Region(display_region)
                }
                DecisionTarget::GridCell(cell) => {
                    DecisionTarget::GridCell(cell.flip_orientation(grid_dimensions).unwrap())
                }
                DecisionTarget::Point(point) => {
                    DecisionTarget::Point(flip_point_orientation(point, field_width, field_height))
                }
            };
            Decision::Run(flipped_target)
        }
        Decision::Stop => Decision::Stop,
        Decision::Kick(target_point) => Decision::Kick(flip_point_orientation(
            target_point,
            field_width,
            field_height,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::zones::Point3D;
    use crate::region::{GridCell, GridDimensions, Region};
    use uom::si::length::meter;

    #[test]
    fn test_convert_decision_team_a_unchanged() {
        let grid_dims = GridDimensions::new(26, 44);
        let region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();
        let decision = Decision::Run(DecisionTarget::Region(region.clone()));
        let converted =
            convert_decision_to_display_orientation(&decision, Team::A, 100.0, 60.0, grid_dims);

        match converted {
            Decision::Run(DecisionTarget::Region(r)) => {
                assert_eq!(r.team, region.team);
                assert_eq!(r.top_left, region.top_left);
                assert_eq!(r.bottom_right, region.bottom_right);
            }
            _ => panic!("Expected Run(Region)"),
        }

        let converted = convert_decision_to_display_orientation(
            &Decision::Stop,
            Team::A,
            100.0,
            60.0,
            grid_dims,
        );
        assert!(matches!(converted, Decision::Stop));
    }

    #[test]
    fn test_convert_decision_team_b_region_flipped() {
        let grid_dims = GridDimensions::new(26, 44);
        let region_b = Region::new(
            Team::B,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let decision = Decision::Run(DecisionTarget::Region(region_b));
        let converted =
            convert_decision_to_display_orientation(&decision, Team::B, 100.0, 60.0, grid_dims);

        match converted {
            Decision::Run(DecisionTarget::Region(r)) => {
                assert_eq!(r.team, Team::A);
                // A1 for Team B = col 26, row 44 in 26x44 grid
                assert_eq!(r.top_left, GridCell::new(26, 44).unwrap());
                assert_eq!(r.bottom_right, GridCell::new(26, 44).unwrap());
            }
            _ => panic!("Expected Run(Region)"),
        }
    }

    #[test]
    fn test_convert_decision_team_b_cell_flipped() {
        let grid_dims = GridDimensions::new(26, 44);
        let decision = Decision::Run(DecisionTarget::GridCell(GridCell::new(1, 1).unwrap()));
        let converted =
            convert_decision_to_display_orientation(&decision, Team::B, 100.0, 60.0, grid_dims);

        match converted {
            Decision::Run(DecisionTarget::GridCell(c)) => {
                assert_eq!(c, GridCell::new(26, 44).unwrap());
            }
            _ => panic!("Expected Run(GridCell)"),
        }
    }

    #[test]
    fn test_convert_decision_team_b_point_flipped() {
        let grid_dims = GridDimensions::new(26, 44);
        let decision = Decision::Run(DecisionTarget::Point(Point3D::from_meters(20.0, 1.0, 15.0)));
        let converted =
            convert_decision_to_display_orientation(&decision, Team::B, 100.0, 60.0, grid_dims);

        match converted {
            Decision::Run(DecisionTarget::Point(p)) => {
                assert_eq!(p.x.get::<meter>(), 80.0);
                assert_eq!(p.y.get::<meter>(), 1.0);
                assert_eq!(p.z.get::<meter>(), 45.0);
            }
            _ => panic!("Expected Run(Point)"),
        }
    }

    #[test]
    fn test_convert_decision_team_b_stop_unchanged() {
        let grid_dims = GridDimensions::new(26, 44);
        let converted = convert_decision_to_display_orientation(
            &Decision::Stop,
            Team::B,
            100.0,
            60.0,
            grid_dims,
        );
        assert!(matches!(converted, Decision::Stop));
    }

    #[test]
    fn test_convert_decision_team_b_kick_flipped() {
        let grid_dims = GridDimensions::new(26, 44);
        let decision = Decision::Kick(Point3D::from_meters(20.0, 0.0, 15.0));
        let converted =
            convert_decision_to_display_orientation(&decision, Team::B, 100.0, 60.0, grid_dims);

        match converted {
            Decision::Kick(p) => {
                assert_eq!(p.x.get::<meter>(), 80.0);
                assert_eq!(p.z.get::<meter>(), 45.0);
            }
            _ => panic!("Expected Kick"),
        }
    }
}
