/// Integration tests for team orientation coordinate conversion.
/// Tests that Team B decisions are correctly converted from Team B's perspective
/// (right-to-left) to display orientation (Team A's left-to-right perspective).
use ynwa_core::football::field_builder::create_football_field;
use ynwa_core::game::{BallDef, Decision, DecisionTarget, Game, GameConfig, PlayerDef, RefereeDef};
use ynwa_core::region::GridCell;
use ynwa_core::system::System;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::team::Team;
use ynwa_script_tests::request_decisions_for_all;

/// Helper function to create a test game with a specific script for a team.
/// Uses the standard football field from the single source of truth: create_football_field().
fn create_game_with_team_script(team: Team, script: String) -> Game {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();

    // For Team A, start near their goal (left side)
    // For Team B, start near their goal (right side, but in Team B coordinates = left in their view)
    let start_region = if team == Team::A {
        grid_dims.create_region(GridCell::new(1, 20).unwrap(), GridCell::new(1, 20).unwrap())
        .unwrap()
    } else {
        // Team B starts at A20 in Team B coordinates (which is their left side)
        grid_dims.create_region(GridCell::new(1, 20).unwrap(), GridCell::new(1, 20).unwrap())
        .unwrap()
    };

    let player = PlayerDef::new(team, 1, format!("Player {:?}1", team), script, start_region)
        .with_reaction_rate(100); // Fast reaction rate

    let config = GameConfig {
        field,
        players: vec![player],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(config, ynwa_core::game::GameStage::Play)
}

#[test]
fn test_team_a_cell_unchanged() {
    // Team A script returns A1, should remain A1 in display coords
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "cell",
                target = "A1"
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::A, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::GridCell(cell)) => {
            assert_eq!(
                cell,
                &GridCell::new(1, 1).unwrap(),
                "Team A: A1 should remain A1"
            );
        }
        _ => panic!("Expected Run(GridCell)"),
    }
}

#[test]
fn test_team_b_cell_flipped() {
    // Team B script returns A1 (in their coordinates), should become R44 in display coords
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "cell",
                target = "A1"
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::B, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::GridCell(cell)) => {
            // Team B's A1 should flip to Team A's Z44 (column 26, row 44 in 26x44 grid)
            assert_eq!(
                cell,
                &GridCell::new(26, 44).unwrap(),
                "Team B: A1 in Team B coords should become Z44 in display coords"
            );
        }
        _ => panic!("Expected Run(GridCell)"),
    }
}

#[test]
fn test_team_a_region_unchanged() {
    // Team A script returns region A1:B2, should remain A1:B2 in display coords
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "region",
                target = {
                    from = "A1",
                    to = "B2"
                }
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::A, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::Region(region)) => {
            assert_eq!(region.top_left, GridCell::new(1, 1).unwrap());
            assert_eq!(region.bottom_right, GridCell::new(2, 2).unwrap());
        }
        _ => panic!("Expected Run(Region)"),
    }
}

#[test]
fn test_team_b_region_flipped() {
    // Team B script returns region A1:B2 (Team B perspective),
    // should flip to Team A region with flipped coordinates
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "region",
                target = {
                    from = "A1",
                    to = "B2"
                }
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::B, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::Region(region)) => {
            // After flip: team remains A (lua_format always creates Team A regions),
            // but coordinates flip
            // A1 -> Z44, B2 -> Y43 (in 26x44 grid)
            assert_eq!(
                region.top_left,
                GridCell::new(25, 43).unwrap(),
                "Team B: B2 in Team B coords should become Y43 (25,43) in display coords"
            );
            assert_eq!(
                region.bottom_right,
                GridCell::new(26, 44).unwrap(),
                "Team B: A1 in Team B coords should become Z44 (26,44) in display coords"
            );
        }
        _ => panic!("Expected Run(Region)"),
    }
}

#[test]
fn test_team_b_point_flipped() {
    // Team B script returns point (20, 0, 15), should flip using real field dimensions
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "point",
                target = {
                    x = 20.0,
                    y = 0.0,
                    z = 15.0
                }
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::B, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::Point(point)) => {
            use uom::si::length::meter;

            let field = create_football_field();
            let field_width = field.width().get::<meter>();
            let field_length = field.length().get::<meter>();

            // Point (20, 0, 15) for Team B should flip to:
            // x: field_width - 20, y: unchanged, z: field_length - 15
            let expected_x = field_width - 20.0;
            let expected_z = field_length - 15.0;

            let tolerance = 0.01;
            assert!(
                (point.x.get::<meter>() - expected_x).abs() < tolerance,
                "Expected x={}, got {}",
                expected_x,
                point.x.get::<meter>()
            );
            assert!(
                (point.y.get::<meter>() - 0.0).abs() < tolerance,
                "Expected y=0.0, got {}",
                point.y.get::<meter>()
            );
            assert!(
                (point.z.get::<meter>() - expected_z).abs() < tolerance,
                "Expected z={}, got {}",
                expected_z,
                point.z.get::<meter>()
            );
        }
        _ => panic!("Expected Run(Point)"),
    }
}

#[test]
fn test_team_b_stop_unchanged() {
    // Stop decision should remain unchanged for both teams
    let script = r#"
        function make_decision()
            return {
                action = "stop"
            }
        end
    "#
    .to_string();

    let mut game = create_game_with_team_script(Team::B, script);
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    request_decisions_for_all(&mut game);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");
    assert!(matches!(decision.as_ref().unwrap(), Decision::Stop));
}
