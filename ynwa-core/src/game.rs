use crate::field::Field;

pub struct GameConfig {
    pub field: Field,
}

// Minimal state - just what we need now
#[derive(Debug, Clone)]
pub struct GameState {
    pub elapsed_time: f32,
}

#[derive(Debug, Clone)]
pub enum GameEvent {
    // Empty for now
}

pub struct Game {
    #[allow(dead_code)]
    config: GameConfig,
    state: GameState,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        Self {
            config,
            state: GameState {
                elapsed_time: 0.0,
            },
        }
    }

    pub fn step(&mut self, delta_time: f32) -> Vec<GameEvent> {
        self.state.elapsed_time += delta_time;
        Vec::new()
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;

    fn create_test_field() -> Field {
        Field::from_meters(100.0, 60.0)
    }

    #[test]
    fn test_game_creation() {
        let field = create_test_field();
        let game = Game::new(GameConfig { field });
        assert_eq!(game.state().elapsed_time, 0.0);
    }

    #[test]
    fn test_step_updates_time() {
        let field = create_test_field();
        let mut game = Game::new(GameConfig { field });
        
        game.step(0.016);
        assert!((game.state().elapsed_time - 0.016).abs() < 0.001);
    }
}
