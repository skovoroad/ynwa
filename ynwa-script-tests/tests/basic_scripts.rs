// Integration test: verify that Lua scripts produce decisions in the decision system

use ynwa_core::game::{Decision, DecisionTarget};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::System;
use ynwa_script_tests::create_test_game_with_script;
use uom::si::length::meter;

/// Helper function to test that a Lua script produces expected decision type
fn test_script_produces_decision(script: &str, expected_decision_check: impl Fn(&Decision) -> bool, test_name: &str) {
    // Create game with inline script
    let mut game = create_test_game_with_script(script);
    
    // Use PlayerReactionSystem to set needs_decision flag
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0); // 1 second should trigger reaction
    
    // Create decision system with ScriptedDecisionMaker
    let decision_maker = ScriptedDecisionMaker::new(&game)
        .expect("Failed to create ScriptedDecisionMaker");
    
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(decision_maker));
    
    // Execute decision system
    decision_system.update(&mut game, 1.0);
    
    // Verify that player has expected decision
    let player_state = &game.state().player_states[0];
    assert!(
        player_state.current_decision.is_some(), 
        "{}: Player should have a decision", 
        test_name
    );
    
    let decision = player_state.current_decision.as_ref().unwrap();
    assert!(
        expected_decision_check(decision),
        "{}: Unexpected decision type: {:?}",
        test_name,
        decision
    );
}

#[test]
fn test_stop_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {action = "stop"}
        end
        "#,
        |d| matches!(d, Decision::Stop),
        "test_stop_decision"
    );
}

#[test]
fn test_run_to_cell_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {
                action = "run",
                target_type = "cell",
                target = "M13"
            }
        end
        "#,
        |d| {
            if let Decision::Run(DecisionTarget::GridCell(cell)) = d {
                // M = 13th letter (1-based), row 13
                cell.col == 13 && cell.row == 13
            } else {
                false
            }
        },
        "test_run_to_cell_decision"
    );
}

#[test]
fn test_run_to_region_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {
                action = "run",
                target_type = "region",
                target = {from = "A1", to = "C3"}
            }
        end
        "#,
        |d| {
            if let Decision::Run(DecisionTarget::Region(region)) = d {
                // A=1, C=3 (1-based columns)
                region.top_left.col == 1 && region.top_left.row == 1
                    && region.bottom_right.col == 3 && region.bottom_right.row == 3
            } else {
                false
            }
        },
        "test_run_to_region_decision"
    );
}

#[test]
fn test_run_to_point_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {
                action = "run",
                target_type = "point",
                target = {x = 50.0, z = 30.0}
            }
        end
        "#,
        |d| {
            if let Decision::Run(DecisionTarget::Point(point)) = d {
                let x = point.x.get::<meter>();
                let z = point.z.get::<meter>();
                (x - 50.0).abs() < 0.01 && (z - 30.0).abs() < 0.01
            } else {
                false
            }
        },
        "test_run_to_point_decision"
    );
}

#[test]
fn test_kick_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {
                action = "kick",
                target = {x = 75.0, z = 30.0}
            }
        end
        "#,
        |d| {
            if let Decision::Kick(point) = d {
                let x = point.x.get::<meter>();
                let z = point.z.get::<meter>();
                (x - 75.0).abs() < 0.01 && (z - 30.0).abs() < 0.01
            } else {
                false
            }
        },
        "test_kick_decision"
    );
}
