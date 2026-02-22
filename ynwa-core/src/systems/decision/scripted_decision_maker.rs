//! Adapter between ynwa-core domain types and ynwa-decisions JSON API.
//!
//! JSON contract (ynwa-core ↔ ynwa-decisions):
//! - `build_config()`: player scripts → JSON config
//! - `build_context()`: Game state → JSON context; Team B sees flipped coordinates
//! - `parse_json_decision()`: JSON → domain Decision; does NOT flip (flipping at system boundaries)
//!
//! Lua script return format:
//! - `{action="stop"}`
//! - `{action="run", target_type="point", target={x, z}}`
//! - `{action="run", target_type="cell", target="A5"}`
//! - `{action="run", target_type="region", target={from="A1", to="C3"}}`
//! - `{action="kick", target={x, z}}`
//! - optional `reason` field (string, logged on error/debug)

use crate::field::zones::Point3D;
use crate::game::{Decision, Game};
use crate::team::Team;
use serde_json::json;
use uom::si::length::meter;
use ynwa_decisions::DecisionEngine;

use super::decision_parser;
use super::{DecisionError, DecisionMaker};

/// DecisionMaker that executes user-defined Lua scripts via ynwa-decisions engine.
///
/// This implementation builds context from game state, executes player scripts,
/// and parses the resulting JSON decisions using the decision_parser module.
pub struct ScriptedDecisionMaker {
    engine: DecisionEngine,
}

impl ScriptedDecisionMaker {
    pub fn new(game: &Game) -> Result<Self, DecisionError> {
        let config = Self::build_config(game);
        let scripting = &game.config().scripting;

        // Create DecisionEngine with preambles from game config
        let engine = DecisionEngine::new(
            &config,
            &scripting.core_preamble,
            &scripting.stdlib_preamble,
        )
        .map_err(|e| {
            DecisionError::RuntimeError(format!("Failed to create DecisionEngine: {}", e))
        })?;

        Ok(Self { engine })
    }

    fn build_config(game: &Game) -> serde_json::Value {
        let config = game.config();
        let scripting = &config.scripting;

        let zones_json = Self::zones_to_json(&config.field);

        json!({
            "team_preambles": {
                "team_a": scripting.team_a_preamble,
                "team_b": scripting.team_b_preamble
            },
            "players": config.players.iter().map(|p| {
                let team_key = match p.team {
                    Team::A => "team_a",
                    Team::B => "team_b",
                };
                json!({
                    "script": p.script,
                    "team": team_key
                })
            }).collect::<Vec<_>>(),
            "static_data": {
                "zones": zones_json
            }
        })
    }

    fn zones_to_json(field: &crate::field::Field) -> serde_json::Value {
        use crate::field::zones::ZoneGeometry;
        use serde_json::json;

        let mut zones_map = serde_json::Map::new();

        for ((name, team), zone) in field.zones() {
            let team_suffix = match team {
                Some(Team::A) => "_a",
                Some(Team::B) => "_b",
                None => "",
            };
            let key = format!("{}{}", name, team_suffix);

            let geometry_json = match &zone.geometry {
                ZoneGeometry::Rectangle(rect) => {
                    json!({
                        "type": "rectangle",
                        "min_x": rect.min.x.get::<meter>(),
                        "max_x": rect.max.x.get::<meter>(),
                        "min_z": rect.min.z.get::<meter>(),
                        "max_z": rect.max.z.get::<meter>()
                    })
                }
                ZoneGeometry::Circle(circle) => {
                    json!({
                        "type": "circle",
                        "center_x": circle.center.x.get::<meter>(),
                        "center_z": circle.center.z.get::<meter>(),
                        "radius": circle.radius.get::<meter>()
                    })
                }
                ZoneGeometry::Arc(arc) => {
                    use uom::si::angle::degree;
                    json!({
                        "type": "arc",
                        "center_x": arc.center.x.get::<meter>(),
                        "center_z": arc.center.z.get::<meter>(),
                        "radius": arc.radius.get::<meter>(),
                        "start_angle": arc.start_angle.get::<degree>(),
                        "end_angle": arc.end_angle.get::<degree>()
                    })
                }
                ZoneGeometry::Point(point) => {
                    json!({
                        "type": "point",
                        "x": point.position.x.get::<meter>(),
                        "z": point.position.z.get::<meter>(),
                        "tolerance": 0.5
                    })
                }
            };

            zones_map.insert(key, geometry_json);
        }

        serde_json::Value::Object(zones_map)
    }

    fn zones_to_json_for_team(field: &crate::field::Field, viewer_team: Team, field_width: f32, field_length: f32) -> serde_json::Value {
        use crate::field::zones::ZoneGeometry;
        use serde_json::json;

        let mut zones_map = serde_json::Map::new();

        for ((name, team), zone) in field.zones() {
            let team_suffix = match team {
                Some(Team::A) => "_a",
                Some(Team::B) => "_b",
                None => "",
            };
            let key = format!("{}{}", name, team_suffix);

            let geometry_json = match &zone.geometry {
                ZoneGeometry::Rectangle(rect) => {
                    // Transform coordinates for Team B
                    let (min_x, max_x, min_z, max_z) = if viewer_team == Team::B {
                        let flipped_min_x = field_width - rect.max.x.get::<meter>();
                        let flipped_max_x = field_width - rect.min.x.get::<meter>();
                        let flipped_min_z = field_length - rect.max.z.get::<meter>();
                        let flipped_max_z = field_length - rect.min.z.get::<meter>();
                        (flipped_min_x, flipped_max_x, flipped_min_z, flipped_max_z)
                    } else {
                        (rect.min.x.get::<meter>(), rect.max.x.get::<meter>(), 
                         rect.min.z.get::<meter>(), rect.max.z.get::<meter>())
                    };
                    
                    json!({
                        "type": "rectangle",
                        "min_x": min_x,
                        "max_x": max_x,
                        "min_z": min_z,
                        "max_z": max_z
                    })
                }
                ZoneGeometry::Circle(circle) => {
                    let (center_x, center_z) = if viewer_team == Team::B {
                        (field_width - circle.center.x.get::<meter>(),
                         field_length - circle.center.z.get::<meter>())
                    } else {
                        (circle.center.x.get::<meter>(), circle.center.z.get::<meter>())
                    };
                    
                    json!({
                        "type": "circle",
                        "center_x": center_x,
                        "center_z": center_z,
                        "radius": circle.radius.get::<meter>()
                    })
                }
                ZoneGeometry::Arc(arc) => {
                    use uom::si::angle::degree;
                    let (center_x, center_z) = if viewer_team == Team::B {
                        (field_width - arc.center.x.get::<meter>(),
                         field_length - arc.center.z.get::<meter>())
                    } else {
                        (arc.center.x.get::<meter>(), arc.center.z.get::<meter>())
                    };
                    
                    // For Team B, angles need to be flipped too (reversed)
                    let (start_angle, end_angle) = if viewer_team == Team::B {
                        (180.0 - arc.end_angle.get::<degree>(), 180.0 - arc.start_angle.get::<degree>())
                    } else {
                        (arc.start_angle.get::<degree>(), arc.end_angle.get::<degree>())
                    };
                    
                    json!({
                        "type": "arc",
                        "center_x": center_x,
                        "center_z": center_z,
                        "radius": arc.radius.get::<meter>(),
                        "start_angle": start_angle,
                        "end_angle": end_angle
                    })
                }
                ZoneGeometry::Point(point) => {
                    let (x, z) = if viewer_team == Team::B {
                        (field_width - point.position.x.get::<meter>(),
                         field_length - point.position.z.get::<meter>())
                    } else {
                        (point.position.x.get::<meter>(), point.position.z.get::<meter>())
                    };
                    
                    json!({
                        "type": "point",
                        "x": x,
                        "z": z,
                        "tolerance": 0.5
                    })
                }
            };

            zones_map.insert(key, geometry_json);
        }

        serde_json::Value::Object(zones_map)
    }

    fn build_context(game: &Game, player_index: usize) -> Result<serde_json::Value, DecisionError> {
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

        let field_width = config.field.width().get::<meter>();
        let field_length = config.field.length().get::<meter>();
        let grid_dims = config.field.grid_dimensions();

        // Build regions map for the current player with exact boundaries
        let regions_json: serde_json::Map<String, serde_json::Value> = player_def
            .regions
            .iter()
            .map(|(name, region)| {
                // Get exact region boundaries from grid cells
                let (min_col, max_col) = (region.top_left.col, region.bottom_right.col);
                let (min_row, max_row) = (region.top_left.row, region.bottom_right.row);
                
                // Convert grid coordinates to meters
                // Note: Both X and Z use cell_width (square cells) as per Region::center() logic
                let cell_width = field_width / grid_dims.columns as f32;
                
                let min_z = (min_col - 1) as f32 * cell_width;
                let max_z = max_col as f32 * cell_width;
                let min_x = (min_row - 1) as f32 * cell_width;
                let max_x = max_row as f32 * cell_width;
                
                // Apply coordinate transformation if viewer is Team B
                let (final_min_x, final_max_x, final_min_z, final_max_z) = if player_team == Team::B {
                    // Use same transformation as flip_point_orientation: x' = width - x, z' = length - z
                    // (This matches the parameter order used throughout the codebase)
                    // When we flip, min and max swap positions
                    let flipped_min_x = field_width - max_x;
                    let flipped_max_x = field_width - min_x;
                    let flipped_min_z = field_length - max_z;
                    let flipped_max_z = field_length - min_z;
                    (flipped_min_x, flipped_max_x, flipped_min_z, flipped_max_z)
                } else {
                    (min_x, max_x, min_z, max_z)
                };
                
                let region_json = json!({
                    "min_x": final_min_x,
                    "max_x": final_max_x,
                    "min_z": final_min_z,
                    "max_z": final_max_z
                });
                
                (name.clone(), region_json)
            })
            .collect();

        // Build zones with coordinate transformation for Team B
        let zones_json = Self::zones_to_json_for_team(&config.field, player_team, field_width, field_length);
        
        // Build context (same as ContextBuilder, but inline)
        let context = json!({
            "me": {
                "team": format!("{:?}", player_team),
                "number": player_def.number,
                "index": player_index,
                "position": Self::position_to_json(&player_state.position, player_team, field_width, field_length),
                "regions": regions_json
            },
            "teammates": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_team,
                player_team,
                field_width,
                field_length
            ),
            "opponents": Self::build_team_positions(
                &config.players,
                &state.player_states,
                player_team.opposite(),
                player_team,
                field_width,
                field_length
            ),
            "ball": {
                "position": Self::position_to_json(&state.ball_state.position, player_team, field_width, field_length),
                "owner_index": state.ball_state.possessed_by,
                "owner_team": match state.ball_state.last_possessing_team {
                    Some(team) => format!("{:?}", team),
                    None => "None".to_string(),
                }
            },
            "game": {
                "elapsed_time": state.elapsed_time
            },
            "zones": zones_json
        });

        Ok(context)
    }

    fn position_to_json(
        pos: &Point3D,
        viewer_team: Team,
        field_width: f32,
        field_length: f32,
    ) -> serde_json::Value {
        use crate::orientation::flip_point_orientation;

        let transformed_pos = if viewer_team == Team::B {
            // Use same parameter order as elsewhere in codebase
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
                .enumerate()
                .filter(|(_, (def, _))| def.team == team)
                .map(|(idx, (def, state))| {
                    json!({
                        "index": idx,
                        "number": def.number,
                        "position": Self::position_to_json(&state.position, viewer_team, field_width, field_length)
                    })
                })
                .collect::<Vec<_>>()
        )
    }
}

impl DecisionMaker for ScriptedDecisionMaker {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        // Build context
        let context = Self::build_context(game, player_index)?;

        // Choose function based on game stage
        let decision_json = match &game.state().stage {
            crate::game::GameStage::Play => self
                .engine
                .make_decision(player_index, &context)
                .map_err(|e| DecisionError::RuntimeError(format!("Engine error: {}", e)))?,
            crate::game::GameStage::Setup(_reason) => {
                self.engine
                    .prepare(player_index, &context)
                    .map_err(|e| DecisionError::RuntimeError(format!("Engine error: {}", e)))?
            }
            crate::game::GameStage::GameOver => {
                // During game over, return Stop decision with no reason
                return Ok((Decision::Stop, None));
            }
        };

        // Parse JSON decision to Decision type using decision_parser (returns tuple)
        let player_team = game.config().players[player_index].team;
        decision_parser::parse_decision(&decision_json, player_team)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
    use crate::region::{GridCell, Region};

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
            players: vec![PlayerDef::new(Team::A, 1, "Test Player".to_string(), script.to_string(), start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::with_stage(config, crate::game::GameStage::Play)
    }

    #[test]
    fn test_json_decision_maker_stop() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            "#,
        );

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        let (decision_value, _) = decision.unwrap();
        assert!(matches!(decision_value, Decision::Stop));
    }

    #[test]
    fn test_json_decision_maker_kick() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {
                    action = "kick",
                    target = {x = 50.0, z = 30.0}
                }
            end
            "#,
        );

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        let (decision_value, _) = decision.unwrap();
        match decision_value {
            Decision::Kick(point) => {
                use uom::si::length::meter;
                assert!((point.x.get::<meter>() - 50.0).abs() < 0.01);
                assert!((point.z.get::<meter>() - 30.0).abs() < 0.01);
            }
            _ => panic!("Expected Kick decision"),
        }
    }

    #[test]
    fn test_json_decision_maker_receives_context() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                local my_number = context.me.number
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

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        let (decision_value, _) = decision.unwrap();
        assert!(matches!(decision_value, Decision::Stop));
    }

    #[test]
    fn test_prepare_function_in_play_stage() {
        let mut game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            
            function prepare(reason)
                return {action = "run", target_type = "cell", target = "B2"}
            end
            "#,
        );

        game.state.stage = crate::game::GameStage::Play;
        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        // In Play stage, should call make_decision, not prepare
        let (decision_value, _) = decision.unwrap();
        assert!(matches!(decision_value, Decision::Stop));
    }

    #[test]
    fn test_prepare_function_in_setup_stage() {
        let mut game = create_test_game_with_script(
            r#"
            function make_decision()
                return {action = "stop"}
            end
            
            function prepare(reason)
                return {action = "run", target_type = "cell", target = "B2"}
            end
            "#,
        );

        game.state.stage = crate::game::GameStage::Setup("kickoff".to_string());
        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        // In Setup stage, should call prepare
        let (decision_value, _) = decision.unwrap();
        match decision_value {
            Decision::Run(target) => match target {
                crate::game::DecisionTarget::GridCell(cell) => {
                    assert_eq!(cell.col, 2);
                    assert_eq!(cell.row, 2);
                }
                _ => panic!("Expected GridCell target"),
            },
            _ => panic!("Expected Run decision"),
        }
    }

    #[test]
    fn test_prepare_function_default_from_stdlib() {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 10).unwrap(),
            GridCell::new(11, 11).unwrap(),
            grid_dims,
        )
        .unwrap();

        let stdlib_preamble = r#"
            function prepare(reason)
                return {action = "stop"}
            end
        "#;

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::A, 1, "Test Player".to_string(), "function make_decision() return {action = 'run', target_type = 'cell', target = 'A1'} end".to_string(),
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig {
                core_preamble: String::new(),
                stdlib_preamble: stdlib_preamble.to_string(),
                team_a_preamble: String::new(),
                team_b_preamble: String::new(),
            },
        };

        let mut game = Game::new(config);
        game.state.stage = crate::game::GameStage::Setup("kickoff".to_string());

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        // Should use default prepare from stdlib
        let (decision_value, _reason) = decision.unwrap();
        assert!(matches!(decision_value, Decision::Stop));
    }

    #[test]
    fn test_region_boundaries_team_a() {
        // Test that Team A receives correct region boundaries
        // Football field: width=60m (Z axis), length=100m (X axis)
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Create a region: columns 10-12 (Z axis), rows 20-22 (X axis)
        let start_region = Region::new(
            Team::A,
            GridCell::new(10, 20).unwrap(),
            GridCell::new(12, 22).unwrap(),
            grid_dims,
        )
        .unwrap();

        let script = r#"
            function make_decision()
                return {action = "stop"}
            end
        "#;

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::A, 1, "Test Player A".to_string(), script.to_string(), start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::new(config);
        
        // Build context and check region boundaries
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let regions = context["me"]["regions"].as_object().unwrap();
        let start_pos = regions.get("start position").unwrap();
        
        // Expected boundaries for Team A (no transformation):
        // cell_width = 60 / 26 = 2.307...
        // columns 10-12: Z from (10-1)*2.307 to 12*2.307
        // rows 20-22: X from (20-1)*2.307 to 22*2.307
        let cell_width = 60.0 / 26.0;
        
        let expected_min_z = 9.0 * cell_width;
        let expected_max_z = 12.0 * cell_width;
        let expected_min_x = 19.0 * cell_width;
        let expected_max_x = 22.0 * cell_width;
        
        assert!((start_pos["min_z"].as_f64().unwrap() - expected_min_z as f64).abs() < 0.01,
            "Team A min_z: expected {}, got {}", expected_min_z, start_pos["min_z"]);
        assert!((start_pos["max_z"].as_f64().unwrap() - expected_max_z as f64).abs() < 0.01,
            "Team A max_z: expected {}, got {}", expected_max_z, start_pos["max_z"]);
        assert!((start_pos["min_x"].as_f64().unwrap() - expected_min_x as f64).abs() < 0.01,
            "Team A min_x: expected {}, got {}", expected_min_x, start_pos["min_x"]);
        assert!((start_pos["max_x"].as_f64().unwrap() - expected_max_x as f64).abs() < 0.01,
            "Team A max_x: expected {}, got {}", expected_max_x, start_pos["max_x"]);
    }

    #[test]
    fn test_region_boundaries_team_b() {
        // Test that Team B receives correctly flipped region boundaries
        // Football field: width=60m (Z axis), length=100m (X axis)
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Create a region: columns 10-12 (Z axis), rows 20-22 (X axis)
        let start_region = Region::new(
            Team::B,
            GridCell::new(10, 20).unwrap(),
            GridCell::new(12, 22).unwrap(),
            grid_dims,
        )
        .unwrap();

        let script = r#"
            function make_decision()
                return {action = "stop"}
            end
        "#;

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::B, 1, "Test Player B".to_string(), script.to_string(), start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::new(config);
        
        // Build context and check region boundaries
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let regions = context["me"]["regions"].as_object().unwrap();
        let start_pos = regions.get("start position").unwrap();
        
        // Expected boundaries for Team B (with flip transformation):
        // cell_width = 60 / 26 = 2.307...
        // Original: columns 10-12 → Z from 9*2.307 to 12*2.307
        //           rows 20-22 → X from 19*2.307 to 22*2.307
        // Flipped using flip_point_orientation logic: x' = width - x, z' = length - z
        //          min/max swap after flip
        let cell_width = 60.0 / 26.0;
        
        let orig_min_z = 9.0 * cell_width;
        let orig_max_z = 12.0 * cell_width;
        let orig_min_x = 19.0 * cell_width;
        let orig_max_x = 22.0 * cell_width;
        
        // After flip: x' = 60 - x (field_width - x), z' = 100 - z (field_length - z)
        let expected_min_x = 60.0 - orig_max_x;
        let expected_max_x = 60.0 - orig_min_x;
        let expected_min_z = 100.0 - orig_max_z;
        let expected_max_z = 100.0 - orig_min_z;
        
        assert!((start_pos["min_x"].as_f64().unwrap() - expected_min_x as f64).abs() < 0.01,
            "Team B min_x: expected {}, got {}", expected_min_x, start_pos["min_x"]);
        assert!((start_pos["max_x"].as_f64().unwrap() - expected_max_x as f64).abs() < 0.01,
            "Team B max_x: expected {}, got {}", expected_max_x, start_pos["max_x"]);
        assert!((start_pos["min_z"].as_f64().unwrap() - expected_min_z as f64).abs() < 0.01,
            "Team B min_z: expected {}, got {}", expected_min_z, start_pos["min_z"]);
        assert!((start_pos["max_z"].as_f64().unwrap() - expected_max_z as f64).abs() < 0.01,
            "Team B max_z: expected {}, got {}", expected_max_z, start_pos["max_z"]);
    }

    #[test]
    fn test_zones_available_in_lua() {
        use crate::field::{FieldBuilder, Zone};
        use crate::field::zones::{Rectangle, ZoneGeometry};

        let field = FieldBuilder::from_meters(100.0, 60.0, 26, 44)
            .with_zone(Zone::new(
                "test_zone",
                Some(Team::A),
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 20.0, 30.0)),
            ))
            .build();
        
        let grid_dims = field.grid_dimensions();
        let start_region = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(2, 2).unwrap(),
            grid_dims,
        )
        .unwrap();

        let test_logic = r#"
            local zone = GAME_DATA.zones.test_zone_a
            if zone and zone.type == "rectangle" then
                return {
                    action = "stop",
                    has_zone = true,
                    zone_type = zone.type,
                    zone_min_x = zone.min_x,
                    zone_max_x = zone.max_x,
                    zone_min_z = zone.min_z,
                    zone_max_z = zone.max_z
                }
            end
            return {action = "stop", has_zone = false}
        "#;

        let script = format!("function make_decision() {} end", test_logic);

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::A, 1, "Test Player".to_string(), script, start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::with_stage(config, crate::game::GameStage::Play);
        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        
        let config_json = ScriptedDecisionMaker::build_config(&game);
        let engine = ynwa_decisions::DecisionEngine::new(
            &config_json,
            &game.config().scripting.core_preamble,
            &game.config().scripting.stdlib_preamble,
        )
        .unwrap();
        
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let decision_json = engine.make_decision(0, &context).unwrap();

        assert_eq!(decision_json.get("has_zone").and_then(|v| v.as_bool()), Some(true));
        assert_eq!(decision_json.get("zone_type").and_then(|v| v.as_str()), Some("rectangle"));
        assert_eq!(decision_json.get("zone_min_x").and_then(|v| v.as_f64()), Some(0.0));
        assert_eq!(decision_json.get("zone_max_x").and_then(|v| v.as_f64()), Some(20.0));
        assert_eq!(decision_json.get("zone_min_z").and_then(|v| v.as_f64()), Some(0.0));
        assert_eq!(decision_json.get("zone_max_z").and_then(|v| v.as_f64()), Some(30.0));
    }

    #[test]
    fn test_ball_owner_team_in_context() {
        use crate::team::Team;
        
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(13, 22).unwrap(),
            GridCell::new(13, 22).unwrap(),
            grid_dims,
        )
        .unwrap();

        // Script that returns ball owner_team
        let script = r#"
            function make_decision()
                return {
                    action = "stop",
                    owner_team = context.ball.owner_team
                }
            end
        "#.to_string();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::A, 1, "Test Player".to_string(), script, start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let mut game = Game::with_stage(config, crate::game::GameStage::Play);

        // Test 1: Neutral ball (None)
        game.state.ball_state.last_possessing_team = None;
        
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let owner_team_value = context
            .get("ball")
            .and_then(|b| b.get("owner_team"))
            .and_then(|v| v.as_str());
        
        assert_eq!(owner_team_value, Some("None"), "Neutral ball should have owner_team='None'");

        // Test 2: Team A owns ball
        game.state.ball_state.last_possessing_team = Some(Team::A);
        
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let owner_team_value = context
            .get("ball")
            .and_then(|b| b.get("owner_team"))
            .and_then(|v| v.as_str());
        
        assert_eq!(owner_team_value, Some("A"), "Team A ball should have owner_team='A'");

        // Test 3: Team B owns ball
        game.state.ball_state.last_possessing_team = Some(Team::B);
        
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let owner_team_value = context
            .get("ball")
            .and_then(|b| b.get("owner_team"))
            .and_then(|v| v.as_str());
        
        assert_eq!(owner_team_value, Some("B"), "Team B ball should have owner_team='B'");

        // Test 4: Verify Lua script can access owner_team
        game.state.ball_state.last_possessing_team = Some(Team::A);
        
        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);
        
        assert!(decision.is_ok(), "Decision should succeed");
    }
}
