use crate::game::Game;
use crate::system::System;
use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

// Design: PhysicsSystem applies velocity to position using basic kinematics.
// position_new = position_old + velocity × delta_time

// Ball friction: deceleration when rolling on ground (m/s²)
const BALL_FRICTION_DECELERATION: f32 = 2.0;

pub struct PhysicsSystem {
    last_update: f32,
}

impl PhysicsSystem {
    pub fn new() -> Self {
        Self { last_update: 0.0 }
    }
}

impl Default for PhysicsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for PhysicsSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let delta_time = timestamp - self.last_update;
        self.last_update = timestamp;

        if delta_time <= 0.0 {
            return;
        }

        // Update players
        for player_state in &mut game.state.player_states {
            let vx = player_state.velocity.x.get::<meter_per_second>();
            let vy = player_state.velocity.y.get::<meter_per_second>();
            let vz = player_state.velocity.z.get::<meter_per_second>();

            let dx = vx * delta_time;
            let dy = vy * delta_time;
            let dz = vz * delta_time;

            player_state.position.x += uom::si::f32::Length::new::<meter>(dx);
            player_state.position.y += uom::si::f32::Length::new::<meter>(dy);
            player_state.position.z += uom::si::f32::Length::new::<meter>(dz);
        }

        // Update ball
        let ball_state = &mut game.state.ball_state;

        if let Some(player_index) = ball_state.possessed_by {
            // Ball moves with player
            if player_index < game.state.player_states.len() {
                ball_state.position = game.state.player_states[player_index].position;
                ball_state.velocity = crate::field::zones::Velocity3D::default();
            }
        } else {
            // Ball rolls freely with friction
            let vx = ball_state.velocity.x.get::<meter_per_second>();
            let vy = ball_state.velocity.y.get::<meter_per_second>();
            let vz = ball_state.velocity.z.get::<meter_per_second>();

            // Calculate speed (magnitude of velocity)
            let speed = (vx * vx + vy * vy + vz * vz).sqrt();

            if speed > 0.0 {
                // Apply friction deceleration
                let deceleration = BALL_FRICTION_DECELERATION * delta_time;
                let new_speed = (speed - deceleration).max(0.0);

                // Scale velocity to new speed
                let scale = new_speed / speed;
                ball_state.velocity.x = uom::si::f32::Velocity::new::<meter_per_second>(vx * scale);
                ball_state.velocity.y = uom::si::f32::Velocity::new::<meter_per_second>(vy * scale);
                ball_state.velocity.z = uom::si::f32::Velocity::new::<meter_per_second>(vz * scale);
            }

            // Apply velocity to position
            let dx = ball_state.velocity.x.get::<meter_per_second>() * delta_time;
            let dy = ball_state.velocity.y.get::<meter_per_second>() * delta_time;
            let dz = ball_state.velocity.z.get::<meter_per_second>() * delta_time;

            ball_state.position.x += uom::si::f32::Length::new::<meter>(dx);
            ball_state.position.y += uom::si::f32::Length::new::<meter>(dy);
            ball_state.position.z += uom::si::f32::Length::new::<meter>(dz);
        }
    }
}

#[cfg(test)]
#[path = "../tests/physics_tests.rs"]
mod tests;
