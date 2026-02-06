// Integration test: verify that Lua scripts produce decisions in the decision system

use uom::si::length::meter;
use ynwa_core::game::{Decision, DecisionTarget};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::System;
use ynwa_script_tests::{
    create_test_game_with_preambles, create_test_game_with_script, load_test_script,
};

/// Helper function to test that a Lua script produces expected decision type
fn test_script_produces_decision(
    script: &str,
    expected_decision_check: impl Fn(&Decision) -> bool,
    test_name: &str,
) {
    // Create game with inline script
    let mut game = create_test_game_with_script(script);

    // Use PlayerReactionSystem to set needs_decision flag
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0); // 1 second should trigger reaction

    // Create decision system with ScriptedDecisionMaker
    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

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
        "test_stop_decision",
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
        "test_run_to_cell_decision",
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
                region.top_left.col == 1
                    && region.top_left.row == 1
                    && region.bottom_right.col == 3
                    && region.bottom_right.row == 3
            } else {
                false
            }
        },
        "test_run_to_region_decision",
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
        "test_run_to_point_decision",
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
        "test_kick_decision",
    );
}

#[test]
fn test_context_structure() {
    // Test that context has all required fields according to documentation
    // Script will fail with error if any required field is missing
    let script = r#"
        function make_decision()
            local missing = {}
            
            -- Check context.me structure
            if not context.me then
                table.insert(missing, "context.me")
            else
                if not context.me.team then table.insert(missing, "context.me.team") end
                if not context.me.number then table.insert(missing, "context.me.number") end
                if not context.me.index then table.insert(missing, "context.me.index") end
                if not context.me.position then table.insert(missing, "context.me.position") end
                if context.me.position then
                    if not context.me.position.x then table.insert(missing, "context.me.position.x") end
                    if not context.me.position.z then table.insert(missing, "context.me.position.z") end
                end
            end
            
            -- Check context.teammates
            if not context.teammates then
                table.insert(missing, "context.teammates")
            end
            
            -- Check context.opponents
            if not context.opponents then
                table.insert(missing, "context.opponents")
            end
            
            -- Check context.ball structure
            if not context.ball then
                table.insert(missing, "context.ball")
            else
                if not context.ball.position then
                    table.insert(missing, "context.ball.position")
                else
                    if not context.ball.position.x then table.insert(missing, "context.ball.position.x") end
                    if not context.ball.position.z then table.insert(missing, "context.ball.position.z") end
                end
                
                -- Check if owner_index key exists (can be nil value, but key must exist)
                local has_owner_key = false
                for key in pairs(context.ball) do
                    if key == "owner_index" then
                        has_owner_key = true
                        break
                    end
                end
                if not has_owner_key then
                    table.insert(missing, "context.ball.owner_index")
                end
            end
            
            -- Check context.game
            if not context.game then
                table.insert(missing, "context.game")
            else
                if not context.game.elapsed_time then
                    table.insert(missing, "context.game.elapsed_time")
                end
            end
            
            -- If any fields are missing, report them all
            if #missing > 0 then
                error("Missing context fields: " .. table.concat(missing, ", "))
            end
            
            -- All fields exist, return valid decision
            return {action = "stop"}
        end
    "#;

    // Create game with test script
    let mut game = create_test_game_with_script(script);

    // Trigger decision
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    // If script executed successfully, all required fields exist
    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Context validation failed: {:?}",
        player_state.last_error
    );

    assert!(
        player_state.current_decision.is_some(),
        "Expected a decision to be created"
    );
}

#[test]
fn test_kick_if_ball_owner() {
    // Test the team library function: "if ball is mine, kick it in random direction"
    // This function uses am_i_ball_owner() from stdlib and ball_owner() from core
    // Since we can't easily set ball owner, we test that the function works correctly
    // when ball is free (owner_index is nil)
    let script = load_test_script("kick_if_ball_owner.lua");

    // Create game with test script
    let mut game = create_test_game_with_preambles(&script);

    // Trigger decision
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    // Verify that script executed without errors
    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Script error: {:?}",
        player_state.last_error
    );

    assert!(
        player_state.current_decision.is_some(),
        "Expected a decision"
    );

    // Since ball is free by default (owner_index is nil),
    // am_i_ball_owner() should return false, so decision should be Stop
    match &player_state.current_decision {
        Some(Decision::Stop) => {
            // Expected: ball is free, so stop
        }
        other => panic!(
            "Expected Stop decision since ball is free, got: {:?}",
            other
        ),
    }
}
