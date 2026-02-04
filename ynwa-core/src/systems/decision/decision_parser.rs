use crate::field::zones::Point3D;
use crate::game::{Decision, DecisionTarget};
use crate::region::{GridCell, Region};
use serde_json::Value;
use uom::si::f32::Length;
use uom::si::length::meter;

use super::DecisionError;

/// Parses Lua script result (JSON) into Decision
pub struct DecisionParser;

impl DecisionParser {
    pub fn parse(value: Value) -> Result<Decision, DecisionError> {
        let obj = value
            .as_object()
            .ok_or_else(|| DecisionError::ScriptError("Result must be a table".to_string()))?;

        let action = obj
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DecisionError::ScriptError("Missing 'action' field".to_string()))?;

        match action {
            "run" => Self::parse_run_decision(obj),
            "stop" => Ok(Decision::Stop),
            _ => Err(DecisionError::ScriptError(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }

    fn parse_run_decision(
        obj: &serde_json::Map<String, Value>,
    ) -> Result<Decision, DecisionError> {
        let target_type = obj
            .get("target_type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                DecisionError::ScriptError("Missing 'target_type' field".to_string())
            })?;

        let target_value = obj
            .get("target")
            .ok_or_else(|| DecisionError::ScriptError("Missing 'target' field".to_string()))?;

        let target = match target_type {
            "cell" => Self::parse_cell_target(target_value)?,
            "region" => Self::parse_region_target(target_value)?,
            "point" => Self::parse_point_target(target_value)?,
            _ => {
                return Err(DecisionError::ScriptError(format!(
                    "Unknown target_type: {}",
                    target_type
                )))
            }
        };

        Ok(Decision::Run(target))
    }

    fn parse_cell_target(value: &Value) -> Result<DecisionTarget, DecisionError> {
        let cell_str = value.as_str().ok_or_else(|| {
            DecisionError::ScriptError("target must be string for 'cell' type".to_string())
        })?;

        let cell = GridCell::from_notation(cell_str)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid cell notation: {}", e)))?;

        Ok(DecisionTarget::GridCell(cell))
    }

    fn parse_region_target(value: &Value) -> Result<DecisionTarget, DecisionError> {
        let obj = value.as_object().ok_or_else(|| {
            DecisionError::ScriptError("target must be table with 'from' and 'to' for 'region' type".to_string())
        })?;

        let from_str = obj
            .get("from")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DecisionError::ScriptError("Missing 'from' field".to_string()))?;

        let to_str = obj
            .get("to")
            .and_then(|v| v.as_str())
            .ok_or_else(|| DecisionError::ScriptError("Missing 'to' field".to_string()))?;

        let from_cell = GridCell::from_notation(from_str)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid 'from' cell: {}", e)))?;

        let to_cell = GridCell::from_notation(to_str)
            .map_err(|e| DecisionError::ScriptError(format!("Invalid 'to' cell: {}", e)))?;

        // Note: We need grid_dimensions to create Region, but we don't have them here
        // For now, we'll create Region without validation
        // TODO: Pass grid_dimensions or validate in LuaDecisionMaker
        let region = Region::new_unchecked(crate::team::Team::A, from_cell, to_cell);

        Ok(DecisionTarget::Region(region))
    }

    fn parse_point_target(value: &Value) -> Result<DecisionTarget, DecisionError> {
        let obj = value.as_object().ok_or_else(|| {
            DecisionError::ScriptError(
                "target must be table with 'x' and 'z' for 'point' type".to_string(),
            )
        })?;

        let x = obj
            .get("x")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| DecisionError::ScriptError("Missing or invalid 'x' field".to_string()))?
            as f32;

        let z = obj
            .get("z")
            .and_then(|v| v.as_f64())
            .ok_or_else(|| DecisionError::ScriptError("Missing or invalid 'z' field".to_string()))?
            as f32;

        // y is optional, default to 0
        let y = obj
            .get("y")
            .and_then(|v| v.as_f64())
            .map(|v| v as f32)
            .unwrap_or(0.0);

        let point = Point3D {
            x: Length::new::<meter>(x),
            y: Length::new::<meter>(y),
            z: Length::new::<meter>(z),
        };

        Ok(DecisionTarget::Point(point))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_stop_decision() {
        let value = json!({
            "action": "stop"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), Decision::Stop));
    }

    #[test]
    fn test_parse_run_to_cell() {
        let value = json!({
            "action": "run",
            "target_type": "cell",
            "target": "A5"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_ok());

        match result.unwrap() {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                assert_eq!(cell.col, 1);
                assert_eq!(cell.row, 5);
            }
            _ => panic!("Expected Run to GridCell"),
        }
    }

    #[test]
    fn test_parse_run_to_region() {
        let value = json!({
            "action": "run",
            "target_type": "region",
            "target": {
                "from": "A5",
                "to": "C7"
            }
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_ok());

        match result.unwrap() {
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
    fn test_parse_run_to_point() {
        let value = json!({
            "action": "run",
            "target_type": "point",
            "target": {
                "x": 25.5,
                "z": 30.0
            }
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_ok());

        match result.unwrap() {
            Decision::Run(DecisionTarget::Point(point)) => {
                assert_eq!(point.x.get::<meter>(), 25.5);
                assert_eq!(point.y.get::<meter>(), 0.0);
                assert_eq!(point.z.get::<meter>(), 30.0);
            }
            _ => panic!("Expected Run to Point"),
        }
    }

    #[test]
    fn test_parse_run_to_point_with_y() {
        let value = json!({
            "action": "run",
            "target_type": "point",
            "target": {
                "x": 10.0,
                "y": 2.5,
                "z": 15.0
            }
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_ok());

        match result.unwrap() {
            Decision::Run(DecisionTarget::Point(point)) => {
                assert_eq!(point.x.get::<meter>(), 10.0);
                assert_eq!(point.y.get::<meter>(), 2.5);
                assert_eq!(point.z.get::<meter>(), 15.0);
            }
            _ => panic!("Expected Run to Point"),
        }
    }

    #[test]
    fn test_parse_missing_action() {
        let value = json!({
            "target": "A5"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecisionError::ScriptError(_)));
    }

    #[test]
    fn test_parse_unknown_action() {
        let value = json!({
            "action": "jump"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("Unknown action")),
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_parse_missing_target_type() {
        let value = json!({
            "action": "run",
            "target": "A5"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("target_type")),
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_parse_missing_target() {
        let value = json!({
            "action": "run",
            "target_type": "cell"
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("target")),
            _ => panic!("Expected ScriptError"),
        }
    }

    #[test]
    fn test_parse_invalid_cell_notation() {
        let value = json!({
            "action": "run",
            "target_type": "cell",
            "target": "1A"  // Digits before letters is invalid
        });

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DecisionError::ScriptError(_)));
    }

    #[test]
    fn test_parse_not_a_table() {
        let value = json!("just a string");

        let result = DecisionParser::parse(value);
        assert!(result.is_err());
        match result.unwrap_err() {
            DecisionError::ScriptError(msg) => assert!(msg.contains("must be a table")),
            _ => panic!("Expected ScriptError"),
        }
    }
}
