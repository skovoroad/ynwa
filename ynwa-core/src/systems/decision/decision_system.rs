use crate::game::{Decision, DecisionTarget, Game, GameStage};
use crate::physics_util::distance_2d;
use crate::region::GridCell;
use crate::system::System;
use rand::Rng;
use std::fmt;
use uom::si::length::meter;

use super::convert_decision_to_display_orientation;
use super::util::resolve_target_point;

/// Distance threshold (metres) at which a running player is considered to have reached their target.
/// Applied every tick in both Setup and Play stages to stop the player immediately,
/// without waiting for the next script invocation (which may be seconds away).
const ARRIVAL_THRESHOLD_METERS: f32 = 0.5;

// Design: DecisionSystem delegates decision-making to DecisionMaker implementations.
// This separates coordination (when to decide) from strategy (what to decide).

/// Errors that can occur during decision-making
#[derive(Debug, Clone)]
pub enum DecisionError {
    ScriptError(String),
    Timeout(String),
    RuntimeError(String),
}

impl fmt::Display for DecisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecisionError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            DecisionError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            DecisionError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
        }
    }
}

impl std::error::Error for DecisionError {}

pub trait DecisionMaker {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError>;
}

/// Temporary stub - generates random run decisions until real AI is implemented
pub struct PlaceholderDecisionMaker;

impl PlaceholderDecisionMaker {
    pub fn new() -> Self {
        Self
    }
}

impl DecisionMaker for PlaceholderDecisionMaker {
    fn make_decision(
        &mut self,
        game: &Game,
        _player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        let grid_dims = game.config().field.grid_dimensions();
        let mut rng = rand::rng();

        let col = rng.random_range(1..=grid_dims.columns);
        let row = rng.random_range(1..=grid_dims.rows);
        let cell =
            GridCell::new(col, row).map_err(|e| DecisionError::RuntimeError(e.to_string()))?;

        Ok((Decision::Run(DecisionTarget::GridCell(cell)), None))
    }
}

impl Default for PlaceholderDecisionMaker {
    fn default() -> Self {
        Self::new()
    }
}

pub struct DecisionSystem {
    decision_maker: Box<dyn DecisionMaker>,
    on_error: fn(&DecisionError, usize) -> Option<Decision>,
}

impl DecisionSystem {
    pub fn new() -> Self {
        Self {
            decision_maker: Box::new(PlaceholderDecisionMaker),
            on_error: Self::default_error_handler,
        }
    }

    pub fn with_decision_maker(mut self, decision_maker: Box<dyn DecisionMaker>) -> Self {
        self.decision_maker = decision_maker;
        self
    }

    // for now just for tests
    pub fn with_error_handler(
        mut self,
        handler: fn(&DecisionError, usize) -> Option<Decision>,
    ) -> Self {
        self.on_error = handler;
        self
    }

    fn default_error_handler(error: &DecisionError, player_index: usize) -> Option<Decision> {
        eprintln!("Player {} decision error: {}", player_index, error);
        None // No decision on error by default
    }
}

impl Default for DecisionSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl System for DecisionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let player_count = game.state.player_states.len();

        for player_index in 0..player_count {
            let field_width = game.config().field.width().get::<meter>();
            let field_length = game.config().field.length().get::<meter>();
            let grid_dims = game.config().field.grid_dimensions();
            let player_team = game.config().players[player_index].team;
            let player_pos = game.state.player_states[player_index].position;
            let current_decision = game.state.player_states[player_index].current_decision.clone();

            // Arrival check: runs every tick regardless of stage or reaction timer.
            // When the player reaches the target of their Run decision, immediately
            // override with Stop — without calling the script. This prevents overshooting
            // caused by the gap between reaction-rate ticks (up to 3 s at low reaction_rate).
            if let Some(decision) = &current_decision {
                if let Some(target) = resolve_target_point(decision, field_width, grid_dims, &game.state.ball_state) {
                    if distance_2d(&player_pos, &target) < ARRIVAL_THRESHOLD_METERS {
                        let stop = convert_decision_to_display_orientation(
                            &Decision::Stop,
                            player_team,
                            field_width,
                            field_length,
                            grid_dims,
                        );
                        let player_state = &mut game.state.player_states[player_index];
                        player_state.current_decision = Some(stop);
                        player_state.decision_processed = false;
                        // In Setup: suppress the next script call so the player stays put
                        // until the manager transitions to Play.
                        // In Play: allow the reaction timer to fire normally — the script
                        // will pick a new target when it next runs.
                        let is_setup = matches!(game.state.stage, GameStage::Setup(_));
                        if is_setup {
                            player_state.needs_decision = false;
                        }
                        continue;
                    }
                }
            }

            if game.state.player_states[player_index].needs_decision {
                let decision_result = self.decision_maker.make_decision(game, player_index);

                let player_team = game.config().players[player_index].team;
                let field_width = game.config().field.width().get::<meter>();
                let field_length = game.config().field.length().get::<meter>();
                let grid_dims = game.config().field.grid_dimensions();

                let player_state = &mut game.state.player_states[player_index];

                match decision_result {
                    Ok((decision, reason)) => {
                        let display_decision = convert_decision_to_display_orientation(
                            &decision,
                            player_team,
                            field_width,
                            field_length,
                            grid_dims,
                        );

                        player_state.current_decision = Some(display_decision);
                        player_state.decision_reason = reason;
                        player_state.decision_processed = false;
                        player_state.needs_decision = false;
                        player_state.last_decision_time = timestamp;
                        player_state.last_error = None;
                    }
                    Err(error) => {
                        let error_message = error.to_string();
                        let error_decision = (self.on_error)(&error, player_index);

                        let converted_error_decision = error_decision.map(|d| {
                            convert_decision_to_display_orientation(
                                &d,
                                player_team,
                                field_width,
                                field_length,
                                grid_dims,
                            )
                        });

                        // Always treat error as "completed attempt" to prevent storm
                        // This ensures rate-limiting via PlayerReactionSystem's reaction_rate
                        player_state.current_decision = converted_error_decision;
                        player_state.decision_reason = None;
                        player_state.decision_processed = false;
                        player_state.needs_decision = false;
                        player_state.last_decision_time = timestamp;
                        player_state.last_error = Some(error_message);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "../../tests/decision_system_tests.rs"]
mod tests;
