//! Manages game stage transitions and football-specific stage logic.
//!
//! GameStage enum:
//! - `Play` - normal gameplay, scripts call `make_decision()`
//! - `Setup(String)` - preparation phase; player decisions assigned by engine, default: `Setup("start")`
//!
//! Setup stage behavior:
//! - Players start at (width/2, 0, -5) — 5m behind field edge
//! - Players marked ready when their `current_decision` is `Stop` (arrival detected by DecisionSystem)
//! - Automatically transitions to Play when all players are ready
//! - Ball placed at `GameState::restart_position` if set, otherwise at `ball.initial_position`
//!
//! Setup reasons and restart rules:
//! - `"kick off"` — `restart_position = None` (ball at center); used for game start and after goal
//! - `"throw in"` — ball at crossing point; `restart_team` = opponent of last touch
//! - `"corner"` — ball at nearest corner; `restart_team` = attacking team (last touch)
//! - `"goal kick"` — ball at goal area (5.5m from goal line); `restart_team` = defending team (last touch)
//!
//! Setup decision assignment (`assign_setup_decisions`):
//! - Called before `check_player_readiness` on every Setup tick
//! - For each player with `needs_decision == true`, resolves the SET_PIECE_KEY for their team/position
//!   using `resolve_set_piece_key`, then assigns `Decision::Run` to the restart point (if taker) or
//!   to the player's region for that key. Missing region → `last_error` set, player stays put.
//! - For "kick off", taker runs to ball initial_position (center), not to restart_position (None).
//!
//! Design decisions:
//! - `Game::new()` uses `GameStage::default()` = `Setup("kick off")`
//! - Tests use `Game::with_stage()` to set stage explicitly
//! - `restart_position` and `restart_team` are set in `handle_event` (before Setup ticks begin)
//!   so they survive the `last_possessing_team = None` reset that happens each Setup tick

use ynwa_core::field::zones::{Point3D, Velocity3D};
use ynwa_core::game::{Decision, DecisionTarget, Game, GameStage};
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

                self.assign_setup_decisions(game);
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

    fn assign_setup_decisions(&self, game: &mut Game) {
        let reason = match &game.state.stage {
            GameStage::Setup(r) => r.clone(),
            _ => return,
        };

        let restart_position = game.state.restart_position;
        let restart_team = game.state.restart_team;
        let ball_initial = game.config().ball.initial_position;
        let field_width = game.config().field.width().get::<meter>();
        let field_length = game.config().field.length().get::<meter>();
        let player_count = game.config().players.len();

        for i in 0..player_count {
            if !game.state.player_states[i].needs_decision {
                continue;
            }

            let player_team = game.config().players[i].team;
            let key = match resolve_set_piece_key(
                &reason,
                player_team,
                restart_team,
                restart_position,
                field_width,
                field_length,
            ) {
                Some(k) => k,
                None => {
                    game.state.player_states[i].last_error =
                        Some(format!("unknown setup reason '{}'", reason));
                    continue;
                }
            };

            let is_taker = game.config().players[i].set_piece_roles.contains(key);

            let decision = if is_taker {
                // Taker runs to the ball: restart_position if set, otherwise center (kick off).
                let target = restart_position.unwrap_or(ball_initial);
                Decision::Run(DecisionTarget::Point(target))
            } else {
                match game.config().players[i].regions.get(key) {
                    Some(region) => Decision::Run(DecisionTarget::Region(region.clone())),
                    None => {
                        game.state.player_states[i].last_error =
                            Some(format!("missing region for set-piece key '{}'", key));
                        continue;
                    }
                }
            };

            game.state.player_states[i].current_decision = Some(decision);
            game.state.player_states[i].needs_decision = false;
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
                game.state.stage = GameStage::Setup("kick off".to_string());
            }
            FootballEvent::Touchline(position, last_team) => {
                for player_state in game.state.player_states.iter_mut() {
                    player_state.is_ready = false;
                    player_state.current_decision = None;
                    player_state.needs_decision = true;
                }
                game.state.restart_position = Some(position);
                game.state.restart_team = Some(last_team.opposite());
                game.state.stage = GameStage::Setup("throw in".to_string());
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
                    game.state.stage = GameStage::Setup("goal kick".to_string());
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

/// Maps a setup reason + runtime state to the SET_PIECE_KEY for a specific player.
///
/// Returns `None` for unknown reasons — caller should treat this as an error.
/// "own/opp": whether restart_team == player_team.
/// "left/right": from the player's perspective (Team B's left is Team A's right).
/// "own half/opp half": whether ball is in the player's own half.
///
/// Team A attacks toward z=field_length; Team A's left is low x, right is high x.
/// Team B attacks toward z=0; Team B's left is high x, right is low x.
pub(crate) fn resolve_set_piece_key(
    reason: &str,
    player_team: Team,
    restart_team: Option<Team>,
    restart_position: Option<Point3D>,
    field_width: f32,
    field_length: f32,
) -> Option<&'static str> {
    let is_own = restart_team.map(|t| t == player_team).unwrap_or(false);

    match reason {
        "kick off" => Some(if is_own { "kick off own" } else { "kick off opp" }),
        "goal kick" => Some(if is_own { "goal kick own" } else { "goal kick opp" }),
        "corner" => {
            let pos = restart_position.unwrap_or_default();
            let ball_x = pos.x.get::<meter>();
            let is_left = if player_team == Team::A {
                ball_x < field_width / 2.0
            } else {
                ball_x >= field_width / 2.0
            };
            Some(match (is_own, is_left) {
                (true,  true)  => "corner own left",
                (true,  false) => "corner own right",
                (false, true)  => "corner opp left",
                (false, false) => "corner opp right",
            })
        }
        "throw in" => {
            let pos = restart_position.unwrap_or_default();
            let ball_x = pos.x.get::<meter>();
            let ball_z = pos.z.get::<meter>();
            let is_left = if player_team == Team::A {
                ball_x < field_width / 2.0
            } else {
                ball_x >= field_width / 2.0
            };
            // "own half": ball is in the player's own half (defending half)
            // Team A defends z < field_length/2; Team B defends z > field_length/2
            let is_own_half = if player_team == Team::A {
                ball_z < field_length / 2.0
            } else {
                ball_z >= field_length / 2.0
            };
            Some(match (is_own, is_left, is_own_half) {
                (true,  true,  true)  => "throw in own left own half",
                (true,  true,  false) => "throw in own left opp half",
                (true,  false, true)  => "throw in own right own half",
                (true,  false, false) => "throw in own right opp half",
                (false, true,  true)  => "throw in opp left own half",
                (false, true,  false) => "throw in opp left opp half",
                (false, false, true)  => "throw in opp right own half",
                (false, false, false) => "throw in opp right opp half",
            })
        }
        _ => None,
    }
}

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
