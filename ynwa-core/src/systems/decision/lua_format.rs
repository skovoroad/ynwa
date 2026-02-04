/// Lua-specific serialization format for decisions.
///
/// These structures represent the JSON format that Lua scripts must return.
/// They are separate from domain types (Decision, DecisionTarget) to:
/// 1. Keep domain layer clean from serialization concerns
/// 2. Allow format changes without affecting game logic
/// 3. Make Lua script contract explicit and documented
use crate::field::zones::Point3D;
use crate::game::{Decision, DecisionTarget};
use crate::region::{GridCell, Region};
use serde::{Deserialize, Serialize};
use uom::si::f32::Length;
use uom::si::length::meter;

use super::DecisionError;

/// Top-level decision returned by Lua scripts
///
/// Lua script must return a table like:
/// - `{action = "stop"}` for stopping
/// - `{action = "run", target_type = "cell", target = "A5"}` for running to cell
/// - `{action = "run", target_type = "region", target = {from = "A5", to = "C7"}}` for region
/// - `{action = "run", target_type = "point", target = {x = 10.5, z = 20.0}}` for point
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum LuaDecision {
    Stop,
    Run {
        target_type: String,
        target: serde_json::Value,
    },
}

/// Target for "run" action - cell identifier (e.g., "A5")
#[derive(Debug, Deserialize, Serialize)]
pub struct LuaCellTarget {
    pub cell: String,
}

/// Target for "run" action - region defined by two cells
#[derive(Debug, Deserialize, Serialize)]
pub struct LuaRegionTarget {
    pub from: String,
    pub to: String,
}

/// Target for "run" action - point in 3D space
#[derive(Debug, Deserialize, Serialize)]
pub struct LuaPointTarget {
    pub x: f32,
    pub z: f32,
    #[serde(default)]
    pub y: f32,
}

impl LuaDecision {
    /// Convert Lua format decision to domain Decision type
    pub fn into_decision(self) -> Result<Decision, DecisionError> {
        match self {
            LuaDecision::Stop => Ok(Decision::Stop),
            LuaDecision::Run {
                target_type,
                target,
            } => {
                let decision_target = match target_type.as_str() {
                    "cell" => Self::parse_cell_target(target)?,
                    "region" => Self::parse_region_target(target)?,
                    "point" => Self::parse_point_target(target)?,
                    _ => {
                        return Err(DecisionError::ScriptError(format!(
                            "Unknown target_type: {}",
                            target_type
                        )))
                    }
                };
                Ok(Decision::Run(decision_target))
            }
        }
    }

    fn parse_cell_target(value: serde_json::Value) -> Result<DecisionTarget, DecisionError> {
        // Try direct string first (simple format: "A5")
        if let Some(cell_str) = value.as_str() {
            let cell = GridCell::from_notation(cell_str).map_err(|e| {
                DecisionError::ScriptError(format!("Invalid cell notation: {}", e))
            })?;
            return Ok(DecisionTarget::GridCell(cell));
        }

        // Try structured format: {cell = "A5"}
        let cell_target: LuaCellTarget = serde_json::from_value(value)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid cell target: {}", e)))?;

        let cell = GridCell::from_notation(&cell_target.cell).map_err(|e| {
            DecisionError::ScriptError(format!("Invalid cell notation: {}", e))
        })?;

        Ok(DecisionTarget::GridCell(cell))
    }

    fn parse_region_target(value: serde_json::Value) -> Result<DecisionTarget, DecisionError> {
        let region_target: LuaRegionTarget = serde_json::from_value(value).map_err(|e| {
            DecisionError::ScriptError(format!(
                "Invalid region target (expected {{from = \"A5\", to = \"C7\"}}): {}",
                e
            ))
        })?;

        let from_cell = GridCell::from_notation(&region_target.from).map_err(|e| {
            DecisionError::ScriptError(format!("Invalid 'from' cell: {}", e))
        })?;

        let to_cell = GridCell::from_notation(&region_target.to)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid 'to' cell: {}", e)))?;

        // Note: We create Region without grid validation
        // Validation happens later in the system if needed
        let region = Region::new_unchecked(crate::team::Team::A, from_cell, to_cell);

        Ok(DecisionTarget::Region(region))
    }

    fn parse_point_target(value: serde_json::Value) -> Result<DecisionTarget, DecisionError> {
        let point_target: LuaPointTarget = serde_json::from_value(value).map_err(|e| {
            DecisionError::ScriptError(format!(
                "Invalid point target (expected {{x = 10.5, z = 20.0}}): {}",
                e
            ))
        })?;

        let point = Point3D {
            x: Length::new::<meter>(point_target.x),
            y: Length::new::<meter>(point_target.y),
            z: Length::new::<meter>(point_target.z),
        };

        Ok(DecisionTarget::Point(point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_stop() {
        let json = json!({"action": "stop"});
        let decision: LuaDecision = serde_json::from_value(json).unwrap();

        match decision {
            LuaDecision::Stop => {}
            _ => panic!("Expected Stop"),
        }
    }

    #[test]
    fn test_deserialize_run_to_cell_simple() {
        let json = json!({
            "action": "run",
            "target_type": "cell",
            "target": "A5"
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let decision = lua_decision.into_decision().unwrap();

        match decision {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                assert_eq!(cell.col, 1);
                assert_eq!(cell.row, 5);
            }
            _ => panic!("Expected Run to GridCell"),
        }
    }

    #[test]
    fn test_deserialize_run_to_region() {
        let json = json!({
            "action": "run",
            "target_type": "region",
            "target": {
                "from": "A5",
                "to": "C7"
            }
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let decision = lua_decision.into_decision().unwrap();

        match decision {
            Decision::Run(DecisionTarget::Region(region)) => {
                assert_eq!(region.top_left.col, 1);
                assert_eq!(region.top_left.row, 5);
                assert_eq!(region.bottom_right.col, 3);
                assert_eq!(region.bottom_right.row, 7);
            }
            _ => panic!("Expected Run to Region"),
        }
    }

    #[test]
    fn test_deserialize_run_to_point() {
        let json = json!({
            "action": "run",
            "target_type": "point",
            "target": {
                "x": 25.5,
                "z": 30.0
            }
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let decision = lua_decision.into_decision().unwrap();

        match decision {
            Decision::Run(DecisionTarget::Point(point)) => {
                assert_eq!(point.x.get::<meter>(), 25.5);
                assert_eq!(point.y.get::<meter>(), 0.0); // default
                assert_eq!(point.z.get::<meter>(), 30.0);
            }
            _ => panic!("Expected Run to Point"),
        }
    }

    #[test]
    fn test_deserialize_run_to_point_with_y() {
        let json = json!({
            "action": "run",
            "target_type": "point",
            "target": {
                "x": 10.0,
                "y": 2.5,
                "z": 15.0
            }
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let decision = lua_decision.into_decision().unwrap();

        match decision {
            Decision::Run(DecisionTarget::Point(point)) => {
                assert_eq!(point.x.get::<meter>(), 10.0);
                assert_eq!(point.y.get::<meter>(), 2.5);
                assert_eq!(point.z.get::<meter>(), 15.0);
            }
            _ => panic!("Expected Run to Point"),
        }
    }

    #[test]
    fn test_invalid_cell_notation() {
        let json = json!({
            "action": "run",
            "target_type": "cell",
            "target": "1A"  // Invalid: digits before letters
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let result = lua_decision.into_decision();

        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("Invalid cell notation")),
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_unknown_target_type() {
        let json = json!({
            "action": "run",
            "target_type": "building",
            "target": "something"
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let result = lua_decision.into_decision();

        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("Unknown target_type")),
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_missing_region_fields() {
        let json = json!({
            "action": "run",
            "target_type": "region",
            "target": {
                "from": "A5"
                // Missing "to"
            }
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let result = lua_decision.into_decision();

        assert!(result.is_err());
    }

    #[test]
    fn test_missing_point_fields() {
        let json = json!({
            "action": "run",
            "target_type": "point",
            "target": {
                "x": 10.0
                // Missing "z"
            }
        });

        let lua_decision: LuaDecision = serde_json::from_value(json).unwrap();
        let result = lua_decision.into_decision();

        assert!(result.is_err());
    }
}
