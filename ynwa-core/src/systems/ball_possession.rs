//! Determines which player possesses the ball based on proximity and player characteristics.
//!
//! Key parameters:
//! - `POSSESSION_RADIUS = 1.0m` - distance to contest the ball
//! - `POSSESSION_COOLDOWN = 1.0s` - minimum interval between possession changes (prevents bounce)
//! - `tackle_rate`: 10-100 - ability to win the ball
//!
//! Possession logic:
//! - Skipped entirely during Setup stage (ball is fixed, possession is meaningless)
//! - Only opponents can contest ball from current possessor (teammates never steal)
//! - Free ball: all players within radius can claim it
//! - Probabilistic selection: Score = `tackle_rate × random_multiplier`, multiplier ∈ [0.5, 1.5]
//!   at temperature = 1.0; the spread shrinks proportionally at lower temperatures
//! - Possession change triggers `needs_decision = true` for all players
//!
//! Ball state fields:
//! - `possessed_by: Option<usize>` - current owner index or None
//! - `last_possessing_team: Option<Team>` - persists during passes to track ownership,
//!   reset to None on Setup stage transition, available in Lua as `context.ball.owner_team`
//!
//! Design: randomness is delegated to the central `RngManager` via `Game`.
//! At temperature=0.0 all scores are proportional to `tackle_rate`; ties are broken
//! in favour of the lower index (stable sort).

use crate::game::Game;
use crate::physics_util::distance_length;
use crate::system::System;
use uom::si::f32::Length;
use uom::si::length::meter;

const POSSESSION_RADIUS: f32 = 1.0; // meters
const POSSESSION_COOLDOWN: f32 = 1.0; // seconds - minimum time between possession changes

/// Ball possession system
pub struct BallPossessionSystem;

impl BallPossessionSystem {
    pub fn new() -> Self {
        Self
    }

    /// Select winner based on tackle_rate with probabilistic selection.
    fn select_winner(&self, game: &Game, candidates: &[usize]) -> usize {
        if candidates.len() == 1 {
            return candidates[0];
        }

        let mut scores: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&idx| {
                let tackle_rate = game.config().players[idx].tackle_rate as f32;
                let score = game.rng_manager().randomize(tackle_rate, 0.5);
                (idx, score)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores[0].0
    }
}

/// Find players within possession radius of the ball.
/// If ball is possessed, only returns opponents of the current owner.
fn find_nearby_players(game: &Game) -> Vec<usize> {
    let ball_pos = &game.state.ball_state.position;
    let radius = Length::new::<meter>(POSSESSION_RADIUS);

    // Determine if we should filter by team
    let owner_team = game
        .state
        .ball_state
        .possessed_by
        .map(|owner_idx| game.config().players[owner_idx].team);

    game.state
        .player_states
        .iter()
        .enumerate()
        .filter_map(|(idx, player_state)| {
            let distance = distance_length(&player_state.position, ball_pos);
            if distance <= radius {
                // If ball has owner, only include opponents
                if let Some(owner_team) = owner_team {
                    let player_team = game.config().players[idx].team;
                    if player_team == owner_team {
                        return None; // Skip teammates
                    }
                }
                Some(idx)
            } else {
                None
            }
        })
        .collect()
}

impl System for BallPossessionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        if matches!(game.state.stage, crate::game::GameStage::Setup(_)) {
            return;
        }

        // Check if we're in cooldown period after last possession change
        let time_since_change = timestamp - game.state.ball_state.last_possession_change_time;
        if time_since_change < POSSESSION_COOLDOWN {
            return; // Skip possession determination during cooldown
        }

        // Find players near the ball (opponents only if ball is possessed)
        let nearby_players = find_nearby_players(game);

        // Determine new possession
        let new_possession = match nearby_players.len() {
            0 => {
                // No opponents nearby - keep current possession
                return;
            }
            1 => Some(nearby_players[0]), // Single opponent gets possession
            _ => {
                // Multiple opponents - probabilistic selection
                let winner = self.select_winner(game, &nearby_players);
                Some(winner)
            }
        };

        // Update possession if it changed
        if new_possession != game.state.ball_state.possessed_by {
            game.state.ball_state.possessed_by = new_possession;
            game.state.ball_state.last_possession_change_time = timestamp;

            // Update last possessing team
            if let Some(player_idx) = new_possession {
                let team = game.config().players[player_idx].team;
                game.state.ball_state.last_possessing_team = Some(team);
            }
            // Note: If new_possession is None (ball is free), last_possessing_team keeps previous value

            // Set needs_decision flag for all players when possession changes
            for player_state in &mut game.state.player_states {
                player_state.needs_decision = true;
            }
        }
    }
}

impl Default for BallPossessionSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[path = "../tests/ball_possession_tests.rs"]
mod tests;
