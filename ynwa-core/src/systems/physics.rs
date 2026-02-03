use crate::game::Game;
use crate::system::System;
use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

// Design: PhysicsSystem applies velocity to position using basic kinematics.
// position_new = position_old + velocity × delta_time

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

            player_state.position.x = player_state.position.x + uom::si::f32::Length::new::<meter>(dx);
            player_state.position.y = player_state.position.y + uom::si::f32::Length::new::<meter>(dy);
            player_state.position.z = player_state.position.z + uom::si::f32::Length::new::<meter>(dz);
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
            start_region,
        )];

        let config = GameConfig {
            field,
            players,
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
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

        system.update(&mut game, 1.0);  // Initialize last_update
        let pos_after_first = game.state.player_states[0].position.x.get::<meter>();
        
        system.update(&mut game, 1.0);  // Same timestamp - delta = 0
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
}
