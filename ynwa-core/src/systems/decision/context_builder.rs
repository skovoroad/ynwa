use crate::game::Game;
use crate::orientation::flip_point_orientation;
use crate::team::Team;
use serde_json::json;
use uom::si::length::meter;

use super::DecisionError;

/// Builds minimal game context for Lua scripts
///
/// TODO: Consider alternative approach using intermediate serde structures
/// (similar to LuaDecision in lua_format.rs) for better type safety and
/// self-documentation. Current json!() macro approach is simpler and sufficient
/// for performance, but explicit structures would make the Lua contract more clear.
pub struct ContextBuilder;

impl ContextBuilder {
    /// Builds context for player's Lua script.
    /// Team B sees coordinates in their own orientation (flipped from display).
    pub fn build(game: &Game, player_index: usize) -> Result<serde_json::Value, DecisionError> {
        let config = game.config();
        let state = game.state();
        
        if player_index >= config.players.len() {
            return Err(DecisionError::RuntimeError(format!(
                "Invalid player_index: {}",
                player_index
            )));
        }
        
        let player_def = &config.players[player_index];
        let player_state = &state.player_states[player_index];
        let player_team = player_def.team;
        
        // Get field dimensions for coordinate transformation
        let field_width = config.field.width().get::<meter>();
        let field_length = config.field.length().get::<meter>();
        
        let context = json!({
            "me": {
                "team": format!("{:?}", player_team),
                "number": player_def.number,
                "position": Self::position_to_json(&player_state.position, player_team, field_width, field_length)
            },
            "teammates": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_team,
                player_team, // viewer_team for coordinate transformation
                field_width,
                field_length
            ),
            "opponents": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_team.opposite(),
                player_team, // viewer_team for coordinate transformation
                field_width,
                field_length
            ),
            "ball": {
                "position": Self::position_to_json(&state.ball_state.position, player_team, field_width, field_length)
            },
            "game": {
                "elapsed_time": state.elapsed_time
            }
        });
        
        Ok(context)
    }
    
    fn position_to_json(
        pos: &crate::field::zones::Point3D,
        viewer_team: Team,
        field_width: f32,
        field_length: f32,
    ) -> serde_json::Value {
        // For Team B, flip coordinates from display orientation to Team B perspective
        let transformed_pos = if viewer_team == Team::B {
            flip_point_orientation(pos, field_width, field_length)
        } else {
            *pos
        };
        
        json!({
            "x": transformed_pos.x.get::<meter>(),
            "y": transformed_pos.y.get::<meter>(),
            "z": transformed_pos.z.get::<meter>()
        })
    }
    
    fn build_team_positions(
        players: &[crate::game::PlayerDef],
        states: &[crate::game::PlayerState],
        team: Team,
        viewer_team: Team,
        field_width: f32,
        field_length: f32,
    ) -> serde_json::Value {
        json!(
            players
                .iter()
                .zip(states.iter())
                .filter(|(def, _)| def.team == team)
                .map(|(def, state)| {
                    json!({
                        "number": def.number,
                        "position": Self::position_to_json(&state.position, viewer_team, field_width, field_length)
                    })
                })
                .collect::<Vec<_>>()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_game_with_players() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region_a = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
            grid_dims,
        )
        .unwrap();
        
        let start_region_b = Region::new(
            Team::B,
            GridCell::new(15, 15).unwrap(),
            GridCell::new(16, 16).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![
                PlayerDef::new(
                    Team::A,
                    1,
                    "Player A1".to_string(),
                    50,
                    50,
                    "function make_decision() return {} end".to_string(),
                    start_region_a.clone(),
                ),
                PlayerDef::new(
                    Team::A,
                    2,
                    "Player A2".to_string(),
                    60,
                    60,
                    "function make_decision() return {} end".to_string(),
                    start_region_a,
                ),
                PlayerDef::new(
                    Team::B,
                    7,
                    "Player B7".to_string(),
                    55,
                    55,
                    "function make_decision() return {} end".to_string(),
                    start_region_b,
                ),
            ],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
        };

        Game::new(config)
    }

    #[test]
    fn test_context_builder_creates_valid_json() {
        let game = create_test_game_with_players();
        let result = ContextBuilder::build(&game, 0);
        
        assert!(result.is_ok());
        let context = result.unwrap();
        
        // Check structure
        assert!(context.get("me").is_some());
        assert!(context.get("teammates").is_some());
        assert!(context.get("opponents").is_some());
        assert!(context.get("ball").is_some());
        assert!(context.get("game").is_some());
    }

    #[test]
    fn test_context_me_fields() {
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap();
        
        let me = context.get("me").unwrap();
        assert_eq!(me.get("team").unwrap().as_str().unwrap(), "A");
        assert_eq!(me.get("number").unwrap().as_u64().unwrap(), 1);
        
        let position = me.get("position").unwrap();
        assert!(position.get("x").is_some());
        assert!(position.get("y").is_some());
        assert!(position.get("z").is_some());
    }

    #[test]
    fn test_context_teammates_filtered_correctly() {
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap();
        
        let teammates = context.get("teammates").unwrap().as_array().unwrap();
        
        // Should have 2 teammates (both Team A players including self)
        assert_eq!(teammates.len(), 2);
        
        // Check numbers
        let numbers: Vec<u64> = teammates
            .iter()
            .map(|p| p.get("number").unwrap().as_u64().unwrap())
            .collect();
        assert!(numbers.contains(&1));
        assert!(numbers.contains(&2));
    }

    #[test]
    fn test_context_opponents_filtered_correctly() {
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap();
        
        let opponents = context.get("opponents").unwrap().as_array().unwrap();
        
        // Should have 1 opponent (Team B player)
        assert_eq!(opponents.len(), 1);
        assert_eq!(
            opponents[0].get("number").unwrap().as_u64().unwrap(),
            7
        );
    }

    #[test]
    fn test_context_ball_position() {
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap();
        
        let ball = context.get("ball").unwrap();
        let position = ball.get("position").unwrap();
        
        assert!(position.get("x").is_some());
        assert!(position.get("y").is_some());
        assert!(position.get("z").is_some());
    }

    #[test]
    fn test_context_game_time() {
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap();
        
        let game_info = context.get("game").unwrap();
        let elapsed_time = game_info.get("elapsed_time").unwrap().as_f64().unwrap();
        
        assert_eq!(elapsed_time, 0.0);
    }

    #[test]
    fn test_context_builder_invalid_player_index() {
        let game = create_test_game_with_players();
        let result = ContextBuilder::build(&game, 999);
        
        assert!(result.is_err());
        match result {
            Err(DecisionError::RuntimeError(msg)) => {
                assert!(msg.contains("Invalid player_index"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_team_a_sees_display_coordinates() {
        // Team A should see coordinates in display orientation (unchanged)
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 0).unwrap(); // Player 0 is Team A
        
        let me = context.get("me").unwrap();
        let position = me.get("position").unwrap();
        
        // Team A player at start_region A10:K11 should see their actual position
        // The center of region A10:K11 is approximately (3.846, 0, 22.727) for 26x44 grid on 100x60 field
        let x = position.get("x").unwrap().as_f64().unwrap();
        let z = position.get("z").unwrap().as_f64().unwrap();
        
        // Check that coordinates are in the left half of the field (Team A side)
        assert!(x < 50.0, "Team A player should be on left side: x={}", x);
        assert!(z > 0.0 && z < 60.0, "z should be within field bounds: z={}", z);
    }

    #[test]
    fn test_team_b_sees_flipped_coordinates() {
        // Team B should see coordinates flipped to their perspective
        let game = create_test_game_with_players();
        let context = ContextBuilder::build(&game, 2).unwrap(); // Player 2 is Team B
        
        let me = context.get("me").unwrap();
        let position = me.get("position").unwrap();
        
        // Team B player at start_region B15:P16 (in Team B coordinates)
        // In display coords, this is on the right side (x > 50)
        // But in Team B's view, they should see themselves on the LEFT side (x < 50)
        let x = position.get("x").unwrap().as_f64().unwrap();
        let z = position.get("z").unwrap().as_f64().unwrap();
        
        // After flipping, Team B player should see themselves on their left side
        assert!(x < 50.0, "Team B player should see themselves on left side (flipped): x={}", x);
        assert!(z > 0.0 && z < 60.0, "z should be within field bounds: z={}", z);
    }

    #[test]
    fn test_team_b_sees_ball_in_own_coordinates() {
        // Team B should see ball position flipped to their perspective
        let game = create_test_game_with_players();
        let context_a = ContextBuilder::build(&game, 0).unwrap(); // Team A player
        let context_b = ContextBuilder::build(&game, 2).unwrap(); // Team B player
        
        let ball_a = context_a.get("ball").unwrap().get("position").unwrap();
        let ball_b = context_b.get("ball").unwrap().get("position").unwrap();
        
        let x_a = ball_a.get("x").unwrap().as_f64().unwrap();
        let z_a = ball_a.get("z").unwrap().as_f64().unwrap();
        
        let x_b = ball_b.get("x").unwrap().as_f64().unwrap();
        let z_b = ball_b.get("z").unwrap().as_f64().unwrap();
        
        // Ball coordinates should be flipped for Team B
        // x: 100 - x_a, z: 60 - z_a
        let expected_x_b = 100.0 - x_a;
        let expected_z_b = 60.0 - z_a;
        
        assert!((x_b - expected_x_b).abs() < 0.01, 
            "Ball x for Team B should be flipped: expected {}, got {}", expected_x_b, x_b);
        assert!((z_b - expected_z_b).abs() < 0.01,
            "Ball z for Team B should be flipped: expected {}, got {}", expected_z_b, z_b);
    }

    #[test]
    fn test_team_b_sees_opponents_in_own_coordinates() {
        // Team B should see opponent (Team A) positions flipped
        let game = create_test_game_with_players();
        let context_b = ContextBuilder::build(&game, 2).unwrap(); // Team B player
        
        let opponents = context_b.get("opponents").unwrap().as_array().unwrap();
        
        // Should have 2 opponents (both Team A players)
        assert_eq!(opponents.len(), 2);
        
        // Check that opponent positions are in Team B's coordinate system
        for opponent in opponents {
            let position = opponent.get("position").unwrap();
            let x = position.get("x").unwrap().as_f64().unwrap();
            let z = position.get("z").unwrap().as_f64().unwrap();
            
            // Opponents (Team A) should appear on the RIGHT side from Team B's perspective
            assert!(x > 50.0, "Opponents should be on right side in Team B coords: x={}", x);
            assert!(z > 0.0 && z < 60.0, "z should be within field bounds: z={}", z);
        }
    }

    #[test]
    fn test_team_b_sees_teammates_in_own_coordinates() {
        // Team B should see teammate positions in their coordinate system
        let game = create_test_game_with_players();
        let context_b = ContextBuilder::build(&game, 2).unwrap(); // Team B player
        
        let teammates = context_b.get("teammates").unwrap().as_array().unwrap();
        
        // Should have 1 teammate (only 1 Team B player in our test)
        assert_eq!(teammates.len(), 1);
        
        let position = teammates[0].get("position").unwrap();
        let x = position.get("x").unwrap().as_f64().unwrap();
        
        // Teammate (self) should be on the LEFT side from Team B's perspective
        assert!(x < 50.0, "Teammate should be on left side in Team B coords: x={}", x);
    }
}
