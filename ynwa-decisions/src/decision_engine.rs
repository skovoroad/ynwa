use crate::lua_executor::LuaExecutor;
use crate::lua_format::LuaDecision;
use serde_json::Value as JsonValue;
use std::time::Duration;

/// Error types for DecisionEngine
#[derive(Debug)]
pub enum DecisionEngineError {
    ScriptError(String),
    RuntimeError(String),
    Timeout(String),
    InvalidPlayerIndex(usize),
    InvalidConfig(String),
}

impl std::fmt::Display for DecisionEngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecisionEngineError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            DecisionEngineError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            DecisionEngineError::Timeout(msg) => write!(f, "Timeout: {}", msg),
            DecisionEngineError::InvalidPlayerIndex(idx) => {
                write!(f, "Invalid player index: {}", idx)
            }
            DecisionEngineError::InvalidConfig(msg) => write!(f, "Invalid config: {}", msg),
        }
    }
}

impl std::error::Error for DecisionEngineError {}

/// Decision engine that executes Lua scripts for game AI
///
/// # JSON Contract
///
/// ## Initialization
/// Expects JSON config with player scripts:
/// ```json
/// {
///   "players": [
///     {"script": "function make_decision() ... end"},
///     ...
///   ]
/// }
/// ```
///
/// ## Input Context
/// Game state as JSON (structure defined by game engine)
///
/// ## Output Decision
/// Returns JSON decision:
/// ```json
/// {
///   "action": "stop"
/// }
/// ```
/// or
/// ```json
/// {
///   "action": "run",
///   "target_type": "point",
///   "target": {"x": 10.5, "z": 20.0}
/// }
/// ```
#[derive(Debug)]
pub struct DecisionEngine {
    /// One Lua executor per player (isolated VMs)
    executors: Vec<LuaExecutor>,
    /// Player scripts, indexed by player_index
    scripts: Vec<String>,
}

impl DecisionEngine {
    /// Create new DecisionEngine from JSON config
    ///
    /// # Example
    /// ```json
    /// {
    ///   "players": [
    ///     {"script": "function make_decision() return {action = 'stop'} end"},
    ///     {"script": "function make_decision() return {action = 'stop'} end"}
    ///   ]
    /// }
    /// ```
    pub fn new(config: &JsonValue) -> Result<Self, DecisionEngineError> {
        let players = config
            .get("players")
            .and_then(|p| p.as_array())
            .ok_or_else(|| {
                DecisionEngineError::InvalidConfig("Missing 'players' array".to_string())
            })?;

        let mut executors = Vec::with_capacity(players.len());
        let mut scripts = Vec::with_capacity(players.len());

        for (player_index, player) in players.iter().enumerate() {
            let script = player
                .get("script")
                .and_then(|s| s.as_str())
                .ok_or_else(|| {
                    DecisionEngineError::InvalidConfig(format!(
                        "Player {} missing 'script' field",
                        player_index
                    ))
                })?;

            // Create executor with 100ms timeout
            let executor = LuaExecutor::new(None, Some(Duration::from_millis(100)))
                .map_err(|e| {
                    DecisionEngineError::RuntimeError(format!(
                        "Failed to create executor for player {}: {}",
                        player_index, e
                    ))
                })?;

            executors.push(executor);
            scripts.push(script.to_string());
        }

        Ok(Self { executors, scripts })
    }

    /// Make decision for a player
    ///
    /// # Arguments
    /// * `player_index` - Index of player (0-based)
    /// * `context` - Game state as JSON
    ///
    /// # Returns
    /// JSON decision
    pub fn make_decision(
        &self,
        player_index: usize,
        context: &JsonValue,
    ) -> Result<JsonValue, DecisionEngineError> {
        // Validate player index
        let executor = self.executors.get(player_index).ok_or_else(|| {
            DecisionEngineError::InvalidPlayerIndex(player_index)
        })?;

        let script = &self.scripts[player_index];

        // Execute script with context
        let result = executor
            .execute(script, "make_decision", context)
            .map_err(Self::convert_script_error)?;

        // Validate that result is a valid LuaDecision (but return raw JSON)
        let _lua_decision: LuaDecision =
            serde_json::from_value(result.data.clone()).map_err(|e| {
                DecisionEngineError::ScriptError(format!("Invalid decision format: {}", e))
            })?;

        Ok(result.data)
    }

    /// Convert ScriptError to DecisionEngineError
    fn convert_script_error(error: crate::lua_executor::ScriptError) -> DecisionEngineError {
        use crate::lua_executor::ScriptError;

        match error {
            ScriptError::SyntaxError(msg) => DecisionEngineError::ScriptError(msg),
            ScriptError::RuntimeError(msg) => DecisionEngineError::RuntimeError(msg),
            ScriptError::Timeout(msg) => DecisionEngineError::Timeout(msg),
            ScriptError::FunctionNotFound(msg) => DecisionEngineError::ScriptError(msg),
            ScriptError::SerializationError(msg) => DecisionEngineError::RuntimeError(msg),
            ScriptError::DeserializationError(msg) => DecisionEngineError::ScriptError(msg),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn create_test_config(script: &str) -> JsonValue {
        json!({
            "players": [
                {"script": script}
            ]
        })
    }

    #[test]
    fn test_decision_engine_creation() {
        let config = create_test_config(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let result = DecisionEngine::new(&config);
        assert!(result.is_ok());

        let engine = result.unwrap();
        assert_eq!(engine.executors.len(), 1);
        assert_eq!(engine.scripts.len(), 1);
    }

    #[test]
    fn test_decision_engine_stop_decision() {
        let config = create_test_config(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let engine = DecisionEngine::new(&config).unwrap();
        let context = json!({"me": {"number": 1}});
        let decision = engine.make_decision(0, &context);

        assert!(decision.is_ok());
        let dec = decision.unwrap();
        assert_eq!(dec.get("action").and_then(|a| a.as_str()), Some("stop"));
    }

    #[test]
    fn test_decision_engine_run_decision() {
        let config = create_test_config(
            r#"
            function make_decision()
                return {
                    action = "run",
                    target_type = "point",
                    target = {x = 10.5, z = 20.0}
                }
            end
            "#,
        );

        let engine = DecisionEngine::new(&config).unwrap();
        let context = json!({"me": {"number": 1}});
        let decision = engine.make_decision(0, &context);

        assert!(decision.is_ok());
        let dec = decision.unwrap();
        assert_eq!(dec.get("action").and_then(|a| a.as_str()), Some("run"));
        assert_eq!(
            dec.get("target_type").and_then(|t| t.as_str()),
            Some("point")
        );
    }

    #[test]
    fn test_decision_engine_receives_context() {
        let config = create_test_config(
            r#"
            function make_decision()
                local my_number = context.me.number
                if my_number == 1 then
                    return {action = "stop"}
                else
                    return {
                        action = "run",
                        target_type = "point",
                        target = {x = 0.0, z = 0.0}
                    }
                end
            end
            "#,
        );

        let engine = DecisionEngine::new(&config).unwrap();
        let context = json!({"me": {"number": 1}});
        let decision = engine.make_decision(0, &context);

        assert!(decision.is_ok());
        assert_eq!(
            decision.unwrap().get("action").and_then(|a| a.as_str()),
            Some("stop")
        );
    }

    #[test]
    fn test_decision_engine_syntax_error() {
        let config = create_test_config(
            r#"
            function make_decision()
                return {action = "stop"  -- Missing closing brace
            end
            "#,
        );

        let engine = DecisionEngine::new(&config).unwrap();
        let context = json!({"me": {"number": 1}});
        let decision = engine.make_decision(0, &context);

        assert!(decision.is_err());
        assert!(matches!(
            decision.unwrap_err(),
            DecisionEngineError::ScriptError(_)
        ));
    }

    #[test]
    fn test_decision_engine_invalid_player_index() {
        let config = create_test_config(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let engine = DecisionEngine::new(&config).unwrap();
        let context = json!({"me": {"number": 1}});
        let decision = engine.make_decision(999, &context);

        assert!(decision.is_err());
        assert!(matches!(
            decision.unwrap_err(),
            DecisionEngineError::InvalidPlayerIndex(999)
        ));
    }

    #[test]
    fn test_decision_engine_missing_config_players() {
        let config = json!({});
        let result = DecisionEngine::new(&config);

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            DecisionEngineError::InvalidConfig(_)
        ));
    }

    #[test]
    fn test_decision_engine_multiple_players() {
        let config = json!({
            "players": [
                {"script": "function make_decision() return {action = 'stop'} end"},
                {"script": "function make_decision() return {action = 'stop'} end"}
            ]
        });

        let engine = DecisionEngine::new(&config).unwrap();
        assert_eq!(engine.executors.len(), 2);

        let context = json!({"me": {"number": 1}});
        assert!(engine.make_decision(0, &context).is_ok());
        assert!(engine.make_decision(1, &context).is_ok());
    }
}
