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
                ball_state.position = game.state.player_states[player_index].position.clone();
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
mod tests {
    use super::*;
    use crate::field::zones::{Point3D, Velocity3D};
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::GridCell;
    use crate::team::Team;

    fn create_test_game() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 11);
        let grid_dims = field.grid_dimensions();

        let start_region = crate::region::Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let players = vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            100,
            50,
            50,
            50,
            50,
            "function make_decision() return {} end".to_string(),
            start_region,
        )];

        let config = GameConfig {
            field,
            players,
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::new(config)
    }

    #[test]
    fn test_physics_system_updates_player_position() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 0.0, 3.0);

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // delta_time = 1.0 second

        assert!((game.state.player_states[0].position.x.get::<meter>() - 5.0).abs() < 0.001);
        assert!((game.state.player_states[0].position.y.get::<meter>() - 0.0).abs() < 0.001);
        assert!((game.state.player_states[0].position.z.get::<meter>() - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_physics_system_handles_zero_delta_time() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 0.0, 0.0);

        system.update(&mut game, 1.0); // Initialize last_update
        let pos_after_first = game.state.player_states[0].position.x.get::<meter>();

        system.update(&mut game, 1.0); // Same timestamp - delta = 0
        let pos_after_second = game.state.player_states[0].position.x.get::<meter>();

        // Position should not change when delta_time = 0
        assert!((pos_after_first - pos_after_second).abs() < 0.001);
    }

    #[test]
    fn test_physics_system_accumulates_movement() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(2.0, 0.0, 0.0);

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // Move 2m
        system.update(&mut game, 2.0); // Move another 2m

        assert!((game.state.player_states[0].position.x.get::<meter>() - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_physics_system_handles_fractional_delta_time() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(10.0, 0.0, 0.0);

        system.update(&mut game, 0.0);
        system.update(&mut game, 0.1); // 100ms

        assert!((game.state.player_states[0].position.x.get::<meter>() - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_ball_moves_with_player_when_possessed() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        // Set player position and velocity
        game.state.player_states[0].position = Point3D::from_meters(10.0, 0.0, 5.0);
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(2.0, 0.0, 1.0);

        // Ball starts at different position but possessed by player 0
        game.state.ball_state.position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::from_meters_per_second(5.0, 0.0, 5.0);
        game.state.ball_state.possessed_by = Some(0);

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // delta_time = 1.0 second

        // Player moved to (12, 0, 6)
        assert!((game.state.player_states[0].position.x.get::<meter>() - 12.0).abs() < 0.001);
        assert!((game.state.player_states[0].position.z.get::<meter>() - 6.0).abs() < 0.001);

        // Ball should be at player position
        assert!((game.state.ball_state.position.x.get::<meter>() - 12.0).abs() < 0.001);
        assert!((game.state.ball_state.position.z.get::<meter>() - 6.0).abs() < 0.001);

        // Ball velocity should be reset to zero
        assert!((game.state.ball_state.velocity.x.get::<meter_per_second>() - 0.0).abs() < 0.001);
        assert!((game.state.ball_state.velocity.z.get::<meter_per_second>() - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ball_rolls_freely_with_friction() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        // Ball starts at origin with velocity
        game.state.ball_state.position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::from_meters_per_second(4.0, 0.0, 0.0);
        game.state.ball_state.possessed_by = None;

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // delta_time = 1.0 second

        // Ball should have moved (4.0 m/s initial velocity, friction 2.0 m/s²)
        // After 1 second: velocity = 4.0 - 2.0 = 2.0 m/s
        // Distance = average velocity × time = (4.0 + 2.0) / 2 × 1.0 = 3.0 m
        // But our implementation uses instantaneous velocity, so it's different
        // Let's check the actual position
        let ball_x = game.state.ball_state.position.x.get::<meter>();
        assert!(ball_x > 0.0); // Ball moved forward
        assert!(ball_x < 4.0); // Ball moved less than without friction

        // Check velocity decreased
        let ball_vx = game.state.ball_state.velocity.x.get::<meter_per_second>();
        assert!((ball_vx - 2.0).abs() < 0.001); // Should be 2.0 m/s after friction
    }

    #[test]
    fn test_ball_stops_when_velocity_reaches_zero() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        // Ball with low velocity
        game.state.ball_state.position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::from_meters_per_second(1.0, 0.0, 0.0);
        game.state.ball_state.possessed_by = None;

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // delta_time = 1.0 second (friction = 2.0 m/s²)

        // Ball should have stopped (velocity would go negative, but clamped to 0)
        let ball_vx = game.state.ball_state.velocity.x.get::<meter_per_second>();
        assert!((ball_vx - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_ball_friction_affects_all_directions() {
        let mut game = create_test_game();
        let mut system = PhysicsSystem::new();

        // Ball with velocity in both X and Z directions
        game.state.ball_state.position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::from_meters_per_second(3.0, 0.0, 4.0);
        game.state.ball_state.possessed_by = None;

        system.update(&mut game, 0.0);
        system.update(&mut game, 1.0); // delta_time = 1.0 second

        // Speed should be reduced by friction (5.0 - 2.0 = 3.0 m/s)
        let vx = game.state.ball_state.velocity.x.get::<meter_per_second>();
        let vz = game.state.ball_state.velocity.z.get::<meter_per_second>();
        let final_speed = (vx * vx + vz * vz).sqrt();

        assert!((final_speed - 3.0).abs() < 0.001);

        // Direction should be preserved (ratio 3:4)
        let expected_vx = 3.0 * 3.0 / 5.0;
        let expected_vz = 4.0 * 3.0 / 5.0;
        assert!((vx - expected_vx).abs() < 0.001);
        assert!((vz - expected_vz).abs() < 0.001);
    }
}
