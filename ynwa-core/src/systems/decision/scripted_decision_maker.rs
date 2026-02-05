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
        ).map_err(|e| {
            DecisionError::RuntimeError(format!("Failed to create DecisionEngine: {}", e))
        })?;

        Ok(Self { engine })
    }

    fn build_config(game: &Game) -> serde_json::Value {
        let config = game.config();
        let scripting = &config.scripting;
        
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
                "index": player_index,
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
                "position": Self::position_to_json(&state.ball_state.position, player_team, field_width, field_length),
                "owner_index": state.ball_state.possessed_by
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
}

impl DecisionMaker for ScriptedDecisionMaker {
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

        // Parse JSON decision to Decision type using decision_parser
        decision_parser::parse_decision(&decision_json)
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

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
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

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
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

        let mut maker = ScriptedDecisionMaker::new(&game).unwrap();
        let decision = maker.make_decision(&game, 0);

        assert!(decision.is_ok());
        assert!(matches!(decision.unwrap(), Decision::Stop));
    }
}
