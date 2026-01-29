use crate::field::Field;
use crate::field::zones::Point3D;
use crate::team::Team;

// Design: PlayerState, BallState, RefereeState are separate despite similar fields (position, velocity).
// Reason: Different systems handle them differently (physics, AI, rules). Shared trait would add
// complexity without benefit since we iterate by type, not across all entities.

#[derive(Debug, Clone)]
pub struct PlayerDef {
    pub team: Team,
}

#[derive(Debug, Clone, Default)]
pub struct BallDef {}

#[derive(Debug, Clone, Default)]
pub struct RefereeDef {}

#[derive(Debug, Clone, Default)]
pub struct PlayerState {
    pub position: Point3D,
    pub velocity: Point3D,
}

#[derive(Debug, Clone, Default)]
pub struct BallState {
    pub position: Point3D,
    pub velocity: Point3D,
}

#[derive(Debug, Clone, Default)]
pub struct RefereeState {
    pub position: Point3D,
    pub velocity: Point3D,
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub field: Field,
    pub players: Vec<PlayerDef>,
    pub ball: BallDef,
    pub referees: Vec<RefereeDef>,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub elapsed_time: f32,
    pub player_states: Vec<PlayerState>,
    pub ball_state: BallState,
    pub referee_states: Vec<RefereeState>,
}

#[derive(Debug, Clone)]
pub enum GameEvent {}

pub struct Game {
    config: GameConfig,
    state: GameState,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let player_states = config.players.iter()
            .map(|_| PlayerState::default())
            .collect();

        Self {
            state: GameState {
                elapsed_time: 0.0,
                player_states,
                ball_state: BallState::default(),
            },
            config,
        }
    }

    pub fn step(&mut self, delta_time: f32) -> Vec<GameEvent> {
        self.state.elapsed_time += delta_time;
        Vec::new()
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> GameConfig {
        GameConfig {
            field: Field::from_meters(100.0, 60.0),
            players: vec![
                PlayerDef { team: Team::A },
                PlayerDef { team: Team::A },
                PlayerDef { team: Team::B },
            ],
            ball: BallDef::default(),
        }
    }

    #[test]
    fn test_state_indices_match_config() {
        let config = create_test_config();
        let player_count = config.players.len();
        
        let game = Game::new(config);
        
        assert_eq!(game.state().player_states.len(), player_count);
    }

    #[test]
    fn test_step_updates_time() {
        let config = create_test_config();
        let mut game = Game::new(config);
        
        game.step(0.016);
        assert!((game.state().elapsed_time - 0.016).abs() < 0.001);
    }
}
