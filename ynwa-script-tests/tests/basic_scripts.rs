// Integration test: verify that Lua scripts produce decisions in the decision system

use uom::si::length::meter;
use ynwa_core::game::{Decision, DecisionTarget};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::System;
use ynwa_script_tests::{
    create_test_game_with_preambles, create_test_game_with_script, load_test_script,
    request_decisions_for_all,
};

/// Helper function to test that a Lua script produces expected decision type
fn test_script_produces_decision(
    script: &str,
    expected_decision_check: impl Fn(&Decision) -> bool,
    test_name: &str,
) {
    // Create game with inline script
    let mut game = create_test_game_with_script(script);

    request_decisions_for_all(&mut game);

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

    request_decisions_for_all(&mut game);

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

    request_decisions_for_all(&mut game);

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

#[test]
fn test_ball_owner_team_context() {
    use ynwa_core::team::Team;

    // Script that checks ball.owner_team
    let script = r#"
        function make_decision()
            local owner_team = context.ball.owner_team
            
            -- Check that owner_team exists and is a string
            if owner_team == nil then
                error("owner_team is nil")
            end
            
            if type(owner_team) ~= "string" then
                error("owner_team is not a string, got: " .. type(owner_team))
            end
            
            -- Return different decisions based on owner_team
            if owner_team == "A" then
                return {action = "run", target_type = "cell", target = "A1"}
            elseif owner_team == "B" then
                return {action = "run", target_type = "cell", target = "B1"}
            elseif owner_team == "None" then
                return {action = "stop"}
            else
                error("Unknown owner_team: " .. owner_team)
            end
        end
    "#;

    // Test 1: Neutral ball (owner_team = "None")
    let mut game = create_test_game_with_script(script);
    game.state.ball_state.last_possessing_team = None;

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Script error with neutral ball: {:?}",
        player_state.last_error
    );
    assert!(
        matches!(player_state.current_decision, Some(Decision::Stop)),
        "Expected Stop for neutral ball, got: {:?}",
        player_state.current_decision
    );

    // Test 2: Team A owns ball (owner_team = "A")
    let mut game = create_test_game_with_script(script);
    game.state.ball_state.last_possessing_team = Some(Team::A);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Script error with Team A ball: {:?}",
        player_state.last_error
    );
    assert!(
        matches!(
            player_state.current_decision,
            Some(Decision::Run(DecisionTarget::GridCell(_)))
        ),
        "Expected Run decision for Team A ball, got: {:?}",
        player_state.current_decision
    );

    // Test 3: Team B owns ball (owner_team = "B")
    let mut game = create_test_game_with_script(script);
    game.state.ball_state.last_possessing_team = Some(Team::B);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Script error with Team B ball: {:?}",
        player_state.last_error
    );
    assert!(
        matches!(
            player_state.current_decision,
            Some(Decision::Run(DecisionTarget::GridCell(_)))
        ),
        "Expected Run decision for Team B ball, got: {:?}",
        player_state.current_decision
    );
}

// ── DecisionTarget::Ball tests ────────────────────────────────────────────────

/// Lua script returns `target_type = "ball"` → Rust should produce `DecisionTarget::Ball`.
#[test]
fn test_run_to_ball_decision() {
    test_script_produces_decision(
        r#"
        function make_decision()
            return {
                action = "run",
                target_type = "ball"
            }
        end
        "#,
        |d| matches!(d, Decision::Run(DecisionTarget::Ball)),
        "test_run_to_ball_decision",
    );
}

/// `target_type = "ball"` should work even when the optional `target` field is absent.
#[test]
fn test_run_to_ball_no_target_field() {
    test_script_produces_decision(
        r#"
        function make_decision()
            -- No "target" field — should be accepted without error
            return {action = "run", target_type = "ball"}
        end
        "#,
        |d| matches!(d, Decision::Run(DecisionTarget::Ball)),
        "test_run_to_ball_no_target_field",
    );
}

/// A script may include a `reason` alongside `target_type = "ball"`.
/// The reason should be preserved.
#[test]
fn test_run_to_ball_with_reason() {
    let script = r#"
        function make_decision()
            return {
                action = "run",
                target_type = "ball",
                reason = "chasing the ball"
            }
        end
    "#;

    let mut game = create_test_game_with_script(script);
    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Unexpected script error: {:?}",
        player_state.last_error
    );
    assert!(
        matches!(
            player_state.current_decision,
            Some(Decision::Run(DecisionTarget::Ball))
        ),
        "Expected Run(Ball), got: {:?}",
        player_state.current_decision
    );
    assert_eq!(
        player_state.decision_reason.as_deref(),
        Some("chasing the ball"),
        "Expected reason to be preserved"
    );
}
