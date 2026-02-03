use mlua::{Lua, LuaSerdeExt, Value};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Errors that can occur during script execution
#[derive(Debug, Clone)]
pub enum ScriptError {
    /// Lua syntax error
    SyntaxError(String),
    /// Runtime error during script execution
    RuntimeError(String),
    /// Error serializing context to Lua
    SerializationError(String),
    /// Error deserializing result from Lua
    DeserializationError(String),
    /// Script function not found
    FunctionNotFound(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::SyntaxError(msg) => write!(f, "Syntax error: {}", msg),
            ScriptError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ScriptError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ScriptError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            ScriptError::FunctionNotFound(msg) => write!(f, "Function not found: {}", msg),
        }
    }
}

impl std::error::Error for ScriptError {}

/// Generic result returned from Lua script.
/// The actual structure depends on the game implementation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Executes Lua scripts with context data.
///
/// This is a generic executor that works with any serializable context
/// and expects scripts to implement a `make_decision()` function.
pub struct LuaExecutor {
    lua: Lua,
    preamble: String,
}

impl LuaExecutor {
    /// Create a new Lua executor with optional preamble code.
    pub fn new(preamble: Option<String>) -> Result<Self, ScriptError> {
        let lua = Lua::new();

        let preamble = preamble.unwrap_or_default();

        // Execute preamble if provided
        if !preamble.is_empty() {
            lua.load(&preamble)
                .exec()
                .map_err(|e| ScriptError::SyntaxError(format!("Preamble error: {}", e)))?;
        }

        Ok(Self { lua, preamble })
    }

    /// Execute a Lua script with the given context.
    ///
    /// The script must define a function with the specified name that returns a table.
    /// The context is made available as a global `context` variable.
    ///
    /// # Arguments
    /// * `script` - The user's Lua code
    /// * `function_name` - Name of the function to call
    /// * `context` - Any serializable context data
    ///
    /// # Returns
    /// A `ScriptResult` containing the returned data as JSON value.
    pub fn execute<T: Serialize>(
        &self,
        script: &str,
        function_name: &str,
        context: &T,
    ) -> Result<ScriptResult, ScriptError> {
        // 1. Serialize context to Lua value
        let lua_context = self
            .lua
            .to_value(context)
            .map_err(|e| ScriptError::SerializationError(e.to_string()))?;

        // 2. Set context as global variable
        self.lua
            .globals()
            .set("context", lua_context)
            .map_err(|e| ScriptError::RuntimeError(e.to_string()))?;

        // 3. Load and execute user script
        self.lua
            .load(script)
            .exec()
            .map_err(|e| {
                let err_msg = e.to_string();
                // Distinguish between syntax and runtime errors
                if err_msg.contains("syntax error") || err_msg.contains("unexpected symbol") {
                    ScriptError::SyntaxError(err_msg)
                } else {
                    ScriptError::RuntimeError(err_msg)
                }
            })?;

        // 4. Call the specified function
        let function: mlua::Function = self
            .lua
            .globals()
            .get(function_name)
            .map_err(|_| {
                ScriptError::FunctionNotFound(format!(
                    "Script must define {}() function",
                    function_name
                ))
            })?;

        let result: Value = function
            .call(())
            .map_err(|e| ScriptError::RuntimeError(e.to_string()))?;

        // 5. Convert result to JSON value for flexibility
        let json_value: serde_json::Value = self
            .lua
            .from_value(result)
            .map_err(|e| ScriptError::DeserializationError(e.to_string()))?;

        Ok(ScriptResult { data: json_value })
    }

    /// Get the preamble code used by this executor
    pub fn preamble(&self) -> &str {
        &self.preamble
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize)]
    struct TestContext {
        player_number: u32,
        elapsed_time: f32,
    }

    #[test]
    fn test_basic_script_execution() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function make_decision()
                return {
                    type = "move",
                    x = 10,
                    y = 20
                }
            end
        "#;

        let context = TestContext {
            player_number: 5,
            elapsed_time: 10.5,
        };

        let result = executor.execute(script, "make_decision", &context).unwrap();
        assert_eq!(result.data["type"], "move");
        assert_eq!(result.data["x"], 10);
        assert_eq!(result.data["y"], 20);
    }

    #[test]
    fn test_script_with_context() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function make_decision()
                return {
                    type = "info",
                    player = context.player_number,
                    time = context.elapsed_time
                }
            end
        "#;

        let context = TestContext {
            player_number: 7,
            elapsed_time: 42.3,
        };

        let result = executor.execute(script, "make_decision", &context).unwrap();
        assert_eq!(result.data["player"], 7);
        // Float comparison with tolerance due to JSON serialization
        let time_value = result.data["time"].as_f64().unwrap();
        assert!((time_value - 42.3).abs() < 0.001);
    }

    #[test]
    fn test_script_with_preamble() {
        let preamble = r#"
            function helper_add(a, b)
                return a + b
            end
        "#;

        let executor = LuaExecutor::new(Some(preamble.to_string())).unwrap();

        let script = r#"
            function make_decision()
                return {
                    result = helper_add(5, 3)
                }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context).unwrap();
        assert_eq!(result.data["result"], 8);
    }

    #[test]
    fn test_missing_function_error() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            -- No make_decision function
            local x = 10
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context);
        assert!(matches!(result, Err(ScriptError::FunctionNotFound(_))));
    }

    #[test]
    fn test_syntax_error() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function make_decision()
                return { invalid syntax here
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context);
        assert!(matches!(result, Err(ScriptError::SyntaxError(_))));
    }

    #[test]
    fn test_runtime_error() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function make_decision()
                error("Intentional error")
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context);
        assert!(matches!(result, Err(ScriptError::RuntimeError(_))));
    }

    #[test]
    fn test_complex_nested_data() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function make_decision()
                return {
                    type = "complex",
                    nested = {
                        array = {1, 2, 3},
                        object = {
                            key = "value"
                        }
                    }
                }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context).unwrap();
        assert_eq!(result.data["type"], "complex");
        assert_eq!(result.data["nested"]["array"][0], 1);
        assert_eq!(result.data["nested"]["object"]["key"], "value");
    }

    #[test]
    fn test_custom_function_name() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function calculate_score()
                return {
                    score = context.player_number * 10
                }
            end
            
            function get_status()
                return {
                    status = "active",
                    time = context.elapsed_time
                }
            end
        "#;

        let context = TestContext {
            player_number: 5,
            elapsed_time: 12.5,
        };

        // Call first function
        let result1 = executor.execute(script, "calculate_score", &context).unwrap();
        assert_eq!(result1.data["score"], 50);

        // Call second function with same executor and script
        let result2 = executor.execute(script, "get_status", &context).unwrap();
        assert_eq!(result2.data["status"], "active");
        let time_value = result2.data["time"].as_f64().unwrap();
        assert!((time_value - 12.5).abs() < 0.001);
    }

    #[test]
    fn test_script_state_resets_on_each_execute() {
        let executor = LuaExecutor::new(None).unwrap();

        let script1 = r#"
            counter = 0
            function increment()
                counter = counter + 1
                return { value = counter }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // First execution sets counter = 1
        let result1 = executor.execute(script1, "increment", &context).unwrap();
        assert_eq!(result1.data["value"], 1);

        // Second execution RELOADS the script, so counter is reset to 0, then incremented to 1
        // This is expected behavior - each execute() reloads the script
        let result2 = executor.execute(script1, "increment", &context).unwrap();
        assert_eq!(result2.data["value"], 1); // Not 2, because script reloaded

        // Third execution also starts from 0
        let result3 = executor.execute(script1, "increment", &context).unwrap();
        assert_eq!(result3.data["value"], 1); // Still 1, not 3
    }

    #[test]
    fn test_preamble_global_state_persists() {
        // Preamble variables DO persist because preamble is loaded once
        let preamble = r#"
            shared_counter = 0
        "#;

        let executor = LuaExecutor::new(Some(preamble.to_string())).unwrap();

        let script = r#"
            function increment_shared()
                shared_counter = shared_counter + 1
                return { value = shared_counter }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // First call: shared_counter goes from 0 to 1
        let result1 = executor.execute(script, "increment_shared", &context).unwrap();
        assert_eq!(result1.data["value"], 1);

        // Second call: shared_counter persists and goes from 1 to 2
        let result2 = executor.execute(script, "increment_shared", &context).unwrap();
        assert_eq!(result2.data["value"], 2);

        // Third call: shared_counter goes from 2 to 3
        let result3 = executor.execute(script, "increment_shared", &context).unwrap();
        assert_eq!(result3.data["value"], 3);
    }

    #[test]
    fn test_context_replaced_between_calls() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function get_player()
                return { player = context.player_number }
            end
        "#;

        let context1 = TestContext {
            player_number: 5,
            elapsed_time: 0.0,
        };

        let context2 = TestContext {
            player_number: 10,
            elapsed_time: 0.0,
        };

        // First call with context1
        let result1 = executor.execute(script, "get_player", &context1).unwrap();
        assert_eq!(result1.data["player"], 5);

        // Second call with context2 - context should be replaced
        let result2 = executor.execute(script, "get_player", &context2).unwrap();
        assert_eq!(result2.data["player"], 10);
    }

    #[test]
    fn test_script_reloading_overwrites_functions() {
        let executor = LuaExecutor::new(None).unwrap();

        let script1 = r#"
            function test_func()
                return { version = 1 }
            end
        "#;

        let script2 = r#"
            function test_func()
                return { version = 2 }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // Load first version
        let result1 = executor.execute(script1, "test_func", &context).unwrap();
        assert_eq!(result1.data["version"], 1);

        // Load second version - should overwrite
        let result2 = executor.execute(script2, "test_func", &context).unwrap();
        assert_eq!(result2.data["version"], 2);
    }

    #[test]
    fn test_preamble_functions_available_to_script() {
        let preamble = r#"
            function math_multiply(a, b)
                return a * b
            end
            
            global_constant = 100
        "#;

        let executor = LuaExecutor::new(Some(preamble.to_string())).unwrap();

        let script = r#"
            function use_preamble()
                return {
                    product = math_multiply(3, 4),
                    constant = global_constant
                }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "use_preamble", &context).unwrap();
        assert_eq!(result.data["product"], 12);
        assert_eq!(result.data["constant"], 100);
    }

    #[test]
    fn test_script_can_overwrite_preamble_functions() {
        let preamble = r#"
            function shared_func()
                return "from_preamble"
            end
        "#;

        let executor = LuaExecutor::new(Some(preamble.to_string())).unwrap();

        let script = r#"
            function shared_func()
                return "from_script"
            end
            
            function call_it()
                return { result = shared_func() }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "call_it", &context).unwrap();
        // Script overwrites preamble function
        assert_eq!(result.data["result"], "from_script");
    }

    #[test]
    fn test_empty_preamble() {
        let executor1 = LuaExecutor::new(Some("".to_string())).unwrap();
        let executor2 = LuaExecutor::new(None).unwrap();

        let script = r#"
            function test()
                return { value = 42 }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // Both should work identically
        let result1 = executor1.execute(script, "test", &context).unwrap();
        let result2 = executor2.execute(script, "test", &context).unwrap();
        assert_eq!(result1.data["value"], 42);
        assert_eq!(result2.data["value"], 42);
    }

    #[test]
    fn test_preamble_syntax_error() {
        let bad_preamble = r#"
            function broken(
                -- Missing closing parenthesis and end
        "#;

        let result = LuaExecutor::new(Some(bad_preamble.to_string()));
        assert!(matches!(result, Err(ScriptError::SyntaxError(_))));
        
        if let Err(ScriptError::SyntaxError(msg)) = result {
            assert!(msg.contains("Preamble error"));
        }
    }

    #[test]
    fn test_function_returns_non_table_number() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function return_number()
                return 42
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "return_number", &context).unwrap();
        assert_eq!(result.data, 42);
    }

    #[test]
    fn test_function_returns_string() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function return_string()
                return "hello world"
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "return_string", &context).unwrap();
        assert_eq!(result.data, "hello world");
    }

    #[test]
    fn test_function_returns_boolean() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function return_true()
                return true
            end
            
            function return_false()
                return false
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result1 = executor.execute(script, "return_true", &context).unwrap();
        assert_eq!(result1.data, true);

        let result2 = executor.execute(script, "return_false", &context).unwrap();
        assert_eq!(result2.data, false);
    }

    #[test]
    fn test_function_returns_nil() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function return_nil()
                return nil
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "return_nil", &context).unwrap();
        assert!(result.data.is_null());
    }

    #[test]
    fn test_function_returns_empty_table() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function return_empty()
                return {}
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "return_empty", &context).unwrap();
        assert!(result.data.is_object());
        assert_eq!(result.data.as_object().unwrap().len(), 0);
    }

    #[test]
    fn test_empty_script() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = "";

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // Should fail because no function is defined
        let result = executor.execute(script, "make_decision", &context);
        assert!(matches!(result, Err(ScriptError::FunctionNotFound(_))));
    }

    #[test]
    fn test_script_with_only_comments() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            -- This is a comment
            -- Another comment
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        let result = executor.execute(script, "make_decision", &context);
        assert!(matches!(result, Err(ScriptError::FunctionNotFound(_))));
    }

    #[test]
    fn test_each_execution_receives_correct_context() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function get_info()
                return {
                    player = context.player_number,
                    time = context.elapsed_time
                }
            end
        "#;

        // Execute with different contexts - each should receive its own context
        for i in 1..=5 {
            let context = TestContext {
                player_number: i * 10,
                elapsed_time: i as f32 * 1.5,
            };

            let result = executor.execute(script, "get_info", &context).unwrap();
            assert_eq!(result.data["player"], i * 10);
            let time_value = result.data["time"].as_f64().unwrap();
            assert!((time_value - (i as f64 * 1.5)).abs() < 0.001);
        }
    }

    #[test]
    fn test_accessing_undefined_context_field_returns_null() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function access_missing()
                return {
                    missing = context.nonexistent_field
                }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // Lua returns nil for missing fields, which becomes null in JSON
        let result = executor.execute(script, "access_missing", &context).unwrap();
        assert!(result.data["missing"].is_null());
    }

    #[test]
    fn test_division_by_zero_returns_infinity_not_error() {
        let executor = LuaExecutor::new(None).unwrap();

        let script = r#"
            function divide_by_zero()
                local x = 10 / 0
                return { result = x }
            end
        "#;

        let context = TestContext {
            player_number: 1,
            elapsed_time: 0.0,
        };

        // Lua allows division by zero (returns inf), not a runtime error
        let result = executor.execute(script, "divide_by_zero", &context).unwrap();
        // JSON represents infinity as null
        assert!(result.data["result"].is_null() || result.data["result"].is_number());
    }
}
