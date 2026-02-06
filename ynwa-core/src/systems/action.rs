use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, DecisionTarget, Game};
use crate::physics_util::{calculate_kick_direction_with_accuracy, calculate_kick_velocity};
use crate::region::Region;
use crate::system::System;
use uom::si::length::meter;

#[cfg(test)]
use uom::si::velocity::meter_per_second;

#[cfg(test)]
use crate::physics_util::{KICK_POWER_DIVISOR, KICK_POWER_VARIATION_MIN, KICK_POWER_VARIATION_MAX};

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
                            let player_position =
                                game.state.player_states[player_index].position;

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
                                let kick_speed = calculate_kick_velocity(player_def.shot_power, rng_power);
                                
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
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::GridCell;
    use crate::team::Team;

    fn create_test_game() -> Game {
        create_test_game_with_player_stats(100, 50, 50, 50, 50)
    }

    fn create_test_game_with_player_stats(
        reaction_rate: u32,
        speed_rate: u32,
        tackle_rate: u32,
        shot_power: u32,
        shot_accuracy: u32,
    ) -> Game {
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
            reaction_rate,
            speed_rate,
            tackle_rate,
            shot_power,
            shot_accuracy,
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

    fn create_test_game_with_two_players() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 11);
        let grid_dims = field.grid_dimensions();

        let start_region = crate::region::Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let players = vec![
            PlayerDef::new(
                Team::A,
                1,
                "Player 1".to_string(),
                100,
                50,
                50,
                100,
                100,
                "function make_decision() return {} end".to_string(),
                start_region.clone(),
            ),
            PlayerDef::new(
                Team::A,
                2,
                "Player 2".to_string(),
                100,
                50,
                50,
                100,
                100,
                "function make_decision() return {} end".to_string(),
                start_region,
            ),
        ];

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
    fn test_action_system_processes_stop_decision() {
        let mut game = create_test_game();
        let mut system = ActionSystem::new();

        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 3.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Stop);
        game.state.player_states[0].decision_processed = false;

        system.update(&mut game, 0.0);

        assert_eq!(game.state.player_states[0].velocity.x.get::<meter_per_second>(), 0.0);
        assert_eq!(game.state.player_states[0].velocity.y.get::<meter_per_second>(), 0.0);
        assert_eq!(game.state.player_states[0].velocity.z.get::<meter_per_second>(), 0.0);
        assert!(game.state.player_states[0].decision_processed);
    }

    #[test]
    fn test_action_system_processes_run_to_point_decision() {
        let mut game = create_test_game();
        let mut system = ActionSystem::new();

        let target = Point3D::from_meters(10.0, 0.0, 0.0);
        game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
        game.state.player_states[0].current_decision =
            Some(Decision::Run(DecisionTarget::Point(target)));
        game.state.player_states[0].decision_processed = false;

        system.update(&mut game, 0.0);

        assert!(game.state.player_states[0].velocity.x.get::<meter_per_second>() > 0.0);
        assert_eq!(game.state.player_states[0].velocity.y.get::<meter_per_second>(), 0.0);
        assert_eq!(game.state.player_states[0].velocity.z.get::<meter_per_second>(), 0.0);
        assert!(game.state.player_states[0].decision_processed);
    }

    #[test]
    fn test_action_system_processes_run_to_cell_decision() {
        let mut game = create_test_game();
        let mut system = ActionSystem::new();

        let cell = GridCell::new(5, 5).unwrap();
        game.state.player_states[0].current_decision =
            Some(Decision::Run(DecisionTarget::GridCell(cell)));
        game.state.player_states[0].decision_processed = false;

        system.update(&mut game, 0.0);

        let velocity = &game.state.player_states[0].velocity;
        let vx = velocity.x.get::<meter_per_second>();
        let vy = velocity.y.get::<meter_per_second>();
        let vz = velocity.z.get::<meter_per_second>();
        let speed = (vx * vx + vy * vy + vz * vz).sqrt();

        assert!(speed > 0.0);
        assert!(game.state.player_states[0].decision_processed);
    }

    #[test]
    fn test_action_system_skips_processed_decisions() {
        let mut game = create_test_game();
        let mut system = ActionSystem::new();

        game.state.player_states[0].current_decision = Some(Decision::Stop);
        game.state.player_states[0].decision_processed = true;
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 3.0, 0.0);

        system.update(&mut game, 0.0);

        assert_eq!(game.state.player_states[0].velocity.x.get::<meter_per_second>(), 5.0);
        assert_eq!(game.state.player_states[0].velocity.y.get::<meter_per_second>(), 3.0);
    }

    #[test]
    fn test_action_system_skips_no_decision() {
        let mut game = create_test_game();
        let mut system = ActionSystem::new();

        game.state.player_states[0].current_decision = None;
        game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(2.0, 1.0, 0.0);

        system.update(&mut game, 0.0);

        assert_eq!(game.state.player_states[0].velocity.x.get::<meter_per_second>(), 2.0);
        assert_eq!(game.state.player_states[0].velocity.y.get::<meter_per_second>(), 1.0);
    }

    #[test]
    fn test_calculate_velocity_normalized() {
        let from = Point3D::from_meters(0.0, 0.0, 0.0);
        let to = Point3D::from_meters(10.0, 0.0, 0.0);

        let velocity = calculate_velocity(&from, &to, 50);

        let vx = velocity.x.get::<meter_per_second>();
        let vy = velocity.y.get::<meter_per_second>();
        let speed = (vx * vx + vy * vy).sqrt();
        assert!((speed - 5.0).abs() < 0.01);
    }

    #[test]
    fn test_kick_with_perfect_accuracy_no_variation() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);
        
        // Set player as ball owner
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        
        // Kick straight along X axis
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        // Use RNG with no variation (0.5)
        let mut system = ActionSystem::with_rng(|| 0.5);
        system.update(&mut game, 0.0);
        
        // Check ball velocity
        let ball_vel = &game.state.ball_state.velocity;
        let vx = ball_vel.x.get::<meter_per_second>();
        let vz = ball_vel.z.get::<meter_per_second>();
        
        // shot_power=100, rng=0.5 → base velocity (no variation)
        // shot_accuracy=100, rng=0.5 → no deviation, straight along X
        let expected_velocity = 100.0 / KICK_POWER_DIVISOR;
        assert!((vx - expected_velocity).abs() < 0.01);
        assert!(vz.abs() < 0.01);
        
        // Possession should be released
        assert_eq!(game.state.ball_state.possessed_by, None);
        
        // Decision should be processed
        assert!(game.state.player_states[0].decision_processed);
    }

    #[test]
    fn test_kick_with_min_power_variation() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);
        
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        // First call for power (0.0 = min), second for accuracy (0.5 = no deviation)
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = ActionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 { 0.0 } else { 0.5 }
        });
        system.update(&mut game, 0.0);
        
        let ball_vel = &game.state.ball_state.velocity;
        let vx = ball_vel.x.get::<meter_per_second>();
        
        // shot_power=100, rng=0.0 → min variation
        let expected_velocity = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MIN;
        assert!((vx - expected_velocity).abs() < 0.01);
        assert_eq!(game.state.ball_state.possessed_by, None);
    }

    #[test]
    fn test_kick_with_max_power_variation() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);
        
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        // First call for power (1.0 = max), second for accuracy (0.5 = no deviation)
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = ActionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 { 1.0 } else { 0.5 }
        });
        system.update(&mut game, 0.0);
        
        let ball_vel = &game.state.ball_state.velocity;
        let vx = ball_vel.x.get::<meter_per_second>();
        
        // shot_power=100, rng=1.0 → max variation
        let expected_velocity = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MAX;
        assert!((vx - expected_velocity).abs() < 0.01);
        assert_eq!(game.state.ball_state.possessed_by, None);
    }

    #[test]
    fn test_kick_with_poor_accuracy_max_deviation() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 10);
        
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        
        let target = Point3D::from_meters(60.0, 30.0, 0.0); // Kick along X
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        // First call for power (0.5), second for accuracy (1.0 = max positive deviation)
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = ActionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 { 0.5 } else { 1.0 }
        });
        system.update(&mut game, 0.0);
        
        let ball_vel = &game.state.ball_state.velocity;
        let vx = ball_vel.x.get::<meter_per_second>();
        let vz = ball_vel.z.get::<meter_per_second>();
        
        // shot_accuracy=10, rng=1.0 → +45 degrees deviation
        // 45° rotation: cos(45°)≈0.707, sin(45°)≈0.707
        let base_velocity = 100.0 / KICK_POWER_DIVISOR;
        let expected_vx = base_velocity * 0.707;
        let expected_vz = base_velocity * 0.707;
        assert!((vx - expected_vx).abs() < 0.1);
        assert!((vz - expected_vz).abs() < 0.1);
        assert_eq!(game.state.ball_state.possessed_by, None);
    }

    #[test]
    fn test_kick_without_possession_ignored() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);
        
        // Player does NOT own the ball
        game.state.ball_state.possessed_by = None;
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::from_meters_per_second(1.0, 0.0, 2.0);
        
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        let mut system = ActionSystem::with_rng(|| 0.5);
        system.update(&mut game, 0.0);
        
        // Ball velocity should not change (kick ignored)
        let ball_vel = &game.state.ball_state.velocity;
        assert_eq!(ball_vel.x.get::<meter_per_second>(), 1.0);
        assert_eq!(ball_vel.z.get::<meter_per_second>(), 2.0);
        
        // Decision should still be marked as processed
        assert!(game.state.player_states[0].decision_processed);
    }

    #[test]
    fn test_kick_by_different_player_ignored() {
        let mut game = create_test_game_with_two_players();
        
        // Ball owned by player 0
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        game.state.ball_state.velocity = Velocity3D::default();
        
        // Player 1 tries to kick
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[1].current_decision = Some(Decision::Kick(target));
        game.state.player_states[1].decision_processed = false;
        
        let mut system = ActionSystem::with_rng(|| 0.5);
        system.update(&mut game, 0.0);
        
        // Ball velocity should not change (player 1 doesn't own ball)
        let ball_vel = &game.state.ball_state.velocity;
        assert_eq!(ball_vel.x.get::<meter_per_second>(), 0.0);
        assert_eq!(ball_vel.z.get::<meter_per_second>(), 0.0);
        
        // Possession should remain with player 0
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
        
        // Decision should still be marked as processed
        assert!(game.state.player_states[1].decision_processed);
    }

    #[test]
    fn test_kick_preserves_player_velocity() {
        let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);
        
        // Set player as ball owner
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
        
        // Give player some velocity (running)
        let player_velocity = Velocity3D::from_meters_per_second(3.0, 0.0, 2.0);
        game.state.player_states[0].velocity = player_velocity;
        
        // Player kicks while running
        let target = Point3D::from_meters(60.0, 30.0, 0.0);
        game.state.player_states[0].current_decision = Some(Decision::Kick(target));
        game.state.player_states[0].decision_processed = false;
        
        let mut system = ActionSystem::with_rng(|| 0.5);
        system.update(&mut game, 0.0);
        
        // Ball should move
        let ball_vel = &game.state.ball_state.velocity;
        assert!(ball_vel.x.get::<meter_per_second>() > 0.0);
        
        // Possession should be released
        assert_eq!(game.state.ball_state.possessed_by, None);
        
        // Player velocity should be UNCHANGED (continues running)
        assert_eq!(game.state.player_states[0].velocity.x.get::<meter_per_second>(), 3.0);
        assert_eq!(game.state.player_states[0].velocity.y.get::<meter_per_second>(), 0.0);
        assert_eq!(game.state.player_states[0].velocity.z.get::<meter_per_second>(), 2.0);
        
        assert!(game.state.player_states[0].decision_processed);
    }
}
