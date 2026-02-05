/// Lua-specific serialization format for decisions.
///
/// These structures represent the JSON format that Lua scripts must return.
/// They validate the decision format without converting to game-specific types.
use serde::{Deserialize, Serialize};

/// Top-level decision returned by Lua scripts
///
/// Lua script must return a table like:
/// - `{action = "stop"}` for stopping
/// - `{action = "run", target_type = "cell", target = "A5"}` for running to cell
/// - `{action = "run", target_type = "region", target = {from = "A5", to = "C7"}}` for region
/// - `{action = "run", target_type = "point", target = {x = 10.5, z = 20.0}}` for point
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum LuaDecision {
    Stop,
    Run {
        target_type: String,
        target: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_lua_decision_stop() {
        let json = json!({"action": "stop"});
        let decision: LuaDecision = serde_json::from_value(json).unwrap();
        assert!(matches!(decision, LuaDecision::Stop));
    }

    #[test]
    fn test_lua_decision_run_cell() {
        let json = json!({
            "action": "run",
            "target_type": "cell",
            "target": "A5"
        });
        let decision: LuaDecision = serde_json::from_value(json).unwrap();
        match decision {
            LuaDecision::Run { target_type, .. } => {
                assert_eq!(target_type, "cell");
            }
            _ => panic!("Expected Run"),
        }
    }

    #[test]
    fn test_lua_decision_run_point() {
        let json = json!({
            "action": "run",
            "target_type": "point",
            "target": {"x": 10.5, "z": 20.0}
        });
        let decision: LuaDecision = serde_json::from_value(json).unwrap();
        match decision {
            LuaDecision::Run { target_type, .. } => {
                assert_eq!(target_type, "point");
            }
            _ => panic!("Expected Run"),
        }
    }

    #[test]
    fn test_lua_decision_run_region() {
        let json = json!({
            "action": "run",
            "target_type": "region",
            "target": {"from": "A5", "to": "C7"}
        });
        let decision: LuaDecision = serde_json::from_value(json).unwrap();
        match decision {
            LuaDecision::Run { target_type, .. } => {
                assert_eq!(target_type, "region");
            }
            _ => panic!("Expected Run"),
        }
    }
}
