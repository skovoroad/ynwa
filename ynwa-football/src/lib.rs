//! YNWA Football - Football rules and world factory.
//!
//! This crate implements football-specific rules on top of `ynwa-core`:
//! - `field_builder` - standard football field with FIFA regulation zones
//! - `game_manager` - `FootballGameManager` system: stage transitions, player readiness
//! - `events` - goal detection, out-of-bounds, game end
//!
//! Entry point: `create_football_world_from_file` creates a ready-to-use `World`.

pub mod events;
pub mod field_builder;
pub mod game_manager;

use field_builder::create_football_field;
use game_manager::FootballGameManager;
use ynwa_core::config::SerializableGameConfig;
use ynwa_core::field::zones::ZoneGeometry;
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage};
use ynwa_core::systems::decision::ScriptedDecisionMaker;
use ynwa_core::systems::{
    ActionSystem, BallPossessionSystem, DecisionSystem, PhysicsSystem, PlayerReactionSystem,
};
use ynwa_core::world::World;

/// Extract ball initial position from center_spot zone (football rule)
fn get_ball_initial_position(field: &ynwa_core::field::Field) -> ynwa_core::field::zones::Point3D {
    let center_spot_zone = field
        .get_zone("center_spot", None)
        .expect("Football field must have center_spot zone");

    match &center_spot_zone.geometry {
        ZoneGeometry::Point(point) => point.position,
        _ => panic!("center_spot must be a Point zone"),
    }
}

fn create_football_game_config_from_file(path: &std::path::Path) -> Result<GameConfig, String> {
    let config = SerializableGameConfig::from_file(path)?;
    let field = create_football_field();

    let config_dir = path.parent();
    let game_config = config.to_game_config(field, config_dir)?;

    let mut game_config = game_config;
    game_config.ball = BallDef {
        initial_position: get_ball_initial_position(&game_config.field),
    };

    Ok(game_config)
}

fn add_football_systems(world: &mut World) {
    world.add_system(Box::new(FootballGameManager::new()));
    world.add_system(Box::new(PlayerReactionSystem));
    world.add_system(Box::new(BallPossessionSystem::new()));

    let decision_system = match ScriptedDecisionMaker::new(world.game()) {
        Ok(scripted_maker) => {
            println!(
                "Successfully initialized ScriptedDecisionMaker for {} players",
                world.game().config().players.len()
            );
            DecisionSystem::new().with_decision_maker(Box::new(scripted_maker))
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to create ScriptedDecisionMaker: {}. Using placeholder.",
                e
            );
            DecisionSystem::new()
        }
    };

    world.add_system(Box::new(decision_system));
    world.add_system(Box::new(ActionSystem::new()));
    world.add_system(Box::new(PhysicsSystem::new()));
}

pub fn create_football_world_from_file(path: &std::path::Path) -> Result<World, String> {
    let game_config = create_football_game_config_from_file(path)?;
    let game = Game::with_stage(game_config, GameStage::Setup("Prepare".to_string()));
    let mut world = World::new(game);
    add_football_systems(&mut world);
    Ok(world)
}

#[cfg(test)]
    #[path = "tests/field_builder_tests.rs"]
    mod field_builder_tests;#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;
    use ynwa_core::game::*;
    use ynwa_core::region::*;
    use ynwa_core::team::Team;

    fn create_football_game_config() -> GameConfig {
        let field = create_football_field();
        let grid_dims = field.grid_dimensions();
        let ball_initial_position = get_ball_initial_position(&field);

        let mut players = Vec::new();
        for i in 0..11 {
            let row = i + 1;
            let start_region = grid_dims
                .create_region(
                    GridCell::new(1, row).unwrap(),
                    GridCell::new(2, row).unwrap(),
                )
                .unwrap();

            players.push(PlayerDef::new(
                Team::A,
                i + 1,
                format!("Player A{}", i + 1),
                "function make_decision() return {} end".to_string(),
                start_region,
            ));
        }
        for i in 0..11 {
            let row = i + 1;
            let start_region = grid_dims
                .create_region(
                    GridCell::new(25, row).unwrap(),
                    GridCell::new(26, row).unwrap(),
                )
                .unwrap();

            players.push(PlayerDef::new(
                Team::B,
                i + 1,
                format!("Player B{}", i + 1),
                "function make_decision() return {} end".to_string(),
                start_region,
            ));
        }

        GameConfig {
            field,
            players,
            ball: BallDef {
                initial_position: ball_initial_position,
            },
            referees: vec![RefereeDef::default()],
            scripting: ynwa_core::game::ScriptingConfig::empty(),
        }
    }

    pub fn create_football_world() -> World {
        let game_config = create_football_game_config();
        let game = Game::with_stage(game_config, GameStage::Setup("Prepare".to_string()));
        let mut world = World::new(game);
        add_football_systems(&mut world);
        world
    }

    #[test]
    fn test_create_football_world() {
        let world = create_football_world();

        assert_eq!(world.game().config().players.len(), 22);
        assert_eq!(world.game().state().elapsed_time, 0.0);
    }

    #[test]
    fn test_ball_initial_position_at_center_spot() {
        let world = create_football_world();
        let game = world.game();

        let center_spot = game
            .config()
            .field
            .get_zone("center_spot", None)
            .expect("Football field must have center_spot");

        let expected_position = match &center_spot.geometry {
            ynwa_core::field::zones::ZoneGeometry::Point(point) => &point.position,
            _ => panic!("center_spot must be a Point zone"),
        };

        let ball_position = &game.state().ball_state.position;
        assert_eq!(ball_position.x.get::<meter>(), expected_position.x.get::<meter>());
        assert_eq!(ball_position.y.get::<meter>(), expected_position.y.get::<meter>());
        assert_eq!(ball_position.z.get::<meter>(), expected_position.z.get::<meter>());
    }

    #[test]
    fn test_create_football_world_from_file() {
        let path = std::path::Path::new("../config/default_game.toml");
        if !path.exists() {
            println!("Skipping test - config file not found at {:?}", path);
            return;
        }

        let world =
            create_football_world_from_file(path).expect("Failed to create world from file");

        assert!(!world.game().config().players.is_empty());
    }
}
