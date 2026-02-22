use crate::game::Game;
use crate::system::System;

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
    pub fn step(&mut self, delta_time: f32) {
        let new_timestamp = self.game.state().elapsed_time + delta_time;

        for system in &mut self.systems {
            system.update(&mut self.game, new_timestamp);
        }

        self.game.state.elapsed_time = new_timestamp;
    }

    pub fn game(&self) -> &Game {
        &self.game
    }

    pub fn game_mut(&mut self) -> &mut Game {
        &mut self.game
    }
}

#[cfg(test)]
#[path = "tests/world_tests.rs"]
mod tests;
