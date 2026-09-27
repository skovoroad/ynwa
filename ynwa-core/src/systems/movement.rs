//! Resolving a decision into movement, shared by `ActionSystem` and replay so both
//! produce identical player velocities from the same journaled decision.

use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{DecisionTarget, Game};
use crate::region::Region;
use uom::si::length::meter;

/// Maximum player speed when speed_rate = 100 (roughly 36 km/h, realistic for professional football)
const MAX_SPEED_METERS_PER_SECOND: f32 = 10.0;

pub(crate) fn calculate_target_point(target: &DecisionTarget, game: &Game) -> Point3D {
    match target {
        DecisionTarget::Point(point) => *point,
        DecisionTarget::GridCell(cell) => {
            let region = Region::new(*cell, *cell);

            region.center(
                game.config().field.grid_dimensions(),
                game.config().field.width().get::<meter>(),
            )
        }
        DecisionTarget::Region(region) => region.center(
            game.config().field.grid_dimensions(),
            game.config().field.width().get::<meter>(),
        ),
        // Ball position is resolved at the moment the decision is processed.
        // The player heads toward where the ball is right now and runs in a
        // straight line from there. If the ball moves, the player's direction
        // won't update until the next script invocation — this is intentional
        // (angular correction, not real-time tracking).
        DecisionTarget::Ball => game.state.ball_state.position,
    }
}

pub(crate) fn calculate_velocity(
    player_position: &Point3D,
    target_point: &Point3D,
    speed_rate: u32,
) -> Velocity3D {
    let dx = target_point.x.get::<meter>() - player_position.x.get::<meter>();
    let dy = target_point.y.get::<meter>() - player_position.y.get::<meter>();
    let dz = target_point.z.get::<meter>() - player_position.z.get::<meter>();

    let distance = (dx * dx + dy * dy + dz * dz).sqrt();

    // Stop when close enough to avoid overshooting
    if distance < 0.5 {
        return Velocity3D::default();
    }

    let base_speed = (speed_rate as f32 / 100.0) * MAX_SPEED_METERS_PER_SECOND;
    let direction_x = dx / distance;
    let direction_y = dy / distance;
    let direction_z = dz / distance;

    Velocity3D::from_meters_per_second(
        direction_x * base_speed,
        direction_y * base_speed,
        direction_z * base_speed,
    )
}

#[cfg(test)]
#[path = "../tests/movement_tests.rs"]
mod tests;
