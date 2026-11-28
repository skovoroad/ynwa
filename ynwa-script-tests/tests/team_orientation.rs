/// Integration tests for team orientation coordinate conversion.
/// Tests that Team B decisions are correctly converted from Team B's perspective
/// (right-to-left) to display orientation (Team A's left-to-right perspective).
use ynwa_core::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, PlayerDef, RefereeDef,
    REGION_START_POSITION,
};
use ynwa_core::region::GridCell;
use ynwa_core::system::System;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::team::Team;
use ynwa_script_tests::{
    create_test_game_with_all_preambles, deterministic_rng, request_decisions_for_all,
};

/// Helper function to create a test game with a specific script for a team.
/// Uses a standard football field (68×104.6m, 26×40 grid) without zones — zones are not needed here.
fn create_game_with_team_script(team: Team, script: String) -> Game {
    let field = ynwa_core::field::Field::from_meters(68.0, 104.615_38, 26, 40);
    let grid_dims = field.grid_dimensions();

    // For Team A, start near their goal (left side)
    // For Team B, start near their goal (right side, but in Team B coordinates = left in their view)
    let start_region = if team == Team::A {
        grid_dims
            .create_region(GridCell::new(1, 20).unwrap(), GridCell::new(1, 20).unwrap())
            .unwrap()
    } else {
        // Team B starts at A20 in Team B coordinates (which is their left side)
        grid_dims
            .create_region(GridCell::new(1, 20).unwrap(), GridCell::new(1, 20).unwrap())
            .unwrap()
    };

    let player = PlayerDef::new(
        team,
        1,
        format!("Player {:?}1", team),
        script,
        std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
    .with_reaction_rate(100); // Fast reaction rate

    let config = GameConfig {
        field,
        players: vec![player],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        team_a_id: "team_a".to_string(),
        team_b_id: "team_b".to_string(),
        team_a_display_name: "Team A".to_string(),
        team_b_display_name: "Team B".to_string(),
        scripting: ynwa_core::game::ScriptingConfig::empty(),
    };

    Game::with_stage(
        config,
        ynwa_core::game::GameStage::Play,
        deterministic_rng(),
    )
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
            // Team B's A1 should flip to Team A's Z40 (column 26, row 40 in 26x40 grid)
            assert_eq!(
                cell,
                &GridCell::new(26, 40).unwrap(),
                "Team B: A1 in Team B coords should become Z40 in display coords"
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
            // A1 -> Z40, B2 -> Y39 (in 26x40 grid)
            assert_eq!(
                region.top_left,
                GridCell::new(25, 39).unwrap(),
                "Team B: B2 in Team B coords should become Y39 (25,39) in display coords"
            );
            assert_eq!(
                region.bottom_right,
                GridCell::new(26, 40).unwrap(),
                "Team B: A1 in Team B coords should become Z40 (26,40) in display coords"
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

            let field_width = 68.0_f32;
            let field_length = 104.615_38_f32;

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

fn script_test_player(team: Team, number: u32, script: &str) -> PlayerDef {
    let field = ynwa_football::field_builder::create_football_field();
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    PlayerDef::new(
        team,
        number,
        format!("Player {number}"),
        script.to_string(),
        std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
}

/// Builds a game whose caller is the first player of `team` and returns the caller's decision.
/// Positions and velocities are given in display orientation; `build_context` converts them into
/// the caller's team orientation and `DecisionSystem` converts the decision back.
fn run_caller_decision(
    team: Team,
    caller_script: &str,
    caller: (f32, f32),
    teammate: (f32, f32),
    teammate_velocity: (f32, f32),
    ball: (f32, f32, f32, f32),
) -> Decision {
    use ynwa_core::field::zones::{Point3D, Velocity3D};

    let players = vec![
        script_test_player(team, 1, caller_script),
        script_test_player(team, 2, ""),
        script_test_player(team.opposite(), 1, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    for state in game.state.player_states.iter_mut() {
        state.needs_decision = false;
    }

    game.state.player_states[0].position = Point3D::from_meters(caller.0, 0.0, caller.1);
    game.state.player_states[1].position = Point3D::from_meters(teammate.0, 0.0, teammate.1);
    game.state.player_states[1].velocity =
        Velocity3D::from_meters_per_second(teammate_velocity.0, 0.0, teammate_velocity.1);
    game.state.ball_state.position = Point3D::from_meters(ball.0, 0.0, ball.1);
    game.state.ball_state.velocity = Velocity3D::from_meters_per_second(ball.2, 0.0, ball.3);
    game.state.player_states[0].needs_decision = true;

    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);

    game.state().player_states[0]
        .current_decision
        .clone()
        .unwrap_or_else(|| {
            panic!(
                "no decision assigned: {:?}",
                game.state().player_states[0].last_error
            )
        })
}

fn point_of(decision: &Decision) -> (f32, f32) {
    use uom::si::length::meter;
    match decision {
        Decision::Run(DecisionTarget::Point(point)) | Decision::Kick(point) => {
            (point.x.get::<meter>(), point.z.get::<meter>())
        }
        other => panic!("expected a point target, got {other:?}"),
    }
}

fn assert_same_point(a: (f32, f32), b: (f32, f32)) {
    assert!(
        (a.0 - b.0).abs() < 0.01 && (a.1 - b.1).abs() < 0.01,
        "expected identical display targets, got {a:?} and {b:?}"
    );
}

#[test]
fn test_chase_ball_lead_is_symmetric_across_teams() {
    let physical = (
        (20.0, 30.0),           // caller
        (0.0, 0.0),             // teammate (unused by chase_ball)
        (0.0, 0.0),             // teammate velocity
        (40.0, 50.0, 4.0, 6.0), // ball position and velocity
    );
    let script = "function make_decision() return chase_ball() end";

    let decision_a = run_caller_decision(
        Team::A,
        script,
        physical.0,
        physical.1,
        physical.2,
        physical.3,
    );
    let decision_b = run_caller_decision(
        Team::B,
        script,
        physical.0,
        physical.1,
        physical.2,
        physical.3,
    );

    let target_a = point_of(&decision_a);
    let target_b = point_of(&decision_b);
    assert_same_point(target_a, target_b);
    // Ball speed 7.21 m/s caps the lead at 1.5 s → 6 m / 9 m ahead of (40, 50).
    assert!((target_a.0 - 46.0).abs() < 0.01 && (target_a.1 - 59.0).abs() < 0.01);
}

#[test]
fn test_pass_to_lead_is_symmetric_across_teams() {
    let physical = (
        (20.0, 30.0),         // caller
        (40.0, 50.0),         // teammate position
        (4.0, 6.0),           // teammate velocity
        (0.0, 0.0, 0.0, 0.0), // ball
    );
    let script = "function make_decision() return pass_to(2) end";

    let decision_a = run_caller_decision(
        Team::A,
        script,
        physical.0,
        physical.1,
        physical.2,
        physical.3,
    );
    let decision_b = run_caller_decision(
        Team::B,
        script,
        physical.0,
        physical.1,
        physical.2,
        physical.3,
    );

    let target_a = point_of(&decision_a);
    let target_b = point_of(&decision_b);
    assert_same_point(target_a, target_b);
    // Distance 28.28 m at 12 m/s caps the lead at 1.5 s → 6 m / 9 m ahead of (40, 50).
    assert!((target_a.0 - 46.0).abs() < 0.01 && (target_a.1 - 59.0).abs() < 0.01);
}
