pub mod field_builder;

use crate::config::SerializableGameConfig;
use crate::game::{BallDef, Game, GameConfig, PlayerDef, RefereeDef};
use crate::region::{GridCell, Region};
use crate::systems::{DecisionSystem, PlayerReactionSystem};
use crate::team::Team;
use crate::world::World;

use field_builder::create_football_field;

fn create_football_game_config() -> GameConfig {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();

    let mut players = Vec::new();
    for i in 0..11 {
        let row = i + 1;
        let start_region = Region::new(
            Team::A,
            GridCell::new(1, row).unwrap(),
            GridCell::new(2, row).unwrap(),
            grid_dims,
        )
        .unwrap();

        players.push(PlayerDef::new(
            Team::A,
            i + 1,
            format!("Player A{}", i + 1),
            50,
            50,
            start_region,
        ));
    }
    for i in 0..11 {
        let row = i + 1;
        let start_region = Region::new(
            Team::B,
            GridCell::new(25, row).unwrap(),
            GridCell::new(26, row).unwrap(),
            grid_dims,
        )
        .unwrap();

        players.push(PlayerDef::new(
            Team::B,
            i + 1,
            format!("Player B{}", i + 1),
            50,
            50,
            start_region,
        ));
    }

    GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
    }
}

fn create_football_game_config_from_file(path: &std::path::Path) -> Result<GameConfig, String> {
    let config = SerializableGameConfig::from_file(path)?;
    let field = create_football_field();
    config.to_game_config(field)
}

fn create_football_game_config_from_toml(toml_str: &str) -> Result<GameConfig, String> {
    let config = SerializableGameConfig::from_toml(toml_str)?;
    let field = create_football_field();
    config.to_game_config(field)
}

fn add_football_systems(world: &mut World) {
    world.add_system(Box::new(PlayerReactionSystem));
    world.add_system(Box::new(DecisionSystem::new()));
}

pub fn create_football_world() -> World {
    let game_config = create_football_game_config();
    let game = Game::new(game_config);
    let mut world = World::new(game);
    add_football_systems(&mut world);
    world
}

pub fn create_football_world_from_file(path: &std::path::Path) -> Result<World, String> {
    let game_config = create_football_game_config_from_file(path)?;
    let game = Game::new(game_config);
    let mut world = World::new(game);
    add_football_systems(&mut world);
    Ok(world)
}

pub fn create_football_world_from_toml(toml_str: &str) -> Result<World, String> {
    let game_config = create_football_game_config_from_toml(toml_str)?;
    let game = Game::new(game_config);
    let mut world = World::new(game);
    add_football_systems(&mut world);
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_football_world() {
        let world = create_football_world();

        assert_eq!(world.game().config().players.len(), 22);
        assert_eq!(world.game().state().elapsed_time, 0.0);
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
