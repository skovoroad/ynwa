/// Parser for converting JSON decisions from Lua scripts to domain types.
///
/// This module uses intermediate serde structures to deserialize JSON,
/// then converts them to domain types (Decision, DecisionTarget, etc.).
/// This approach is consistent with how config.rs handles deserialization.

use crate::field::zones::Point3D;
use crate::game::{Decision, DecisionTarget};
use crate::region::{GridCell, Region};
use crate::team::Team;
use serde::{Deserialize, Serialize};
use uom::si::f32::Length;
use uom::si::length::meter;

use super::DecisionError;

/// Intermediate structure for region target from JSON
#[derive(Debug, Deserialize, Serialize)]
struct LuaRegionTarget {
    from: String,
    to: String,
}

/// Intermediate structure for point target from JSON
#[derive(Debug, Deserialize, Serialize)]
struct LuaPointTarget {
    x: f32,
    z: f32,
    #[serde(default)]
    y: f32,
}

/// Parse JSON decision value into domain Decision type
pub fn parse_decision(value: &serde_json::Value) -> Result<Decision, DecisionError> {
    let action = value
        .get("action")
        .and_then(|a| a.as_str())
        .ok_or_else(|| DecisionError::RuntimeError("Missing 'action' field".to_string()))?;

    match action {
        "stop" => Ok(Decision::Stop),
        "run" => {
            let target_type = value
                .get("target_type")
                .and_then(|t| t.as_str())
                .ok_or_else(|| {
                    DecisionError::RuntimeError("Missing 'target_type' field".to_string())
                })?;

            let target = value.get("target").ok_or_else(|| {
                DecisionError::RuntimeError("Missing 'target' field".to_string())
            })?;

            let decision_target = match target_type {
                "cell" => parse_cell_target(target)?,
                "region" => parse_region_target(target)?,
                "point" => parse_point_as_target(target)?,
                _ => {
                    return Err(DecisionError::RuntimeError(format!(
                        "Unknown target_type: {}",
                        target_type
                    )))
                }
            };

            Ok(Decision::Run(decision_target))
        }
        "kick" => {
            let target = value.get("target").ok_or_else(|| {
                DecisionError::RuntimeError("Missing 'target' field for kick".to_string())
            })?;

            let kick_target = parse_point(target)?;
            Ok(Decision::Kick(kick_target))
        }
        _ => Err(DecisionError::RuntimeError(format!(
            "Unknown action: {}",
            action
        ))),
    }
}

/// Parse cell target from JSON string
fn parse_cell_target(value: &serde_json::Value) -> Result<DecisionTarget, DecisionError> {
    let cell_str = value.as_str().ok_or_else(|| {
        DecisionError::RuntimeError(format!(
            "Cell target must be a string (e.g., 'A5'), got: {:?}",
            value
        ))
    })?;

    let cell = GridCell::from_notation(cell_str).map_err(|e| {
        DecisionError::RuntimeError(format!("Invalid cell notation '{}': {}", cell_str, e))
    })?;

    Ok(DecisionTarget::GridCell(cell))
}

/// Parse region target from JSON structure
///
/// Note: Creates region without bounds validation using `new_unchecked`.
/// Validation will happen later in the decision system when the region is used.
fn parse_region_target(value: &serde_json::Value) -> Result<DecisionTarget, DecisionError> {
    // Deserialize using serde
    let region_target: LuaRegionTarget = serde_json::from_value(value.clone()).map_err(|e| {
        DecisionError::RuntimeError(format!(
            "Invalid region target format (expected {{from: 'A5', to: 'C7'}}): {}",
            e
        ))
    })?;

    let from_cell = GridCell::from_notation(&region_target.from).map_err(|e| {
        DecisionError::RuntimeError(format!(
            "Invalid 'from' cell '{}': {}",
            region_target.from, e
        ))
    })?;

    let to_cell = GridCell::from_notation(&region_target.to).map_err(|e| {
        DecisionError::RuntimeError(format!("Invalid 'to' cell '{}': {}", region_target.to, e))
    })?;

    // Use new_unchecked since we don't have grid dimensions here.
    // Validation will occur later when the region is actually used in the game.
    let region = Region::new_unchecked(Team::A, from_cell, to_cell);

    Ok(DecisionTarget::Region(region))
}

/// Parse point coordinates from JSON
pub fn parse_point(value: &serde_json::Value) -> Result<Point3D, DecisionError> {
    // Deserialize using serde
    let point_target: LuaPointTarget = serde_json::from_value(value.clone()).map_err(|e| {
        DecisionError::RuntimeError(format!(
            "Invalid point target format (expected {{x: 10.5, z: 20.0}}): {}",
            e
        ))
    })?;

    // Point from Lua is in player's orientation - no conversion here
    // Conversion happens in DecisionSystem via convert_decision_to_display_orientation
    let point = Point3D {
        x: Length::new::<meter>(point_target.x),
        y: Length::new::<meter>(point_target.y),
        z: Length::new::<meter>(point_target.z),
    };

    Ok(point)
}

/// Parse point and wrap it in DecisionTarget::Point
fn parse_point_as_target(value: &serde_json::Value) -> Result<DecisionTarget, DecisionError> {
    let point = parse_point(value)?;
    Ok(DecisionTarget::Point(point))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_stop_decision() {
        let json = json!({"action": "stop"});
        let decision = parse_decision(&json).unwrap();
        assert!(matches!(decision, Decision::Stop));
    }

    #[test]
    fn test_parse_run_to_cell_decision() {
        let json = json!({
            "action": "run",
            "target_type": "cell",
            "target": "B5"
        });
        let decision = parse_decision(&json).unwrap();
        match decision {
            Decision::Run(DecisionTarget::GridCell(cell)) => {
                assert_eq!(cell.row, 5);
                assert_eq!(cell.col, 2); // B = 2
            }
            _ => panic!("Expected Run with GridCell"),
        }
    }

    #[test]
    fn test_parse_run_to_point_decision() {
        let json = json!({
            "action": "run",
            "target_type": "point",
            "target": {"x": 50.0, "z": 30.0}
        });
        let decision = parse_decision(&json).unwrap();
        match decision {
            Decision::Run(DecisionTarget::Point(point)) => {
                assert!((point.x.get::<meter>() - 50.0).abs() < 0.01);
                assert!((point.z.get::<meter>() - 30.0).abs() < 0.01);
            }
            _ => panic!("Expected Run with Point"),
        }
    }

    #[test]
    fn test_parse_kick_decision() {
        let json = json!({
            "action": "kick",
            "target": {"x": 50.0, "z": 30.0}
        });
        let decision = parse_decision(&json).unwrap();
        match decision {
            Decision::Kick(point) => {
                assert!((point.x.get::<meter>() - 50.0).abs() < 0.01);
                assert!((point.z.get::<meter>() - 30.0).abs() < 0.01);
            }
            _ => panic!("Expected Kick"),
        }
    }

    #[test]
    fn test_parse_region_target() {
        let json = json!({
            "action": "run",
            "target_type": "region",
            "target": {"from": "A5", "to": "C7"}
        });
        let decision = parse_decision(&json).unwrap();
        match decision {
            Decision::Run(DecisionTarget::Region(_region)) => {
                // Success
            }
            _ => panic!("Expected Run with Region"),
        }
    }
}
