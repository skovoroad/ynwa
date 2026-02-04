use crate::game::Game;
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
    /// Build context with minimal information: positions, ball, time
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
        
        let context = json!({
            "me": {
                "team": format!("{:?}", player_def.team),
                "number": player_def.number,
                "position": Self::position_to_json(&player_state.position)
            },
            "teammates": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_def.team
            ),
            "opponents": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_def.team.opposite()
            ),
            "ball": {
                "position": Self::position_to_json(&state.ball_state.position)
            },
            "game": {
                "elapsed_time": state.elapsed_time
            }
        });
        
        Ok(context)
    }
    
    fn position_to_json(pos: &crate::field::zones::Point3D) -> serde_json::Value {
        json!({
            "x": pos.x.get::<meter>(),
            "y": pos.y.get::<meter>(),
            "z": pos.z.get::<meter>()
        })
    }
    
    fn build_team_positions(
        players: &[crate::game::PlayerDef],
        states: &[crate::game::PlayerState],
        team: Team,
    ) -> serde_json::Value {
        json!(
            players
                .iter()
                .zip(states.iter())
                .filter(|(def, _)| def.team == team)
                .map(|(def, state)| {
                    json!({
                        "number": def.number,
                        "position": Self::position_to_json(&state.position)
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
}
