use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
use crate::region::{GridCell};
use crate::team::Team;

fn create_test_game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap()).unwrap();

    let config = GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
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
