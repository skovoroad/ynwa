//! Manages game stage transitions and football-specific stage logic.
//!
//! GameStage enum:
//! - `Play` - normal gameplay, scripts call `make_decision()`
//! - `Setup(String)` - preparation phase, scripts call `get_setup_position(reason)`, default: `Setup("start")`
//!
//! Setup stage behavior:
//! - Players start at (width/2, 0, -5) — 5m behind field edge
//! - Players marked ready when inside their `start_position` region
//! - Automatically transitions to Play when all players are ready
//!
//! Design decisions:
//! - `Game::new()` uses `GameStage::default()` = `Setup("start")`
//! - Tests use `Game::with_stage()` to set stage explicitly
//! - Stage transitions are one-way (no Play → Setup) — temporary until event system is complete

use crate::game::{Game, GameStage};
use crate::region::Region;
use crate::system::System;
use uom::si::length::meter;

use super::events::{check_events, FootballEvent};

/// Football game manager - manages football-specific game logic
pub struct FootballGameManager;

impl FootballGameManager {
    pub fn new() -> Self {
        Self
    }
}

impl System for FootballGameManager {
    fn update(&mut self, game: &mut Game, _timestamp: f32) {
        match &game.state.stage {
            GameStage::Setup(_stage_name) => {
                game.state.ball_state.position = game.config().ball.initial_position;
                game.state.ball_state.velocity = crate::field::zones::Velocity3D::default();
                game.state.ball_state.possessed_by = None;
                game.state.ball_state.last_possessing_team = None;

                self.check_player_readiness(game);

                if game.state.player_states.iter().all(|p| p.is_ready) {
                    game.state.stage = GameStage::Play;
                }
            }
            GameStage::Play => {
                if let Some(event) = check_events(game) {
                    self.handle_event(game, event);
                }
            }
            GameStage::GameOver => {
                // Game is over, do nothing
            }
        }
    }
}

impl FootballGameManager {
    fn check_player_readiness(&self, game: &mut Game) {
        let field_width = game.config().field.width().get::<meter>();
        let grid_dims = game.config().field.grid_dimensions();

        // Collect player start regions first to avoid borrowing issues
        let start_regions: Vec<_> = game
            .config()
            .players
            .iter()
            .map(|player_def| {
                player_def
                    .regions
                    .get("start position")
                    .expect("Player must have 'start position' region")
                    .clone()
            })
            .collect();

        for (idx, player_state) in game.state.player_states.iter_mut().enumerate() {
            if player_state.is_ready {
                continue; // Already ready
            }

            let start_region = &start_regions[idx];

            // Check if player is inside their start region
            if is_player_in_start_region(
                &player_state.position,
                start_region,
                grid_dims,
                field_width,
            ) {
                player_state.is_ready = true;
            }
        }
    }

    fn handle_event(&self, game: &mut Game, event: FootballEvent) {
        match event {
            FootballEvent::GameEnd => {
                game.state.stage = GameStage::GameOver;
            }
            FootballEvent::Goal(_team) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.stage = GameStage::Setup("after_goal".to_string());
            }
            FootballEvent::Touchline(_position) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.stage = GameStage::Setup("throw_in".to_string());
            }
            FootballEvent::GoalLine(_position) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.stage = GameStage::Setup("set_piece".to_string());
            }
        }
    }
}

fn is_player_in_start_region(
    position: &crate::field::zones::Point3D,
    start_region: &Region,
    grid_dims: crate::region::GridDimensions,
    field_width: f32,
) -> bool {
    start_region.contains_point(grid_dims, field_width, position)
}

impl Default for FootballGameManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/game_manager_tests.rs"]
mod tests;
