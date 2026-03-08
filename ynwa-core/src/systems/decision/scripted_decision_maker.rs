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

        let field_width = config.field.width().get::<meter>();
        let field_length = config.field.length().get::<meter>();

        // static_data duplicated for each player to maintain orientation
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
                    "team": team_key,
                    "static_data": {
                        "zones": Self::zones_to_json_for_team(&config.field, p.team, field_width, field_length),
                        "field": {
                            "width": field_width,
                            "length": field_length,
                            "columns": config.field.grid_columns(),
                            "rows": config.field.grid_rows()
                        }
                    }
                })
            }).collect::<Vec<_>>()
        })
    }

    fn zones_to_json_for_team(
        field: &crate::field::Field,
        viewer_team: Team,
        field_width: f32,
        field_length: f32,
    ) -> serde_json::Value {
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
                        (
                            rect.min.x.get::<meter>(),
                            rect.max.x.get::<meter>(),
                            rect.min.z.get::<meter>(),
                            rect.max.z.get::<meter>(),
                        )
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
                        (
                            field_width - circle.center.x.get::<meter>(),
                            field_length - circle.center.z.get::<meter>(),
                        )
                    } else {
                        (
                            circle.center.x.get::<meter>(),
                            circle.center.z.get::<meter>(),
                        )
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
                        (
                            field_width - arc.center.x.get::<meter>(),
                            field_length - arc.center.z.get::<meter>(),
                        )
                    } else {
                        (arc.center.x.get::<meter>(), arc.center.z.get::<meter>())
                    };

                    // For Team B, angles need to be flipped too (reversed)
                    let (start_angle, end_angle) = if viewer_team == Team::B {
                        (
                            180.0 - arc.end_angle.get::<degree>(),
                            180.0 - arc.start_angle.get::<degree>(),
                        )
                    } else {
                        (
                            arc.start_angle.get::<degree>(),
                            arc.end_angle.get::<degree>(),
                        )
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
                        (
                            field_width - point.position.x.get::<meter>(),
                            field_length - point.position.z.get::<meter>(),
                        )
                    } else {
                        (
                            point.position.x.get::<meter>(),
                            point.position.z.get::<meter>(),
                        )
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
                // col → X (field width), row → Z (field length); square cells
                let cell_size = field_width / grid_dims.columns as f32;

                let min_x = (min_col - 1) as f32 * cell_size;
                let max_x = max_col as f32 * cell_size;
                let min_z = (min_row - 1) as f32 * cell_size;
                let max_z = max_row as f32 * cell_size;

                // Apply coordinate transformation if viewer is Team B
                let (final_min_x, final_max_x, final_min_z, final_max_z) = if player_team == Team::B
                {
                    let flipped_min_x = field_width - max_x;
                    let flipped_max_x = field_width - min_x;
                    let flipped_min_z = field_length - max_z;
                    let flipped_max_z = field_length - min_z;
                    (flipped_min_x, flipped_max_x, flipped_min_z, flipped_max_z)
                } else {
                    (min_x, max_x, min_z, max_z)
                };

                // display_notation: always in Team A (display) orientation, with team B's
                // own notation appended in parentheses when they differ, e.g. "R42 (M3)"
                let display_notation = if player_team == Team::B {
                    let team_notation = region
                        .flip_orientation(grid_dims)
                        .map(|r| r.to_grid_notation())
                        .unwrap_or_default();
                    format!("{} ({})", region.to_grid_notation(), team_notation)
                } else {
                    region.to_grid_notation()
                };

                let region_json = json!({
                    "min_x": final_min_x,
                    "max_x": final_max_x,
                    "min_z": final_min_z,
                    "max_z": final_max_z,
                    "display_notation": display_notation
                });

                (name.clone(), region_json)
            })
            .collect();

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
            }
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
        let context = Self::build_context(game, player_index)?;

        let decision_json = match &game.state().stage {
            crate::game::GameStage::Play => self
                .engine
                .make_decision(player_index, &context)
                .map_err(|e| DecisionError::RuntimeError(format!("Engine error: {}", e)))?,
            crate::game::GameStage::Setup(_) => {
                // DecisionSystem must never call make_decision during Setup — it should
                // skip scripted decisions entirely (see the is_setup guard in decision_system.rs).
                // Reaching this branch means a caller bug: fail loudly instead of hiding it.
                return Err(DecisionError::RuntimeError(
                    "ScriptedDecisionMaker::make_decision called during Setup stage; \
                     Setup decisions must be assigned by FootballGameManager, not by scripts"
                        .to_string(),
                ));
            }
            crate::game::GameStage::GameOver => {
                return Ok((Decision::Stop, None));
            }
        };

        decision_parser::parse_decision(&decision_json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef, REGION_START_POSITION};
    use crate::region::{GridCell};

    fn create_test_game_with_script(script: &str) -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = grid_dims.create_region(GridCell::new(10, 10).unwrap(), GridCell::new(11, 11).unwrap()).unwrap();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                script.to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
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
    fn test_region_boundaries_team_a() {
        // Test that Team A receives correct region boundaries.
        // Contract: col → X (field width=60m), row → Z (field length=100m).
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Region: columns 10-12 → X axis, rows 20-22 → Z axis
        let start_region = grid_dims.create_region(GridCell::new(10, 20).unwrap(), GridCell::new(12, 22).unwrap()).unwrap();

        let script = r#"
            function make_decision()
                return {action = "stop"}
            end
        "#;

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player A".to_string(),
                script.to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::new(config);

        // Build context and check region boundaries
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let regions = context["me"]["regions"].as_object().unwrap();
        let start_pos = regions.get("start").unwrap();

        // Expected boundaries for Team A (no transformation):
        // cell_size = 60 / 26 = 2.307...
        // columns 10-12 → X from (10-1)*cell_size to 12*cell_size
        // rows 20-22    → Z from (20-1)*cell_size to 22*cell_size
        let cell_size = 60.0 / 26.0_f64;

        let expected_min_x = 9.0 * cell_size;
        let expected_max_x = 12.0 * cell_size;
        let expected_min_z = 19.0 * cell_size;
        let expected_max_z = 22.0 * cell_size;

        assert!(
            (start_pos["min_x"].as_f64().unwrap() - expected_min_x).abs() < 0.01,
            "Team A min_x: expected {}, got {}",
            expected_min_x,
            start_pos["min_x"]
        );
        assert!(
            (start_pos["max_x"].as_f64().unwrap() - expected_max_x).abs() < 0.01,
            "Team A max_x: expected {}, got {}",
            expected_max_x,
            start_pos["max_x"]
        );
        assert!(
            (start_pos["min_z"].as_f64().unwrap() - expected_min_z).abs() < 0.01,
            "Team A min_z: expected {}, got {}",
            expected_min_z,
            start_pos["min_z"]
        );
        assert!(
            (start_pos["max_z"].as_f64().unwrap() - expected_max_z).abs() < 0.01,
            "Team A max_z: expected {}, got {}",
            expected_max_z,
            start_pos["max_z"]
        );
    }

    #[test]
    fn test_region_boundaries_team_b() {
        // Test that Team B receives correctly flipped region boundaries.
        // Contract: col → X (field width=60m), row → Z (field length=100m).
        let field = Field::from_meters(60.0, 100.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Region: columns 10-12 → X axis, rows 20-22 → Z axis
        let start_region = grid_dims.create_region(GridCell::new(10, 20).unwrap(), GridCell::new(12, 22).unwrap()).unwrap();

        let script = r#"
            function make_decision()
                return {action = "stop"}
            end
        "#;

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::B,
                1,
                "Test Player B".to_string(),
                script.to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::new(config);

        // Build context and check region boundaries
        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let regions = context["me"]["regions"].as_object().unwrap();
        let start_pos = regions.get("start").unwrap();

        // Pre-flip (Team A frame):
        // cell_size = 60 / 26
        // columns 10-12 → X: (9..12) * cell_size
        // rows 20-22    → Z: (19..22) * cell_size
        //
        // After flip (Team B): x' = field_width - x,  z' = field_length - z  (min/max swap)
        let cell_size = 60.0 / 26.0_f64;

        let orig_min_x = 9.0 * cell_size;
        let orig_max_x = 12.0 * cell_size;
        let orig_min_z = 19.0 * cell_size;
        let orig_max_z = 22.0 * cell_size;

        let expected_min_x = 60.0 - orig_max_x;
        let expected_max_x = 60.0 - orig_min_x;
        let expected_min_z = 100.0 - orig_max_z;
        let expected_max_z = 100.0 - orig_min_z;

        assert!(
            (start_pos["min_x"].as_f64().unwrap() - expected_min_x).abs() < 0.01,
            "Team B min_x: expected {}, got {}",
            expected_min_x,
            start_pos["min_x"]
        );
        assert!(
            (start_pos["max_x"].as_f64().unwrap() - expected_max_x).abs() < 0.01,
            "Team B max_x: expected {}, got {}",
            expected_max_x,
            start_pos["max_x"]
        );
        assert!(
            (start_pos["min_z"].as_f64().unwrap() - expected_min_z).abs() < 0.01,
            "Team B min_z: expected {}, got {}",
            expected_min_z,
            start_pos["min_z"]
        );
        assert!(
            (start_pos["max_z"].as_f64().unwrap() - expected_max_z).abs() < 0.01,
            "Team B max_z: expected {}, got {}",
            expected_max_z,
            start_pos["max_z"]
        );
    }

    /// build_context() region JSON must agree with Region::center() on which axis is X.
    /// Specifically, (min_x+max_x)/2 from the JSON must equal Region::center().x,
    /// and same for Z. This catches any divergence between the two computations.
    #[test]
    fn test_build_context_region_agrees_with_region_center() {
        use crate::field::Field;
        use uom::si::length::meter;

        let field = Field::from_meters(60.0, 101.538_46, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Use an off-centre region so X/Z confusion is clearly detectable
        let start_region = grid_dims
            .create_region(
                GridCell::from_notation("B3").unwrap(),
                GridCell::from_notation("D5").unwrap(),
            )
            .unwrap();

        // Ground truth from Region::center()
        let center = start_region.center(grid_dims, field.width().get::<meter>());
        let expected_cx = center.x.get::<meter>();
        let expected_cz = center.z.get::<meter>();

        let script = r#"function make_decision() return {action="stop"} end"#;
        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "tester".to_string(),
                script.to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };
        let game = Game::new(config);

        let ctx = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let sp = &ctx["me"]["regions"]["start"];

        let json_cx = ((sp["min_x"].as_f64().unwrap() + sp["max_x"].as_f64().unwrap()) / 2.0) as f32;
        let json_cz = ((sp["min_z"].as_f64().unwrap() + sp["max_z"].as_f64().unwrap()) / 2.0) as f32;

        assert!(
            (json_cx - expected_cx).abs() < 0.01,
            "build_context center_x={json_cx:.4} must equal Region::center().x={expected_cx:.4}"
        );
        assert!(
            (json_cz - expected_cz).abs() < 0.01,
            "build_context center_z={json_cz:.4} must equal Region::center().z={expected_cz:.4}"
        );
    }

    /// For Team B, the center of the flipped region from build_context must equal
    /// flip_point_orientation applied to Region::center().
    #[test]
    fn test_build_context_region_team_b_agrees_with_flipped_center() {
        use crate::field::Field;
        use crate::orientation::flip_point_orientation;
        use uom::si::length::meter;

        let field = Field::from_meters(60.0, 101.538_46, 26, 44);
        let grid_dims = field.grid_dimensions();
        let fw = field.width().get::<meter>();
        let fl = field.length().get::<meter>();

        let start_region = grid_dims
            .create_region(
                GridCell::from_notation("B3").unwrap(),
                GridCell::from_notation("D5").unwrap(),
            )
            .unwrap();

        // Ground truth: flip Region::center() the same way build_context does
        let center_a = start_region.center(grid_dims, fw);
        let center_b = flip_point_orientation(&center_a, fw, fl);
        let expected_cx = center_b.x.get::<meter>();
        let expected_cz = center_b.z.get::<meter>();

        let script = r#"function make_decision() return {action="stop"} end"#;
        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::B,
                1,
                "tester".to_string(),
                script.to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };
        let game = Game::new(config);

        let ctx = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let sp = &ctx["me"]["regions"]["start"];

        let json_cx = ((sp["min_x"].as_f64().unwrap() + sp["max_x"].as_f64().unwrap()) / 2.0) as f32;
        let json_cz = ((sp["min_z"].as_f64().unwrap() + sp["max_z"].as_f64().unwrap()) / 2.0) as f32;

        assert!(
            (json_cx - expected_cx).abs() < 0.01,
            "Team B build_context center_x={json_cx:.4} must equal flip(Region::center()).x={expected_cx:.4}"
        );
        assert!(
            (json_cz - expected_cz).abs() < 0.01,
            "Team B build_context center_z={json_cz:.4} must equal flip(Region::center()).z={expected_cz:.4}"
        );
    }

    #[test]
    fn test_zones_available_in_lua() {
        use crate::field::zones::{Rectangle, ZoneGeometry};
        use crate::field::{FieldBuilder, Zone};

        let field = FieldBuilder::from_meters(100.0, 60.0, 26, 44)
            .with_zone(Zone::new(
                "test_zone",
                Some(Team::A),
                ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 20.0, 30.0)),
            ))
            .build();

        let grid_dims = field.grid_dimensions();
        let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap()).unwrap();

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
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                script,
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::with_stage(config, crate::game::GameStage::Play);
        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());

        // Verify zone data is accessible from Lua via GAME_DATA
        let config_json = ScriptedDecisionMaker::build_config(&game);
        let engine = ynwa_decisions::DecisionEngine::new(
            &config_json,
            &game.config().scripting.core_preamble,
            &game.config().scripting.stdlib_preamble,
        )
        .unwrap();

        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let decision_json = engine.make_decision(0, &context).unwrap();

        assert_eq!(
            decision_json.get("has_zone").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            decision_json.get("zone_type").and_then(|v| v.as_str()),
            Some("rectangle")
        );
        assert_eq!(
            decision_json.get("zone_min_x").and_then(|v| v.as_f64()),
            Some(0.0)
        );
        assert_eq!(
            decision_json.get("zone_max_x").and_then(|v| v.as_f64()),
            Some(20.0)
        );
        assert_eq!(
            decision_json.get("zone_min_z").and_then(|v| v.as_f64()),
            Some(0.0)
        );
        assert_eq!(
            decision_json.get("zone_max_z").and_then(|v| v.as_f64()),
            Some(30.0)
        );
    }

    #[test]
    fn test_ball_owner_team_in_context() {
        use crate::team::Team;

        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = grid_dims.create_region(GridCell::new(13, 22).unwrap(), GridCell::new(13, 22).unwrap()).unwrap();

        // Script that returns ball owner_team
        let script = r#"
            function make_decision()
                return {
                    action = "stop",
                    owner_team = context.ball.owner_team
                }
            end
        "#
        .to_string();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                script,
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
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

        assert_eq!(
            owner_team_value,
            Some("None"),
            "Neutral ball should have owner_team='None'"
        );

        // Test 2: Team A owns ball
        game.state.ball_state.last_possessing_team = Some(Team::A);

        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let owner_team_value = context
            .get("ball")
            .and_then(|b| b.get("owner_team"))
            .and_then(|v| v.as_str());

        assert_eq!(
            owner_team_value,
            Some("A"),
            "Team A ball should have owner_team='A'"
        );

        // Test 3: Team B owns ball
        game.state.ball_state.last_possessing_team = Some(Team::B);

        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let owner_team_value = context
            .get("ball")
            .and_then(|b| b.get("owner_team"))
            .and_then(|v| v.as_str());

        assert_eq!(
            owner_team_value,
            Some("B"),
            "Team B ball should have owner_team='B'"
        );

        // Test 4: Verify Lua script can access owner_team
        game.state.ball_state.last_possessing_team = Some(Team::A);

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok(), "Decision should succeed");
    }

    #[test]
    fn test_game_data_field_dimensions_in_static_data() {
        // Deliberately unusual dimensions to avoid accidental match with any defaults
        let field = Field::from_meters(47.0, 83.0, 47, 83);
        let grid_dims = field.grid_dimensions();
        let start_region = grid_dims
            .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
            .unwrap();

        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Test Player".to_string(),
                "function make_decision() return {action='stop'} end".to_string(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::with_stage(config, crate::game::GameStage::Play);
        let config_json = ScriptedDecisionMaker::build_config(&game);

        let field_data = config_json
            .get("players")
            .and_then(|p| p.get(0))
            .and_then(|p| p.get("static_data"))
            .and_then(|sd| sd.get("field"))
            .expect("players[0].static_data.field must exist");

        assert!(
            (field_data["width"].as_f64().unwrap() - 47.0).abs() < 0.01,
            "GAME_DATA.field.width should be field width (X axis)"
        );
        assert!(
            (field_data["length"].as_f64().unwrap() - 83.0).abs() < 0.01,
            "GAME_DATA.field.length should be field length (Z axis)"
        );
    }

    #[test]
    fn test_game_data_field_accessible_from_lua() {
        let game = create_test_game_with_script(
            r#"
            function make_decision()
                return {
                    action = "stop",
                    field_width = GAME_DATA.field.width,
                    field_length = GAME_DATA.field.length
                }
            end
            "#,
        );

        let config_json = ScriptedDecisionMaker::build_config(&game);
        let engine = ynwa_decisions::DecisionEngine::new(
            &config_json,
            &game.config().scripting.core_preamble,
            &game.config().scripting.stdlib_preamble,
        )
        .unwrap();

        let context = ScriptedDecisionMaker::build_context(&game, 0).unwrap();
        let result = engine.make_decision(0, &context).unwrap();

        // Field was created with from_meters(100.0, 60.0, 26, 44) in create_test_game_with_script
        assert!(
            (result["field_width"].as_f64().unwrap() - 100.0).abs() < 0.01,
            "Lua GAME_DATA.field.width should match field width"
        );
        assert!(
            (result["field_length"].as_f64().unwrap() - 60.0).abs() < 0.01,
            "Lua GAME_DATA.field.length should match field length"
        );
    }
}
