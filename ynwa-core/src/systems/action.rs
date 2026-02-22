use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, DecisionTarget, Game};
use crate::physics_util::{calculate_kick_direction_with_accuracy, calculate_kick_velocity};
use crate::region::Region;
use crate::system::System;
use uom::si::length::meter;

#[cfg(test)]
use uom::si::velocity::meter_per_second;

#[cfg(test)]
use crate::physics_util::{KICK_POWER_DIVISOR, KICK_POWER_VARIATION_MAX, KICK_POWER_VARIATION_MIN};

// Maximum player speed when speed_rate = 100 (roughly 36 km/h, realistic for professional football)
const MAX_SPEED_METERS_PER_SECOND: f32 = 10.0;

// Design: ActionSystem translates decisions into physical actions (velocity changes).
// Separates high-level decision-making from low-level physics.

fn calculate_target_point(target: &DecisionTarget, game: &Game) -> Point3D {
    match target {
        DecisionTarget::Point(point) => *point,
        DecisionTarget::GridCell(cell) => {
            let region = Region::new(
                crate::team::Team::A,
                *cell,
                *cell,
                game.config().field.grid_dimensions(),
            )
            .expect("Cell should form valid region");

            region.center(
                game.config().field.grid_dimensions(),
                game.config().field.width().get::<meter>(),
            )
        }
        DecisionTarget::Region(region) => region.center(
            game.config().field.grid_dimensions(),
            game.config().field.width().get::<meter>(),
        ),
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

    if distance < 0.01 {
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

pub struct ActionSystem {
    /// Optional random number generator for testing (0.0 to 1.0)
    /// If None, uses rand::random()
    rng: Option<Box<dyn Fn() -> f32 + Send>>,
}

impl ActionSystem {
    pub fn new() -> Self {
        Self { rng: None }
    }

    /// Create a system with a custom RNG for testing
    #[cfg(test)]
    pub fn with_rng<F>(rng: F) -> Self
    where
        F: Fn() -> f32 + Send + 'static,
    {
        Self {
            rng: Some(Box::new(rng)),
        }
    }

    fn get_random(&self) -> f32 {
        if let Some(ref rng) = self.rng {
            rng()
        } else {
            rand::random()
        }
    }
}

impl System for ActionSystem {
    fn update(&mut self, game: &mut Game, _timestamp: f32) {
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
                                let player_def = &game.config().players[player_index];
                                let ball_position = game.state.ball_state.position;

                                // Get two random values for power and accuracy
                                let rng_power = self.get_random();
                                let rng_accuracy = self.get_random();

                                // Calculate kick speed with variation
                                let kick_speed =
                                    calculate_kick_velocity(player_def.shot_power, rng_power);

                                // Calculate direction with accuracy-based deviation
                                let (dx, dz) = calculate_kick_direction_with_accuracy(
                                    &target_point,
                                    &ball_position,
                                    player_def.shot_accuracy,
                                    rng_accuracy,
                                );

                                // Set ball velocity (kick along ground, y=0)
                                game.state.ball_state.velocity = Velocity3D::from_meters_per_second(
                                    dx * kick_speed,
                                    0.0,
                                    dz * kick_speed,
                                );

                                // Release possession
                                game.state.ball_state.possessed_by = None;
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
