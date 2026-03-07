use crate::game::Game;
use crate::system::System;

/// Player reaction system - determines when players need to make decisions
pub struct PlayerReactionSystem;

impl PlayerReactionSystem {
    pub fn new() -> Self {
        Self
    }

    // TODO: cache the value
    fn reaction_interval(reaction_rate: u32) -> f32 {
        // reaction_rate 100 -> 0.5s, reaction_rate 10 -> 3.0s (linear)
        // Formula: interval = 0.5 + (100 - rate) * (3.0 - 0.5) / (100 - 10)
        let rate = reaction_rate.clamp(10, 100) as f32;
        0.5 + (100.0 - rate) * 2.5 / 90.0
    }
}

impl System for PlayerReactionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        let player_count = game.config().players.len();
        let is_setup = matches!(game.state().stage, crate::game::GameStage::Setup(_));

        for i in 0..player_count {
            if is_setup {
                // In Setup: request a decision only when the player has none yet.
                // Suppresses needs_decision for players already moving or arrived
                // (DecisionSystem is the final guard for the arrived-Stop case).
                if game.state.player_states[i].current_decision.is_none() {
                    game.state.player_states[i].needs_decision = true;
                } else {
                    game.state.player_states[i].needs_decision = false;
                }
            } else {
                let reaction_rate = game.config().players[i].reaction_rate;
                let interval = Self::reaction_interval(reaction_rate);
                let player_state = &mut game.state.player_states[i];
                if timestamp - player_state.last_decision_time >= interval {
                    player_state.needs_decision = true;
                }
            }
        }
    }
}

impl Default for PlayerReactionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/player_reaction_tests.rs"]
mod tests;
