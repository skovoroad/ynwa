use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, DecisionTarget, Game};
use crate::region::Region;
use crate::system::System;
use uom::si::length::meter;

#[cfg(test)]
use uom::si::velocity::meter_per_second;

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

pub struct ActionSystem;

impl ActionSystem {
    pub fn new() -> Self {
        Self
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
                        Decision::Kick(_target_point) => {
                            // TODO: Implement kick mechanics
                            // For now, just stop the player (no action taken)
                            game.state.player_states[player_index].velocity = Velocity3D::default();
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
            50, 50, "function make_decision() return {} end".to_string(),
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
}
