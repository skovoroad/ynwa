use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::region::{GridCell, Region};
use crate::team::Team;
use std::collections::HashMap;
use uom::si::length::meter;

/// Named numeric statistics. Keys are game-specific (defined by game managers, not core).
#[derive(Debug, Clone, Default)]
pub struct StatSet {
    values: HashMap<String, f64>,
}

impl StatSet {
    pub fn get(&self, key: &str) -> f64 {
        *self.values.get(key).unwrap_or(&0.0)
    }

    pub fn set(&mut self, key: &str, value: f64) {
        self.values.insert(key.to_string(), value);
    }

    pub fn increment(&mut self, key: &str, delta: f64) {
        *self.values.entry(key.to_string()).or_insert(0.0) += delta;
    }
}

// Design: PlayerState, BallState, RefereeState are separate despite similar fields (position, velocity).
// Reason: Different systems handle them differently (physics, AI, rules). Shared trait would add
// complexity without benefit since we iterate by type, not across all entities.

#[derive(Debug, Clone)]
pub enum DecisionTarget {
    Region(Region),
    GridCell(GridCell),
    Point(Point3D),
    /// Chase the ball: target point is resolved to the ball's current position each tick.
    Ball,
}

#[derive(Debug, Clone)]
pub enum Decision {
    Run(DecisionTarget),
    Stop,
    Kick(Point3D), // Kick the ball towards target point
}

/// Region key required in `PlayerDef::regions` when starting in `GameStage::Play`.
/// Contract between core and game-specific layers (e.g. `ynwa-football`).
pub const REGION_START_POSITION: &str = "start";

#[derive(Debug, Clone)]
pub struct PlayerDef {
    pub team: Team,
    pub number: u32,
    pub name: String,
    pub reaction_rate: u32,
    pub speed_rate: u32,
    pub tackle_rate: u32,
    pub shot_power: u32,
    pub shot_accuracy: u32,
    pub script: String,
    pub regions: HashMap<String, Region>,
}

impl PlayerDef {
    pub fn new(
        team: Team,
        number: u32,
        name: String,
        script: String,
        regions: HashMap<String, Region>,
    ) -> Self {
        Self {
            team,
            number,
            name,
            reaction_rate: 50,
            speed_rate: 50,
            tackle_rate: 50,
            shot_power: 50,
            shot_accuracy: 50,
            script,
            regions,
        }
    }

    pub fn with_reaction_rate(mut self, rate: u32) -> Self {
        self.reaction_rate = rate;
        self
    }

    pub fn with_speed_rate(mut self, rate: u32) -> Self {
        self.speed_rate = rate;
        self
    }

    pub fn with_tackle_rate(mut self, rate: u32) -> Self {
        self.tackle_rate = rate;
        self
    }

    pub fn with_shot_power(mut self, power: u32) -> Self {
        self.shot_power = power;
        self
    }

    pub fn with_shot_accuracy(mut self, accuracy: u32) -> Self {
        self.shot_accuracy = accuracy;
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct BallDef {
    pub initial_position: Point3D,
}

#[derive(Debug, Clone, Default)]
pub struct RefereeDef {}

#[derive(Debug, Clone)]
pub struct PlayerState {
    pub position: Point3D,
    pub velocity: Velocity3D,
    pub last_decision_time: f32,
    pub needs_decision: bool,
    pub current_decision: Option<Decision>,
    pub decision_reason: Option<String>, // Short explanation of why this decision was made
    pub decision_processed: bool,
    pub last_error: Option<String>,
    pub is_ready: bool, // True when player has reached their Setup target (current_decision is Stop)
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            position: Point3D::default(),
            velocity: Velocity3D::default(),
            last_decision_time: 0.0,
            needs_decision: true,
            current_decision: None,
            decision_reason: None,
            decision_processed: false,
            last_error: None,
            is_ready: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BallState {
    pub position: Point3D,
    pub velocity: Velocity3D,
    /// Index of player possessing the ball, or None if ball is free
    pub possessed_by: Option<usize>,
    /// Timestamp of last possession change (for anti-bounce)
    pub last_possession_change_time: f32,
    /// Team that last possessed the ball, or None if ball was never possessed
    pub last_possessing_team: Option<Team>,
}

impl Default for BallState {
    fn default() -> Self {
        Self {
            position: Point3D::default(),
            velocity: Velocity3D::default(),
            possessed_by: None,
            last_possession_change_time: 0.0,
            last_possessing_team: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RefereeState {
    pub position: Point3D,
    pub velocity: Velocity3D,
}

#[derive(Debug, Clone)]
pub struct ScriptingConfig {
    pub core_preamble: String,
    pub stdlib_preamble: String,
    pub team_a_preamble: String,
    pub team_b_preamble: String,
}

impl ScriptingConfig {
    pub fn empty() -> Self {
        Self {
            core_preamble: String::new(),
            stdlib_preamble: String::new(),
            team_a_preamble: String::new(),
            team_b_preamble: String::new(),
        }
    }

    pub fn team_preamble(&self, team: Team) -> &str {
        match team {
            Team::A => &self.team_a_preamble,
            Team::B => &self.team_b_preamble,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum GameStage {
    Play,
    Setup(String),
    GameOver,
}

impl Default for GameStage {
    fn default() -> Self {
        GameStage::Setup("kick off".to_string())
    }
}

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub field: Field,
    pub players: Vec<PlayerDef>,
    pub ball: BallDef,
    pub referees: Vec<RefereeDef>,
    pub scripting: ScriptingConfig,
}

#[derive(Debug, Clone)]
pub struct GameState {
    pub elapsed_time: f32,
    pub stage: GameStage,
    pub player_states: Vec<PlayerState>,
    pub ball_state: BallState,
    pub referee_states: Vec<RefereeState>,
    /// Indexed by Team. Populated by game-specific managers (e.g. FootballGameManager).
    pub team_stats: HashMap<Team, StatSet>,
    /// Parallel to player_states: player_stats[i] corresponds to player_states[i].
    pub player_stats: Vec<StatSet>,
    /// Ball placement point for current Setup stage. None means use ball.initial_position.
    pub restart_position: Option<Point3D>,
    /// Team that initiates play at current Setup stage restart. None for "start".
    pub restart_team: Option<Team>,
}

pub struct Game {
    config: GameConfig,
    pub state: GameState,
}

impl Game {
    pub fn new(config: GameConfig) -> Self {
        Self::with_stage(config, GameStage::default())
    }

    pub fn with_stage(config: GameConfig, stage: GameStage) -> Self {
        let player_states = config
            .players
            .iter()
            .enumerate()
            .map(|(idx, _player_def)| {
                let position = match &stage {
                    GameStage::Setup(_) => {
                        // Players start off the side of the field (x = -5), centered along field length (Z axis)
                        let field_length = config.field.length().get::<meter>();
                        Point3D::from_meters(
                            -5.0,
                            0.0,
                            field_length / 2.0,
                        )
                    }
                    GameStage::Play | GameStage::GameOver => {
                        let start_region = config.players[idx]
                            .regions
                            .get(REGION_START_POSITION)
                            .expect("Player must have 'start' region");
                        start_region.center(
                            config.field.grid_dimensions(),
                            config.field.width().get::<meter>(),
                        )
                    }
                };

                PlayerState {
                    position,
                    velocity: Velocity3D::default(),
                    last_decision_time: 0.0,
                    needs_decision: true,
                    current_decision: None,
                    decision_reason: None,
                    decision_processed: false,
                    last_error: None,
                    is_ready: false,
                }
            })
            .collect();
        let referee_states = config
            .referees
            .iter()
            .map(|_| RefereeState::default())
            .collect();

        let team_stats = config
            .players
            .iter()
            .map(|p| p.team)
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .map(|t| (t, StatSet::default()))
            .collect();

        Self {
            state: GameState {
                elapsed_time: 0.0,
                stage,
                player_states,
                ball_state: BallState {
                    position: config.ball.initial_position,
                    velocity: Velocity3D::default(),
                    possessed_by: None,
                    last_possession_change_time: -1.0, // Set to negative so first change is allowed
                    last_possessing_team: None,        // Neutral at game start
                },
                referee_states,
                team_stats,
                player_stats: vec![StatSet::default(); config.players.len()],
                restart_position: None,
                restart_team: None,
            },
            config,
        }
    }

    pub fn step(&mut self, delta_time: f32) {
        self.state.elapsed_time += delta_time;
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }

    pub fn config(&self) -> &GameConfig {
        &self.config
    }
}

#[cfg(test)]
#[path = "tests/game_tests.rs"]
mod tests;
