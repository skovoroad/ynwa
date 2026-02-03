use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::region::{GridCell, Region};
use crate::team::Team;
use std::collections::HashMap;
use uom::si::length::meter;

// Design: PlayerState, BallState, RefereeState are separate despite similar fields (position, velocity).
// Reason: Different systems handle them differently (physics, AI, rules). Shared trait would add
// complexity without benefit since we iterate by type, not across all entities.

#[derive(Debug, Clone)]
pub enum DecisionTarget {
    Region(Region),
    GridCell(GridCell),
    Point(Point3D),
}

#[derive(Debug, Clone)]
pub enum Decision {
    Run(DecisionTarget),
    Stop,
}

#[derive(Debug, Clone)]
pub struct PlayerDef {
    pub team: Team,
    pub number: u32,
    pub name: String,
    pub reaction_rate: u32, // 10-100: player's reaction speed
    pub speed_rate: u32,    // 10-100: player's movement speed
    pub script: String,     // Lua script for decision making
    pub regions: HashMap<String, Region>,
}

impl PlayerDef {
    pub fn new(
        team: Team,
        number: u32,
        name: String,
        reaction_rate: u32,
        speed_rate: u32,
        script: String,
        start_position: Region,
    ) -> Self {
        let mut regions = HashMap::new();
        regions.insert("start position".to_string(), start_position);

        Self {
            team,
            number,
            name,
            reaction_rate,
            speed_rate,
            script,
            regions,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BallDef {}

#[derive(Debug, Clone, Default)]
pub struct RefereeDef {}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub position: Point3D,
    pub velocity: Velocity3D,
    pub last_decision_time: f32,
    pub needs_decision: bool,
    pub current_decision: Option<Decision>,
    pub decision_processed: bool,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Point3D::default(),
            velocity: Velocity3D::default(),
            last_decision_time: 0.0,
            needs_decision: true,
            current_decision: None,
            decision_processed: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct BallState {
    pub position: Point3D,
    pub velocity: Velocity3D,
}

#[derive(Debug, Clone, Default)]
pub struct RefereeState {
    pub position: Point3D,
    pub velocity: Velocity3D,
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
    pub(crate) state: GameState,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        let player_states = config
            .players
            .iter()
            .map(|player_def| {
                let start_region = player_def
                    .regions
                    .get("start position")
                    .expect("Player must have 'start position' region");
                let position = start_region.center(
                    config.field.grid_dimensions(),
                    config.field.width().get::<meter>(),
                );
                PlayerState {
                    position,
                    velocity: Velocity3D::default(),
                    last_decision_time: 0.0,
                    needs_decision: true,
                    current_decision: None,
                    decision_processed: false,
                }
            })
            .collect();
        let referee_states = config
            .referees
            .iter()
            .map(|_| RefereeState::default())
            .collect();

        Self {
            state: GameState {
                elapsed_time: 0.0,
                player_states,
                ball_state: BallState::default(),
                referee_states,
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
    use crate::region::GridCell;

    fn create_test_config() -> GameConfig {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Create start position regions for test players
        let start_region_a1 = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();

        let start_region_a2 = Region::new(
            Team::A,
            GridCell::new(3, 3).unwrap(),
            GridCell::new(4, 4).unwrap(),
            grid_dims,
        )
        .unwrap();

        let start_region_b = Region::new(
            Team::B,
            GridCell::new(20, 20).unwrap(),
            GridCell::new(21, 21).unwrap(),
            grid_dims,
        )
        .unwrap();

        GameConfig {
            field,
            players: vec![
                PlayerDef::new(Team::A, 1, "Player A1".to_string(), 50, 50, "function make_decision() return {} end".to_string(), start_region_a1),
                PlayerDef::new(Team::A, 2, "Player A2".to_string(), 50, 50, "function make_decision() return {} end".to_string(), start_region_a2),
                PlayerDef::new(Team::B, 1, "Player B1".to_string(), 50, 50, "function make_decision() return {} end".to_string(), start_region_b),
            ],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
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

    #[test]
    fn test_player_initial_position_from_start_region() {
        let config = create_test_config();
        let game = Game::new(config);

        // Calculate expected positions from regions
        let cell_width =
            game.config().field.width().get::<meter>() / game.config().field.grid_columns() as f32;

        // Player A1: region (1,1) to (2,2) -> center at (1.5, 1.5) in grid coords
        // Grid coords to meters: (col-1)*cell_width for min, col*cell_width for max
        // Center X: ((1-1) + 2) / 2 * cell_width = 1.0 * cell_width
        // Center Z: ((1-1) + 2) / 2 * cell_width = 1.0 * cell_width
        let expected_a1_x = 1.0 * cell_width;
        let expected_a1_z = 1.0 * cell_width;

        // Player A2: region (3,3) to (4,4) -> center at (3.5, 3.5) in grid coords
        let expected_a2_x = 3.0 * cell_width;
        let expected_a2_z = 3.0 * cell_width;

        // Player B1: region (20,20) to (21,21) -> center at (20.5, 20.5) in grid coords
        let expected_b1_x = 20.0 * cell_width;
        let expected_b1_z = 20.0 * cell_width;

        // Check actual positions
        assert_eq!(game.state().player_states.len(), 3);

        let tolerance = 0.01; // 1cm tolerance for floating point comparison

        assert!(
            (game.state().player_states[0].position.x.get::<meter>() - expected_a1_x).abs()
                < tolerance
        );
        assert!(
            (game.state().player_states[0].position.z.get::<meter>() - expected_a1_z).abs()
                < tolerance
        );

        assert!(
            (game.state().player_states[1].position.x.get::<meter>() - expected_a2_x).abs()
                < tolerance
        );
        assert!(
            (game.state().player_states[1].position.z.get::<meter>() - expected_a2_z).abs()
                < tolerance
        );

        assert!(
            (game.state().player_states[2].position.x.get::<meter>() - expected_b1_x).abs()
                < tolerance
        );
        assert!(
            (game.state().player_states[2].position.z.get::<meter>() - expected_b1_z).abs()
                < tolerance
        );

        // All players should be on the ground (Y = 0)
        for player_state in &game.state().player_states {
            assert_eq!(player_state.position.y.get::<meter>(), 0.0);
        }
    }
}
