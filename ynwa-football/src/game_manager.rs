//! Manages game stage transitions and football-specific stage logic.
//!
//! GameStage enum:
//! - `Play` - normal gameplay, scripts call `make_decision()`
//! - `Setup(String)` - preparation phase, scripts call `get_setup_position(reason)`, default: `Setup("start")`
//!
//! Setup stage behavior:
//! - Players start at (width/2, 0, -5) — 5m behind field edge
//! - Players marked ready when their `current_decision` is `Stop` (arrival detected by DecisionSystem)
//! - Automatically transitions to Play when all players are ready
//! - Ball placed at `GameState::restart_position` if set, otherwise at `ball.initial_position`
//!
//! Setup reasons and restart rules:
//! - `"start"` / `"after_goal"` — `restart_position = None` (ball at center)
//! - `"throw_in"` — ball at crossing point; `restart_team` = opponent of last touch
//! - `"corner"` — ball at nearest corner; `restart_team` = attacking team (last touch)
//! - `"goal_kick"` — ball at goal area (5.5m from goal line); `restart_team` = defending team (last touch)
//!
//! Design decisions:
//! - `Game::new()` uses `GameStage::default()` = `Setup("start")`
//! - Tests use `Game::with_stage()` to set stage explicitly
//! - `restart_position` and `restart_team` are set in `handle_event` (before Setup ticks begin)
//!   so they survive the `last_possessing_team = None` reset that happens each Setup tick

use ynwa_core::field::zones::{Point3D, Velocity3D};
use ynwa_core::game::{Game, GameStage};
use ynwa_core::system::System;
use ynwa_core::team::Team;
use uom::si::length::meter;

use crate::events::{check_events, FootballEvent};

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
                game.state.ball_state.position = game
                    .state
                    .restart_position
                    .unwrap_or(game.config().ball.initial_position);
                game.state.ball_state.velocity = Velocity3D::default();
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
        for player_state in game.state.player_states.iter_mut() {
            if player_state.is_ready {
                continue;
            }
            if matches!(player_state.current_decision, Some(ynwa_core::game::Decision::Stop)) {
                player_state.is_ready = true;
            }
        }
    }

    fn handle_event(&self, game: &mut Game, event: FootballEvent) {
        match event {
            FootballEvent::GameEnd => {
                game.state.stage = GameStage::GameOver;
            }
            FootballEvent::Goal(team) => {
                // `team` is the owner of the goal that was scored into — the scorer is the opponent
                game.state.team_stats
                    .entry(team.opposite())
                    .or_default()
                    .increment("score", 1.0);
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.restart_position = None;
                game.state.restart_team = Some(team); // team that conceded restarts from center
                game.state.stage = GameStage::Setup("after_goal".to_string());
            }
            FootballEvent::Touchline(position, last_team) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.restart_position = Some(position);
                game.state.restart_team = Some(last_team.opposite());
                game.state.stage = GameStage::Setup("throw_in".to_string());
            }
            FootballEvent::GoalLine(position, last_team) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                let field_length = game.config().field.length().get::<meter>();
                let field_width = game.config().field.width().get::<meter>();
                let ball_z = position.z.get::<meter>();

                // Team B attacks toward z=0 (Team A's goal); Team A attacks toward z=field_length.
                let attacking_team = if ball_z < field_length / 2.0 { Team::B } else { Team::A };

                if last_team == attacking_team {
                    let goal_kick_z = if ball_z < field_length / 2.0 {
                        GOAL_KICK_OFFSET
                    } else {
                        field_length - GOAL_KICK_OFFSET
                    };
                    game.state.restart_position = Some(Point3D::from_meters(
                        field_width / 2.0,
                        0.0,
                        goal_kick_z,
                    ));
                    game.state.restart_team = Some(last_team.opposite()); // defending team takes goal kick
                    game.state.stage = GameStage::Setup("goal_kick".to_string());
                } else {
                    game.state.restart_position = Some(nearest_corner(position, field_width, field_length));
                    game.state.restart_team = Some(attacking_team); // attacking team takes corner
                    game.state.stage = GameStage::Setup("corner".to_string());
                }
            }
        }
    }
}

// Standard goal kick distance from the goal line (FIFA: 5.5m = 6 yards)
const GOAL_KICK_OFFSET: f32 = 5.5;

fn nearest_corner(pos: Point3D, field_width: f32, field_length: f32) -> Point3D {
    let corners = [
        (0.0_f32, 0.0_f32),
        (field_width, 0.0),
        (0.0, field_length),
        (field_width, field_length),
    ];
    let (cx, cz) = corners
        .iter()
        .copied()
        .min_by(|&(ax, az), &(bx, bz)| {
            let da = (ax - pos.x.get::<meter>()).powi(2) + (az - pos.z.get::<meter>()).powi(2);
            let db = (bx - pos.x.get::<meter>()).powi(2) + (bz - pos.z.get::<meter>()).powi(2);
            da.partial_cmp(&db).unwrap()
        })
        .unwrap();
    Point3D::from_meters(cx, 0.0, cz)
}

impl Default for FootballGameManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "tests/game_manager_tests.rs"]
mod tests;
