//! Replay of a recorded playthrough by re-simulating physics only.
//!
//! The journal carries everything that cannot be recomputed (decisions, random-branch
//! outcomes, stage changes). A replay world runs just this driver and the physics system;
//! the driver feeds journaled events back in at the tick they were recorded on.

use crate::field::zones::Velocity3D;
use crate::game::{Decision, Game};
use crate::journal::{JournalEntry, JournalEvent};
use crate::system::System;
use crate::systems::movement::{calculate_target_point, calculate_velocity};

/// Applies journaled events as the replayed clock advances.
///
/// Events are applied in journal order, and each one is applied at the first tick whose
/// timestamp reaches its own — mirroring the tick it was recorded on.
pub struct ReplayDriver {
    journal: Vec<JournalEntry>,
    next_index: usize,
}

impl ReplayDriver {
    pub fn new(journal: Vec<JournalEntry>) -> Self {
        Self {
            journal,
            next_index: 0,
        }
    }

    fn apply_due_events(&mut self, game: &mut Game, timestamp: f32) {
        while self.next_index < self.journal.len()
            && self.journal[self.next_index].timestamp <= timestamp
        {
            let event = self.journal[self.next_index].event.clone();
            self.next_index += 1;
            Self::apply_event(game, timestamp, &event);
        }
    }

    fn apply_event(game: &mut Game, timestamp: f32, event: &JournalEvent) {
        match event {
            JournalEvent::DecisionAssigned {
                player_index,
                decision,
                reason,
            } => Self::apply_decision(game, timestamp, *player_index, decision, reason.clone()),
            JournalEvent::PossessionChange {
                possessed_by,
                last_possessing_team,
            } => {
                game.state.ball_state.possessed_by = *possessed_by;
                game.state.ball_state.last_possessing_team = *last_possessing_team;
                game.state.ball_state.last_possession_change_time = timestamp;
                for player_state in &mut game.state.player_states {
                    player_state.needs_decision = true;
                }
            }
            JournalEvent::KickOutcome { ball_velocity, .. } => {
                game.state.ball_state.velocity = *ball_velocity;
                game.state.ball_state.possessed_by = None;
                game.state.ball_state.last_possession_change_time = timestamp;
            }
            JournalEvent::StageChange { stage } => {
                game.state.stage = stage.clone();
            }
            JournalEvent::RestartSet {
                restart_position,
                restart_team,
            } => {
                game.state.restart_position = *restart_position;
                game.state.restart_team = *restart_team;
            }
            JournalEvent::DecisionsReset => {
                for player_state in &mut game.state.player_states {
                    player_state.current_decision = None;
                    player_state.is_ready = false;
                    player_state.needs_decision = true;
                }
            }
            JournalEvent::StatUpdate { team, key, delta } => {
                game.state
                    .team_stats
                    .entry(*team)
                    .or_default()
                    .increment(key, *delta);
            }
            JournalEvent::External { .. } => {}
        }
    }

    /// Reproduces the physical effect `ActionSystem` applies to a recorded decision, reusing its
    /// movement helpers so the replay computes exactly the same velocities.
    fn apply_decision(
        game: &mut Game,
        timestamp: f32,
        player_index: usize,
        decision: &Decision,
        reason: Option<String>,
    ) {
        {
            let player_state = &mut game.state.player_states[player_index];
            player_state.current_decision = Some(decision.clone());
            player_state.decision_reason = reason;
            player_state.decision_processed = false;
            player_state.needs_decision = false;
            player_state.last_decision_time = timestamp;
            player_state.last_error = None;
        }

        match decision {
            Decision::Stop => {
                game.state.player_states[player_index].velocity = Velocity3D::default();
            }
            Decision::Run(target) => {
                let player_position = game.state.player_states[player_index].position;
                let speed_rate = game.config().players[player_index].speed_rate;
                let target_point = calculate_target_point(target, game);
                game.state.player_states[player_index].velocity =
                    calculate_velocity(&player_position, &target_point, speed_rate);
            }
            // A kick only releases the player; the ball velocity comes from `KickOutcome`.
            Decision::Kick(_) => {}
        }

        game.state.player_states[player_index].decision_processed = true;
    }
}

impl System for ReplayDriver {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        self.apply_due_events(game, timestamp);
    }
}

#[cfg(test)]
#[path = "tests/replay_tests.rs"]
mod tests;
