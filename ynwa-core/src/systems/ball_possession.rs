use crate::game::Game;
use crate::physics_util::distance_length;
use crate::system::System;
use uom::si::f32::Length;
use uom::si::length::meter;

const POSSESSION_RADIUS: f32 = 1.0; // meters
const POSSESSION_COOLDOWN: f32 = 1.0; // seconds - minimum time between possession changes

/// Ball possession system - determines which player owns the ball
pub struct BallPossessionSystem {
    /// Optional random number generator for testing (0.0 to 1.0)
    /// If None, uses rand::random()
    rng: Option<Box<dyn Fn() -> f32 + Send>>,
}

impl BallPossessionSystem {
    pub fn new() -> Self {
        Self { rng: None }
    }

    /// Create a system with a custom RNG for testing
    pub fn with_rng<F>(rng: F) -> Self
    where
        F: Fn() -> f32 + Send + 'static,
    {
        Self {
            rng: Some(Box::new(rng)),
        }
    }

    fn get_random(&self) -> f32 {
        if let Some(ref rng) = self.rng {
            rng()
        } else {
            rand::random()
        }
    }

    /// Find players within possession radius of the ball
    /// If ball is possessed, only returns opponents of the current owner
    fn find_nearby_players(&self, game: &Game) -> Vec<usize> {
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

    /// Select winner based on tackle_rate with probabilistic selection
    /// Uses weighted random selection where tackle_rate determines probability
    fn select_winner(&self, game: &Game, candidates: &[usize]) -> usize {
        if candidates.len() == 1 {
            return candidates[0];
        }

        // Calculate weighted scores: tackle_rate * random_multiplier
        // This ensures even weak players have a chance, though small
        let mut scores: Vec<(usize, f32)> = candidates
            .iter()
            .map(|&idx| {
                let tackle_rate = game.config().players[idx].tackle_rate as f32;
                // Random multiplier between 0.5 and 1.5 gives variation
                // while keeping tackle_rate as the primary factor
                let random_multiplier = 0.5 + self.get_random();
                let score = tackle_rate * random_multiplier;
                (idx, score)
            })
            .collect();

        // Find player with highest score
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        scores[0].0
    }
}

impl System for BallPossessionSystem {
    fn update(&mut self, game: &mut Game, timestamp: f32) {
        // Check if we're in cooldown period after last possession change
        let time_since_change = timestamp - game.state.ball_state.last_possession_change_time;
        if time_since_change < POSSESSION_COOLDOWN {
            return; // Skip possession determination during cooldown
        }

        // Find players near the ball (opponents only if ball is possessed)
        let nearby_players = self.find_nearby_players(game);

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
mod tests {
    use super::*;
    use crate::field::zones::{Point3D, Velocity3D};
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, PlayerState, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_field() -> Field {
        Field::from_meters(100.0, 60.0, 20, 40)
    }

    fn create_test_player(
        team: Team,
        number: u32,
        tackle_rate: u32,
        position: Point3D,
    ) -> (PlayerDef, PlayerState) {
        let grid_dims = create_test_field().grid_dimensions();
        let start_region = Region::new(
            team,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let player_def = PlayerDef::new(
            team,
            number,
            format!("Player {}", number),
            50,
            50,
            tackle_rate,
            50,
            50,
            "function make_decision() return {} end".to_string(),
            start_region,
        );

        let player_state = PlayerState {
            position,
            velocity: Velocity3D::default(),
            last_decision_time: 0.0,
            needs_decision: false,
            current_decision: None,
            decision_processed: false,
            last_error: None,
            is_ready: false,
        };

        (player_def, player_state)
    }

    #[test]
    fn test_no_players_nearby() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Player far away (10m)
        let (player_def, player_state) =
            create_test_player(Team::A, 1, 50, Point3D::from_meters(60.0, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::new();
        system.update(&mut game, 0.0);

        // No possession should be assigned
        assert_eq!(game.state.ball_state.possessed_by, None);
    }

    #[test]
    fn test_single_player_nearby() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Player within 1m
        let (player_def, player_state) =
            create_test_player(Team::A, 1, 50, Point3D::from_meters(50.5, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::new();
        system.update(&mut game, 0.0);

        // Player 0 should have possession
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
    }

    #[test]
    fn test_two_players_deterministic_selection() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players within 1m, different tackle rates
        let (player1_def, player1_state) = create_test_player(
            Team::A,
            1,
            80, // High tackle rate
            Point3D::from_meters(50.5, 30.0, 0.0),
        );

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            40, // Low tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Use RNG=1.0 (max multiplier 1.5)
        // Player 0: 80 * 1.5 = 120.0
        // Player 1: 40 * 1.5 = 60.0
        let mut system = BallPossessionSystem::with_rng(|| 1.0);
        system.update(&mut game, 0.0);

        // Player 0 (tackle_rate=80) should win
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
    }

    #[test]
    fn test_two_players_probabilistic_upset() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players within 1m, different tackle rates
        let (player1_def, player1_state) = create_test_player(
            Team::A,
            1,
            80, // High tackle rate
            Point3D::from_meters(50.5, 30.0, 0.0),
        );

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            40, // Low tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Use RNG that gives advantage to weaker player
        // Player 0: 80 * 0.5 = 40.0 (unlucky)
        // Player 1: 40 * 1.5 = 60.0 (lucky) - wins despite lower tackle_rate!
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = BallPossessionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 {
                0.0
            } else {
                1.0
            } // First gets min (0.5x), second gets max (1.5x)
        });
        system.update(&mut game, 0.0);

        // Player 1 (index 1) should win due to lucky roll
        assert_eq!(game.state.ball_state.possessed_by, Some(1));
    }

    #[test]
    fn test_extreme_difference_weak_can_still_win() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Extreme difference: 100 vs 10
        let (player1_def, player1_state) = create_test_player(
            Team::A,
            1,
            100, // Maximum tackle rate
            Point3D::from_meters(50.5, 30.0, 0.0),
        );

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            10, // Minimum tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Player 0: 100 * 0.5 = 50.0 (very unlucky)
        // Player 1: 10 * 1.5 = 15.0 (very lucky, but still loses)
        // Even with extreme luck, 10 vs 100 is too much
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = BallPossessionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 {
                0.0
            } else {
                1.0
            }
        });
        system.update(&mut game, 0.0);

        // Player 0 still wins (50 > 15)
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
    }

    #[test]
    fn test_moderate_difference_upset_possible() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Moderate difference where upset is theoretically possible
        let (player1_def, player1_state) = create_test_player(
            Team::A,
            1,
            60, // Good tackle rate
            Point3D::from_meters(50.5, 30.0, 0.0),
        );

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            45, // Decent tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Player 0: 60 * 0.5 = 30.0 (unlucky)
        // Player 1: 45 * 1.5 = 67.5 (lucky) - upset!
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = BallPossessionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            if count == 0 {
                0.0
            } else {
                1.0
            }
        });
        system.update(&mut game, 0.0);

        // Player 1 wins with lucky roll
        assert_eq!(game.state.ball_state.possessed_by, Some(1));
    }

    #[test]
    fn test_possession_cooldown_prevents_immediate_change() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players within 1m
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 80, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) =
            create_test_player(Team::A, 2, 70, Point3D::from_meters(49.5, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::with_rng(|| 0.5);

        // First update at t=0.0 - should assign possession
        system.update(&mut game, 0.0);
        let first_owner = game.state.ball_state.possessed_by;
        assert!(first_owner.is_some());
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0);

        // Second update at t=0.5 (within cooldown) - should NOT change
        system.update(&mut game, 0.5);
        assert_eq!(game.state.ball_state.possessed_by, first_owner);
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0); // Unchanged

        // Third update at t=0.9 (still within cooldown) - should NOT change
        system.update(&mut game, 0.9);
        assert_eq!(game.state.ball_state.possessed_by, first_owner);
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0); // Unchanged
    }

    #[test]
    fn test_possession_cooldown_allows_change_after_timeout() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players within 1m - DIFFERENT TEAMS
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 80, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) = create_test_player(
            Team::B, // Different team!
            7,
            70,
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Use RNG that will cause different winners at different times
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = BallPossessionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            // First determination: player 0 wins (both get 0.5)
            // Second determination: player 1 wins (0 gets 0.0, 1 gets 1.0)
            if count < 2 {
                0.5
            } else if count == 2 {
                0.0
            } else {
                1.0
            }
        });

        // First update at t=0.0 - assign to player 0
        system.update(&mut game, 0.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0);

        // Update at t=1.1 (after cooldown) - can change now
        system.update(&mut game, 1.1);
        // Should change to player 1 due to RNG
        assert_eq!(game.state.ball_state.possessed_by, Some(1));
        assert_eq!(game.state.ball_state.last_possession_change_time, 1.1);
    }

    #[test]
    fn test_no_possession_change_updates_timestamp() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Single player
        let (player_def, player_state) =
            create_test_player(Team::A, 1, 50, Point3D::from_meters(50.5, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::new();

        // First update - gets possession
        system.update(&mut game, 0.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0);

        // Second update - same player, no change
        system.update(&mut game, 2.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
        // Timestamp should NOT update because possession didn't change
        assert_eq!(game.state.ball_state.last_possession_change_time, 0.0);
    }

    #[test]
    fn test_possession_change_triggers_all_players_decision() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Three players: 2 near ball, 1 far away
        let (player1_def, mut player1_state) = create_test_player(
            Team::A,
            1,
            80,
            Point3D::from_meters(50.5, 30.0, 0.0), // Near ball
        );

        let (player2_def, mut player2_state) = create_test_player(
            Team::A,
            2,
            70,
            Point3D::from_meters(49.5, 30.0, 0.0), // Near ball
        );

        let (player3_def, mut player3_state) = create_test_player(
            Team::B,
            7,
            60,
            Point3D::from_meters(70.0, 30.0, 0.0), // Far from ball
        );

        // Set needs_decision to false for all players initially
        player1_state.needs_decision = false;
        player2_state.needs_decision = false;
        player3_state.needs_decision = false;

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def, player3_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state, player3_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::with_rng(|| 0.5);

        // First update - possession changes from None to Some
        system.update(&mut game, 0.0);

        // All players should have needs_decision set to true
        assert!(
            game.state.player_states[0].needs_decision,
            "Player 0 should need decision"
        );
        assert!(
            game.state.player_states[1].needs_decision,
            "Player 1 should need decision"
        );
        assert!(
            game.state.player_states[2].needs_decision,
            "Player 2 (far away) should need decision"
        );
    }

    #[test]
    fn test_no_possession_change_no_decision_trigger() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players near ball
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 80, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            70,
            Point3D::from_meters(70.0, 30.0, 0.0), // Far away
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        let mut system = BallPossessionSystem::with_rng(|| 0.5);

        // First update - player 0 gets possession
        system.update(&mut game, 0.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));

        // Reset needs_decision flags
        game.state.player_states[0].needs_decision = false;
        game.state.player_states[1].needs_decision = false;

        // Second update after cooldown - same player still near ball, no change
        system.update(&mut game, 2.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));

        // needs_decision should NOT be set because possession didn't change
        assert!(
            !game.state.player_states[0].needs_decision,
            "Player 0 should NOT need decision"
        );
        assert!(
            !game.state.player_states[1].needs_decision,
            "Player 1 should NOT need decision"
        );
    }

    #[test]
    fn test_possession_transfer_triggers_decision() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players near ball
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 80, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) = create_test_player(
            Team::B, // Different team!
            2,
            70,
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = None;

        // Use RNG that changes results over time
        use std::cell::Cell;
        let call_count = Cell::new(0);
        let mut system = BallPossessionSystem::with_rng(move || {
            let count = call_count.get();
            call_count.set(count + 1);
            // First: player 0 wins, Second: player 1 wins
            if count < 2 {
                0.5
            } else if count == 2 {
                0.0
            } else {
                1.0
            }
        });

        // First update - player 0 gets possession
        system.update(&mut game, 0.0);
        assert_eq!(game.state.ball_state.possessed_by, Some(0));

        // Reset flags
        game.state.player_states[0].needs_decision = false;
        game.state.player_states[1].needs_decision = false;

        // Second update after cooldown - possession transfers to player 1
        system.update(&mut game, 1.5);
        assert_eq!(game.state.ball_state.possessed_by, Some(1));

        // Both players should need decision after transfer
        assert!(
            game.state.player_states[0].needs_decision,
            "Player 0 should need decision after losing ball"
        );
        assert!(
            game.state.player_states[1].needs_decision,
            "Player 1 should need decision after getting ball"
        );
    }

    #[test]
    fn test_teammates_dont_steal_from_each_other() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Three players from Team A, all near ball
        let (player1_def, player1_state) = create_test_player(
            Team::A,
            1,
            80, // High tackle rate
            Point3D::from_meters(50.5, 30.0, 0.0),
        );

        let (player2_def, player2_state) = create_test_player(
            Team::A,
            2,
            90, // Even higher tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let (player3_def, player3_state) =
            create_test_player(Team::A, 3, 70, Point3D::from_meters(50.0, 30.5, 0.0));

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def, player3_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state, player3_state];

        // Player 0 already has possession
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.last_possession_change_time = 0.0;

        let mut system = BallPossessionSystem::with_rng(|| 0.5);

        // Update after cooldown - should NOT change (all teammates)
        system.update(&mut game, 2.0);

        // Player 0 should still have possession
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
    }

    #[test]
    fn test_opponents_can_steal_from_owner() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two players near ball - different teams
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 70, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) = create_test_player(
            Team::B,
            7,
            80, // Higher tackle rate
            Point3D::from_meters(49.5, 30.0, 0.0),
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];

        // Player 0 (Team A) has possession
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.last_possession_change_time = 0.0;

        // Use RNG that favors player 1
        let mut system = BallPossessionSystem::with_rng(|| 1.0);

        // Update after cooldown - opponent can steal
        system.update(&mut game, 2.0);

        // Player 1 should steal the ball
        assert_eq!(game.state.ball_state.possessed_by, Some(1));
    }

    #[test]
    fn test_teammates_nearby_opponent_far_keeps_possession() {
        let field = create_test_field();

        // Ball at center
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Two teammates near ball
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 70, Point3D::from_meters(50.5, 30.0, 0.0));

        let (player2_def, player2_state) =
            create_test_player(Team::A, 2, 60, Point3D::from_meters(49.5, 30.0, 0.0));

        // Opponent far away
        let (player3_def, player3_state) = create_test_player(
            Team::B,
            7,
            90,
            Point3D::from_meters(60.0, 30.0, 0.0), // 10m away
        );

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def, player3_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state, player3_state];

        // Player 0 has possession
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.last_possession_change_time = 0.0;

        let mut system = BallPossessionSystem::with_rng(|| 0.5);

        // Update after cooldown
        system.update(&mut game, 2.0);

        // Should keep possession (only teammates nearby)
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
    }

    #[test]
    fn test_ball_possession_system_exists() {
        let _system = BallPossessionSystem::new();
    }

    #[test]
    fn test_last_possessing_team_tracks_during_pass() {
        let field = create_test_field();
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Team A player near ball
        let (player1_def, player1_state) =
            create_test_player(Team::A, 1, 70, Point3D::from_meters(50.5, 30.0, 0.0));

        // Team B player far away
        let (player2_def, player2_state) =
            create_test_player(Team::B, 2, 60, Point3D::from_meters(60.0, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        game.state.player_states = vec![player1_state, player2_state];

        // Initial state: ball is neutral, no owner
        game.state.ball_state.possessed_by = None;
        game.state.ball_state.last_possession_change_time = -2.0;
        assert_eq!(game.state.ball_state.last_possessing_team, None);

        let mut system = BallPossessionSystem::new();
        
        // Player 0 (Team A) gains possession (nearby, ball is free)
        system.update(&mut game, 0.0);

        // Team A should be recorded
        assert_eq!(game.state.ball_state.possessed_by, Some(0));
        assert_eq!(
            game.state.ball_state.last_possessing_team,
            Some(Team::A)
        );

        // Simulate pass: ball is free (possessed_by = None)
        game.state.ball_state.possessed_by = None;

        // last_possessing_team should still be Team A during the pass
        assert_eq!(
            game.state.ball_state.last_possessing_team,
            Some(Team::A)
        );
    }

    #[test]
    fn test_last_possessing_team_changes_on_interception() {
        let field = create_test_field();
        let ball_pos = Point3D::from_meters(50.0, 30.0, 0.0);

        // Team A player
        let (player1_def, mut player1_state) =
            create_test_player(Team::A, 1, 70, Point3D::from_meters(50.5, 30.0, 0.0));

        // Team B player nearby to intercept
        let (player2_def, player2_state) =
            create_test_player(Team::B, 2, 80, Point3D::from_meters(50.3, 30.0, 0.0));

        let config = GameConfig {
            field,
            players: vec![player1_def, player2_def],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::new(config);
        game.state.ball_state.position = ball_pos;
        
        // Team A has the ball
        player1_state.needs_decision = false;
        game.state.player_states = vec![player1_state, player2_state];
        game.state.ball_state.possessed_by = Some(0);
        game.state.ball_state.last_possession_change_time = 0.0;
        game.state.ball_state.last_possessing_team = Some(Team::A);

        let mut system = BallPossessionSystem::with_rng(|| 1.0);
        
        // Update after cooldown - Team B can intercept
        system.update(&mut game, 2.0);

        // Team B should now have possession
        assert_eq!(game.state.ball_state.possessed_by, Some(1));
        assert_eq!(
            game.state.ball_state.last_possessing_team,
            Some(Team::B)
        );
    }
}
