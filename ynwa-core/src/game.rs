use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::orientation::flip_point_orientation;
use crate::region::{GridCell, GridDimensions, Region};
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
    Kick(Point3D), // Kick the ball towards target point
}

/// Converts Team B coordinates to display orientation (Team A perspective).
/// Team A decisions pass through unchanged.
pub fn convert_decision_to_display_orientation(
    decision: &Decision,
    team: Team,
    field_width: f32,
    field_height: f32,
    grid_dimensions: GridDimensions,
) -> Decision {
    // Team A is already in display orientation, no conversion needed
    if team == Team::A {
        return decision.clone();
    }

    // Team B: flip all coordinates to Team A orientation
    match decision {
        Decision::Run(target) => {
            let flipped_target = match target {
                DecisionTarget::Region(region) => {
                    // Flip the coordinates but keep team as A (display orientation)
                    let flipped = region.flip_orientation(grid_dimensions).unwrap();
                    // Region::flip_orientation swaps the team, but we want all display coords to be Team A
                    let display_region =
                        Region::new_unchecked(Team::A, flipped.top_left, flipped.bottom_right);
                    DecisionTarget::Region(display_region)
                }
                DecisionTarget::GridCell(cell) => {
                    // flip_orientation returns Result, unwrap is safe for same reason
                    DecisionTarget::GridCell(cell.flip_orientation(grid_dimensions).unwrap())
                }
                DecisionTarget::Point(point) => {
                    DecisionTarget::Point(flip_point_orientation(point, field_width, field_height))
                }
            };
            Decision::Run(flipped_target)
        }
        Decision::Stop => Decision::Stop,
        Decision::Kick(target_point) => Decision::Kick(flip_point_orientation(
            &target_point,
            field_width,
            field_height,
        )),
    }
}

#[derive(Debug, Clone)]
pub struct PlayerDef {
    pub team: Team,
    pub number: u32,
    pub name: String,
    pub reaction_rate: u32, // 10-100: player's reaction speed
    pub speed_rate: u32,    // 10-100: player's movement speed
    pub tackle_rate: u32,   // 10-100: player's ball control ability
    pub shot_power: u32,    // 10-100: player's shot power
    pub shot_accuracy: u32, // 10-100: player's shot accuracy
    pub script: String,     // Lua script for decision making
    pub regions: HashMap<String, Region>,
}

impl PlayerDef {
    pub fn new(
        team: Team,
        number: u32,
        name: String,
        script: String,
        start_position: Region,
    ) -> Self {
        let mut regions = HashMap::new();
        regions.insert("start position".to_string(), start_position.clone());
        regions.insert("attack position".to_string(), start_position.clone());
        regions.insert("defence position".to_string(), start_position);

        Self {
            team,
            number,
            name,
            reaction_rate: 50, // Default values
            speed_rate: 50,
            tackle_rate: 50,
            shot_power: 50,
            shot_accuracy: 50,
            script,
            regions,
        }
    }

    /// Set custom reaction rate (default: 50)
    pub fn with_reaction_rate(mut self, rate: u32) -> Self {
        self.reaction_rate = rate;
        self
    }

    /// Set custom speed rate (default: 50)
    pub fn with_speed_rate(mut self, rate: u32) -> Self {
        self.speed_rate = rate;
        self
    }

    /// Set custom tackle rate (default: 50)
    pub fn with_tackle_rate(mut self, rate: u32) -> Self {
        self.tackle_rate = rate;
        self
    }

    /// Set custom shot power (default: 50)
    pub fn with_shot_power(mut self, power: u32) -> Self {
        self.shot_power = power;
        self
    }

    /// Set custom shot accuracy (default: 50)
    pub fn with_shot_accuracy(mut self, accuracy: u32) -> Self {
        self.shot_accuracy = accuracy;
        self
    }

    /// Set attack position (different from start position)
    pub fn with_attack_position(mut self, attack_position: Region) -> Self {
        self.regions.insert("attack position".to_string(), attack_position);
        self
    }

    /// Set defence position (different from start position)
    pub fn with_defence_position(mut self, defence_position: Region) -> Self {
        self.regions.insert("defence position".to_string(), defence_position);
        self
    }
}

#[derive(Debug, Clone)]
pub struct BallDef {
    pub initial_position: Point3D,
}

impl Default for BallDef {
    fn default() -> Self {
        Self {
            initial_position: Point3D::default(),
        }
    }
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
    pub is_ready: bool, // True when player is in start position (for Setup stage)
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
        GameStage::Setup("start".to_string())
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
                        // In Setup stage, players start behind the field (z = -5)
                        let field_width = config.field.width().get::<meter>();
                        Point3D::from_meters(
                            field_width / 2.0, // Center of field width (X axis)
                            0.0,               // Ground level
                            -5.0,              // 5 meters behind the field (Z axis)
                        )
                    }
                    GameStage::Play | GameStage::GameOver => {
                        // In Play/GameOver stage, players start at their start_position
                        let start_region = config.players[idx]
                            .regions
                            .get("start position")
                            .expect("Player must have 'start position' region");
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

        Self {
            state: GameState {
                elapsed_time: 0.0,
                stage,
                player_states,
                ball_state: BallState {
                    position: config.ball.initial_position.clone(),
                    velocity: Velocity3D::default(),
                    possessed_by: None,
                    last_possession_change_time: -1.0, // Set to negative so first change is allowed
                    last_possessing_team: None, // Neutral at game start
                },
                referee_states,
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

