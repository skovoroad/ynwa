use crate::game::{Game, GameEvent};
use crate::system::System;

/// World contains the game state and all systems that operate on it.
///
/// World is the main entry point for running a game simulation. It coordinates
/// the game loop by executing all registered systems in sequence.
///
/// # Example
///
/// ```no_run
/// use ynwa_core::football::create_football_world;
/// use uom::si::length::meter;
///
/// // Create a football world with default systems
/// let mut world = create_football_world();
///
/// // Game loop
/// loop {
///     // Update with fixed timestep (16.67ms ≈ 60 FPS)
///     world.step(1.0 / 60.0);
///     
///     // Access game state
///     let game = world.game();
///     println!("Elapsed time: {:.2}s", game.state().elapsed_time);
///     
///     // Check player positions
///     for (i, player) in game.state().player_states.iter().enumerate() {
///         println!("Player {}: ({:.2}, {:.2})", 
///             i, 
///             player.position.x.get::<meter>(), 
///             player.position.z.get::<meter>());
///     }
/// }
/// ```
///
/// Design: World owns Game and systems, coordinating the game loop.
/// Systems receive &mut Game (not &mut World) to avoid borrow checker issues during iteration.
pub struct World {
    game: Game,
    systems: Vec<Box<dyn System>>,
}

impl World {
    pub fn new(game: Game) -> Self {
        Self {
            game,
            systems: Vec::new(),
        }
    }

    /// Systems are executed in the order they are added.
    pub fn add_system(&mut self, system: Box<dyn System>) {
        self.systems.push(system);
    }

    /// Updates timestamp, executes all systems, then updates elapsed_time.
    pub fn step(&mut self, delta_time: f32) -> Vec<GameEvent> {
        let new_timestamp = self.game.state().elapsed_time + delta_time;

        for system in &mut self.systems {
            system.update(&mut self.game, new_timestamp);
        }

        self.game.state.elapsed_time = new_timestamp;

        Vec::new()
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_game() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                50,
                50,
                50,
                "function make_decision() return {} end".to_string(),
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::new(config)
    }

    #[test]
    fn test_world_creation() {
        let game = create_test_game();
        let world = World::new(game);

        assert_eq!(world.game().state().elapsed_time, 0.0);
    }

    #[test]
    fn test_world_step_updates_time() {
        let game = create_test_game();
        let mut world = World::new(game);

        world.step(0.016);

        assert!((world.game().state().elapsed_time - 0.016).abs() < 0.001);
    }

    // Test system for verification
    use std::cell::RefCell;
    use std::rc::Rc;

    struct TestSystem {
        call_count: Rc<RefCell<u32>>,
    }

    impl System for TestSystem {
        fn update(&mut self, _game: &mut Game, _timestamp: f32) {
            *self.call_count.borrow_mut() += 1;
        }
    }

    #[test]
    fn test_world_executes_systems() {
        let game = create_test_game();
        let mut world = World::new(game);

        let call_count = Rc::new(RefCell::new(0));
        let test_system = Box::new(TestSystem {
            call_count: Rc::clone(&call_count),
        });
        world.add_system(test_system);

        world.step(0.016);
        assert_eq!(*call_count.borrow(), 1);

        world.step(0.016);
        assert_eq!(*call_count.borrow(), 2);
    }

    #[test]
    fn test_world_passes_correct_timestamp_to_systems() {
        let game = create_test_game();
        let mut world = World::new(game);

        let received_timestamp = Rc::new(RefCell::new(0.0_f32));

        struct TimestampTestSystem {
            received_timestamp: Rc<RefCell<f32>>,
        }

        impl System for TimestampTestSystem {
            fn update(&mut self, _game: &mut Game, timestamp: f32) {
                *self.received_timestamp.borrow_mut() = timestamp;
            }
        }

        let test_system = Box::new(TimestampTestSystem {
            received_timestamp: Rc::clone(&received_timestamp),
        });
        world.add_system(test_system);

        world.step(0.016);
        assert!((*received_timestamp.borrow() - 0.016).abs() < 0.001);

        world.step(0.016);
        assert!((*received_timestamp.borrow() - 0.032).abs() < 0.001);
    }
}
