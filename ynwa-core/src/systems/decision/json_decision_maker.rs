use crate::field::zones::Point3D;
use crate::game::{Decision, DecisionTarget, Game};
use crate::region::{GridCell, Region};
use crate::team::Team;
use serde_json::json;
use uom::si::length::meter;
use ynwa_decisions::DecisionEngine;

use super::{DecisionError, DecisionMaker};

/// DecisionMaker that uses ynwa-decisions library through JSON
pub struct JsonDecisionMaker {
    engine: DecisionEngine,
}

impl JsonDecisionMaker {
    pub fn new(game: &Game) -> Result<Self, DecisionError> {
        // Build JSON config for DecisionEngine
        let config = Self::build_config(game);

        // Create DecisionEngine
        let engine = DecisionEngine::new(&config).map_err(|e| {
            DecisionError::RuntimeError(format!("Failed to create DecisionEngine: {}", e))
        })?;

        Ok(Self { engine })
    }

    fn build_config(game: &Game) -> serde_json::Value {
        let config = game.config();
        json!({
            "players": config.players.iter().map(|p| {
                json!({"script": p.script})
            }).collect::<Vec<_>>()
        })
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

        // Build context (same as ContextBuilder, but inline)
        let context = json!({
            "me": {
                "team": format!("{:?}", player_team),
                "number": player_def.number,
                "position": Self::position_to_json(&player_state.position, player_team, field_width, field_length)
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
                "position": Self::position_to_json(&state.ball_state.position, player_team, field_width, field_length)
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

    fn parse_json_decision(
        value: &serde_json::Value,
        game: &Game,
        player_team: Team,
    ) -> Result<Decision, DecisionError> {
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
                    "cell" => Self::parse_cell_target(target)?,
                    "region" => Self::parse_region_target(target, game)?,
                    "point" => Self::parse_point_target(target, game, player_team)?,
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

                let kick_target = Self::parse_point(target, game, player_team)?;
                Ok(Decision::Kick(kick_target))
            }
            _ => Err(DecisionError::RuntimeError(format!(
                "Unknown action: {}",
                action
            ))),
        }
    }

    fn parse_cell_target(value: &serde_json::Value) -> Result<DecisionTarget, DecisionError> {
        let cell_str = value.as_str().ok_or_else(|| {
            DecisionError::RuntimeError("Cell target must be a string".to_string())
        })?;

        let cell = GridCell::from_notation(cell_str)
            .map_err(|e| DecisionError::RuntimeError(format!("Invalid cell notation: {}", e)))?;

        Ok(DecisionTarget::GridCell(cell))
    }

    fn parse_region_target(
        value: &serde_json::Value,
        game: &Game,
    ) -> Result<DecisionTarget, DecisionError> {
        let from_str = value
            .get("from")
            .and_then(|f| f.as_str())
            .ok_or_else(|| {
                DecisionError::RuntimeError("Region target missing 'from' field".to_string())
            })?;

        let to_str = value.get("to").and_then(|t| t.as_str()).ok_or_else(|| {
            DecisionError::RuntimeError("Region target missing 'to' field".to_string())
        })?;

        let from_cell = GridCell::from_notation(from_str).map_err(|e| {
            DecisionError::RuntimeError(format!("Invalid 'from' cell notation: {}", e))
        })?;

        let to_cell = GridCell::from_notation(to_str).map_err(|e| {
            DecisionError::RuntimeError(format!("Invalid 'to' cell notation: {}", e))
        })?;

        let grid_dims = game.config().field.grid_dimensions();
        let region = Region::new(Team::A, from_cell, to_cell, grid_dims).map_err(|e| {
            DecisionError::RuntimeError(format!("Invalid region: {}", e))
        })?;

        Ok(DecisionTarget::Region(region))
    }

    /// Parse point coordinates from JSON value
    fn parse_point(
        value: &serde_json::Value,
        _game: &Game,
        _player_team: Team,
    ) -> Result<Point3D, DecisionError> {
        let x = value.get("x").and_then(|x| x.as_f64()).ok_or_else(|| {
            DecisionError::RuntimeError("Point target missing 'x' field".to_string())
        })? as f32;

        let z = value.get("z").and_then(|z| z.as_f64()).ok_or_else(|| {
            DecisionError::RuntimeError("Point target missing 'z' field".to_string())
        })? as f32;

        let y = value
            .get("y")
            .and_then(|y| y.as_f64())
            .map(|y| y as f32)
            .unwrap_or(0.0);

        // Point from Lua is in player's orientation - no conversion here
        // Conversion happens in DecisionSystem via convert_decision_to_display_orientation
        use uom::si::f32::Length;

        let point = Point3D {
            x: Length::new::<meter>(x),
            y: Length::new::<meter>(y),
            z: Length::new::<meter>(z),
        };

        Ok(point)
    }

    fn parse_point_target(
        value: &serde_json::Value,
        game: &Game,
        player_team: Team,
    ) -> Result<DecisionTarget, DecisionError> {
        let point = Self::parse_point(value, game, player_team)?;
        Ok(DecisionTarget::Point(point))
    }
}

impl DecisionMaker for JsonDecisionMaker {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<Decision, DecisionError> {
        // Build context
        let context = Self::build_context(game, player_index)?;

        // Get decision from engine
        let decision_json = self
            .engine
            .make_decision(player_index, &context)
            .map_err(|e| DecisionError::RuntimeError(format!("Engine error: {}", e)))?;

        // Parse JSON decision to Decision type
        let player_team = game.config().players[player_index].team;
        Self::parse_json_decision(&decision_json, game, player_team)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::field::Field;
    use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};

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
                50,
                50,
                50,
                script.to_string(),
                start_region,
            )],
            ball: BallDef::default(),
            referees: vec![RefereeDef::default()],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::new(config)
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

        let mut maker = JsonDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        assert!(matches!(decision.unwrap(), Decision::Stop));
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

        let mut maker = JsonDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        match decision.unwrap() {
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

        let mut maker = JsonDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        assert!(matches!(decision.unwrap(), Decision::Stop));
    }
}
