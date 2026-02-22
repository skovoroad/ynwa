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
            let reaction_rate = if is_setup {
                // Use maximum frequency during setup stage
                100
            } else {
                game.config().players[i].reaction_rate
            };

            let mut interval = Self::reaction_interval(reaction_rate);
            if is_setup {
                // Player must't leave cell before check
                // TODO: revise this hack
                interval = interval / 3.; 
            }
            let player_state = &mut game.state.player_states[i];

            if timestamp - player_state.last_decision_time >= interval {
                player_state.needs_decision = true;
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

