use crate::game::{Decision, Game, GameConfig};
use crate::scripting::LuaExecutor;
use std::time::Duration;

use super::{ContextBuilder, DecisionError, DecisionMaker, LuaDecision};

/// DecisionMaker that executes Lua scripts for each player
pub struct LuaDecisionMaker {
    /// One executor per player (isolated VMs), indexed by player_index
    executors: Vec<LuaExecutor>,
}

impl LuaDecisionMaker {
    /// Create new LuaDecisionMaker with executors for all players
    pub fn new(game_config: &GameConfig) -> Result<Self, DecisionError> {
        let mut executors = Vec::with_capacity(game_config.players.len());

        for (player_index, _player_def) in game_config.players.iter().enumerate() {
            // Create executor with 100ms timeout
            let executor = LuaExecutor::new(
                None, // No preamble yet
                Some(Duration::from_millis(100)),
            )
            .map_err(|e| {
                DecisionError::RuntimeError(format!(
                    "Failed to create executor for player {}: {}",
                    player_index, e
                ))
            })?;

            // Note: Scripts are executed on-demand, not pre-loaded
            // This allows reload_script() to be called later if needed

            executors.push(executor);
        }

        Ok(Self { executors })
    }
}

impl DecisionMaker for LuaDecisionMaker {
    fn make_decision(&mut self, game: &Game, player_index: usize) -> Result<Decision, DecisionError> {
        // Get executor for this player
        let executor = self.executors.get(player_index).ok_or_else(|| {
            DecisionError::RuntimeError(format!("No executor for player {}", player_index))
        })?;

        // Get player's script
        let player_script = &game.config().players[player_index].script;

        // Build minimal context
        let context = ContextBuilder::build(game, player_index)?;

        // Execute script with context
        let result = executor
            .execute(player_script, "make_decision", &context)
            .map_err(|e| Self::convert_script_error(e))?;

        // Parse result into Decision using Lua format
        let lua_decision: LuaDecision = serde_json::from_value(result.data)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid decision format: {}", e)))?;
        
        lua_decision.into_decision()
    }
}

impl LuaDecisionMaker {
    /// Convert ScriptError to DecisionError
    fn convert_script_error(error: crate::scripting::ScriptError) -> DecisionError {
        use crate::scripting::ScriptError;
        
        match error {
            ScriptError::SyntaxError(msg) => DecisionError::ScriptError(msg),
            ScriptError::RuntimeError(msg) => DecisionError::RuntimeError(msg),
            ScriptError::Timeout(msg) => DecisionError::Timeout(msg),
            ScriptError::FunctionNotFound(msg) => DecisionError::ScriptError(msg),
            ScriptError::SerializationError(msg) => DecisionError::RuntimeError(msg),
            ScriptError::DeserializationError(msg) => DecisionError::ScriptError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, DecisionTarget, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};
    use crate::team::Team;

    fn create_test_game_with_script(script: &str) -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                50,
                50,
                script.to_string(),
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
        };

        Game::new(config)
    }

    #[test]
    fn test_lua_decision_maker_creates_executors() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let result = LuaDecisionMaker::new(game.config());
        assert!(result.is_ok());

        let maker = result.unwrap();
        assert_eq!(maker.executors.len(), 1);
    }

    #[test]
    fn test_lua_decision_maker_executes_stop_decision() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        assert!(matches!(decision.unwrap(), Decision::Stop));
    }

    #[test]
    fn test_lua_decision_maker_executes_run_to_cell() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {
                    action = "run",
                    target_type = "cell",
                    target = "A5"
                }
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        match decision.unwrap() {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                assert_eq!(cell.col, 1);
                assert_eq!(cell.row, 5);
            }
            _ => panic!("Expected Run to GridCell"),
        }
    }

    #[test]
    fn test_lua_decision_maker_receives_context() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                -- Access context provided by host
                local my_number = context.me.number
                local ball_x = context.ball.position.x
                
                -- Return decision based on context
                if my_number == 1 then
                    return {action = "stop"}
                else
                    return {
                        action = "run",
                        target_type = "cell",
                        target = "B2"
                    }
                end
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        // Player number is 1, so should return Stop
        assert!(matches!(decision.unwrap(), Decision::Stop));
    }

    #[test]
    fn test_lua_decision_maker_syntax_error() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"  -- Missing closing brace
            end
            "#,
        );

        let result = LuaDecisionMaker::new(game.config());
        assert!(result.is_ok()); // Creation succeeds, error happens during execution

        let mut maker = result.unwrap();
        let decision = maker.make_decision(&game, 0);
        
        assert!(decision.is_err());
        match decision.unwrap_err() {
            DecisionError::ScriptError(msg) => {
                assert!(!msg.is_empty());
            }
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_lua_decision_maker_runtime_error() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                error("Runtime error test")
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_err());
        match decision.unwrap_err() {
            DecisionError::RuntimeError(msg) => {
                assert!(msg.contains("Runtime error test"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_lua_decision_maker_missing_function() {
        let game = create_test_game_with_script(
            r#"
            -- No make_decision function defined
            function other_function()
                return true
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_err());
        assert!(matches!(
            decision.unwrap_err(),
            DecisionError::ScriptError(_)
        ));
    }

    #[test]
    fn test_lua_decision_maker_invalid_return_format() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return "just a string"
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_err());
        match decision.unwrap_err() {
            DecisionError::ScriptError(msg) => {
                // serde error message for invalid type
                assert!(msg.contains("Invalid decision format"));
            }
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_lua_decision_maker_multiple_players() {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
            grid_dims,
        )
        .unwrap();

        let config = GameConfig {
            field,
            players: vec![
                PlayerDef::new(
                    Team::A,
                    1,
                    "Player 1".to_string(),
                    50,
                    50,
                    r#"
                    function make_decision()
                        return {action = "stop"}
                    end
                    "#
                    .to_string(),
                    start_region.clone(),
                ),
                PlayerDef::new(
                    Team::A,
                    2,
                    "Player 2".to_string(),
                    60,
                    60,
                    r#"
                    function make_decision()
                        return {
                            action = "run",
                            target_type = "cell",
                            target = "C3"
                        }
                    end
                    "#
                    .to_string(),
                    start_region,
                ),
            ],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
        };

        let game = Game::new(config);
        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();

        // Player 0 should stop
        let decision0 = maker.make_decision(&game, 0).unwrap();
        assert!(matches!(decision0, Decision::Stop));

        // Player 1 should run to C3
        let decision1 = maker.make_decision(&game, 1).unwrap();
        match decision1 {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                assert_eq!(cell.col, 3);
                assert_eq!(cell.row, 3);
            }
            _ => panic!("Expected Run to GridCell"),
        }
    }

    #[test]
    fn test_lua_decision_maker_invalid_player_index() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 999);

        assert!(decision.is_err());
        match decision.unwrap_err() {
            DecisionError::RuntimeError(msg) => {
                assert!(msg.contains("No executor"));
            }
            _ => panic!("Expected RuntimeError"),
        }
    }

    #[test]
    fn test_lua_decision_maker_timeout() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                -- Infinite loop to trigger timeout
                while true do
                    local x = 1 + 1
                end
                return {action = "stop"}
            end
            "#,
        );

        let mut maker = LuaDecisionMaker::new(game.config()).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_err());
        // Should be Timeout error
        match decision.unwrap_err() {
            DecisionError::Timeout(_) => {
                // Success
            }
            other => panic!("Expected Timeout error, got {:?}", other),
        }
    }
}
