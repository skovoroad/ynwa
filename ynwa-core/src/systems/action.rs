use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, DecisionTarget, Game};
use crate::physics_util::{kick_speed, max_kick_deviation, rotate_kick_direction};
use crate::region::Region;
use crate::system::System;
use uom::si::angle::degree;
use uom::si::f32::Angle;
use uom::si::length::meter;

#[cfg(test)]
use uom::si::velocity::meter_per_second;

/// Maximum player speed when speed_rate = 100 (roughly 36 km/h, realistic for professional football)
const MAX_SPEED_METERS_PER_SECOND: f32 = 10.0;

// Design: ActionSystem translates decisions into physical actions (velocity changes).
// Separates high-level decision-making from low-level physics.

fn calculate_target_point(target: &DecisionTarget, game: &Game) -> Point3D {
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

fn calculate_velocity(
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

pub struct ActionSystem;

impl ActionSystem {
    pub fn new() -> Self {
        Self
    }
}

impl System for ActionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let player_count = game.state.player_states.len();

        for player_index in 0..player_count {
            let decision = game.state.player_states[player_index]
                .current_decision
                .clone();
            let decision_processed = game.state.player_states[player_index].decision_processed;

            if let Some(decision) = decision {
                if !decision_processed {
                    match decision {
                        Decision::Stop => {
                            game.state.player_states[player_index].velocity = Velocity3D::default();
                        }
                        Decision::Run(target) => {
                            let player_def = &game.config().players[player_index];
                            let player_position = game.state.player_states[player_index].position;

                            let target_point = calculate_target_point(&target, game);

                            let velocity = calculate_velocity(
                                &player_position,
                                &target_point,
                                player_def.speed_rate,
                            );

                            game.state.player_states[player_index].velocity = velocity;
                        }
                        Decision::Kick(target_point) => {
                            // Only process kick if player owns the ball
                            if game.state.ball_state.possessed_by == Some(player_index) {
                                let shot_power = game.config().players[player_index].shot_power;
                                let shot_accuracy =
                                    game.config().players[player_index].shot_accuracy;
                                let ball_position = game.state.ball_state.position;

                                let speed =
                                    game.rng_manager().randomize(kick_speed(shot_power), 0.25);

                                let max_deviation_degrees =
                                    max_kick_deviation(shot_accuracy).get::<degree>();
                                let deviation_degrees =
                                    game.rng_manager().randomize_range(max_deviation_degrees);
                                let deviation = Angle::new::<degree>(deviation_degrees);

                                let (dx, dz) =
                                    rotate_kick_direction(&target_point, &ball_position, deviation);

                                game.state.ball_state.velocity =
                                    Velocity3D::from_meters_per_second(dx * speed, 0.0, dz * speed);

                                // Release possession and reset cooldown timer so
                                // BallPossessionSystem won't immediately re-assign
                                // the ball back to the kicker on the next tick.
                                game.state.ball_state.possessed_by = None;
                                game.state.ball_state.last_possession_change_time = timestamp;
                            }
                            // If player doesn't own ball, ignore kick decision (no action)
                        }
                    }

                    game.state.player_states[player_index].decision_processed = true;
                }
            }
        }
    }
}

impl Default for ActionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/action_tests.rs"]
mod tests;
