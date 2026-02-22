/// Integration tests for team orientation coordinate conversion.
/// Tests that Team B decisions are correctly converted from Team B's perspective
/// (right-to-left) to display orientation (Team A's left-to-right perspective).
use ynwa_core::field::Field;
use ynwa_core::game::{BallDef, Decision, DecisionTarget, Game, GameConfig, PlayerDef, RefereeDef};
use ynwa_core::region::{GridCell};
use ynwa_core::system::System;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::team::Team;

/// Helper function to create a test game with a specific script for a team
/// Using football field dimensions: 105m x 68m with 18 columns x 44 rows
fn create_game_with_team_script(team: Team, script: String) -> Game {
    let field = Field::from_meters(105.0, 68.0, 18, 44);
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::GridCell(cell)) => {
            // Team B's A1 should flip to Team A's R44 (column 18, row 44 in 18x44 grid)
            assert_eq!(
                cell,
                &GridCell::new(18, 44).unwrap(),
                "Team B: A1 in Team B coords should become R44 in display coords"
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::Region(region)) => {
            // After flip: team remains A (lua_format always creates Team A regions),
            // but coordinates flip
            // A1 -> R44, B2 -> Q43 (in 18x44 grid)
            assert_eq!(
                region.top_left,
                GridCell::new(17, 43).unwrap(),
                "Team B: B2 in Team B coords should become Q43 (17,43) in display coords"
            );
            assert_eq!(
                region.bottom_right,
                GridCell::new(18, 44).unwrap(),
                "Team B: A1 in Team B coords should become R44 (18,44) in display coords"
            );
        }
        _ => panic!("Expected Run(Region)"),
    }
}

#[test]
fn test_team_b_point_flipped() {
    // Team B script returns point (20, 0, 15), should flip to (85, 0, 53) in display coords
    // Football field is 105m x 68m
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");

    match decision.as_ref().unwrap() {
        Decision::Run(DecisionTarget::Point(point)) => {
            use uom::si::length::meter;

            // Football field is 105m x 68m
            // Point (20, 0, 15) for Team B should flip to:
            // x: 105 - 20 = 85
            // y: unchanged = 0
            // z: 68 - 15 = 53
            let tolerance = 0.01;
            assert!(
                (point.x.get::<meter>() - 85.0).abs() < tolerance,
                "Expected x=85.0, got {}",
                point.x.get::<meter>()
            );
            assert!(
                (point.y.get::<meter>() - 0.0).abs() < tolerance,
                "Expected y=0.0, got {}",
                point.y.get::<meter>()
            );
            assert!(
                (point.z.get::<meter>() - 53.0).abs() < tolerance,
                "Expected z=53.0, got {}",
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
    let mut reaction_system = PlayerReactionSystem::new();
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(ScriptedDecisionMaker::new(&game).unwrap()));

    // Trigger decision
    reaction_system.update(&mut game, 0.0);
    decision_system.update(&mut game, 0.0);

    // Check decision
    let decision = &game.state().player_states[0].current_decision;
    assert!(decision.is_some(), "Expected a decision");
    assert!(matches!(decision.as_ref().unwrap(), Decision::Stop));
}
