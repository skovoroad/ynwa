use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::System;
use ynwa_script_tests::create_test_game_with_preambles;

const MAKE_DECISION_STUB: &str = r#"
function make_decision()
    return {action = "stop"}
end
"#;

#[test]
fn test_opponent_team_function() {
    // Test opponent_team function for both teams
    let test_script = r#"
function test_opponent_team()
    local my_team = my_team_name()
    local opp_team = opponent_team()
    
    -- Validate opponent team is different from my team
    if my_team == opp_team then
        error("Opponent team should be different from my team")
    end
    
    -- Validate opponent team is either A or B
    if opp_team ~= "A" and opp_team ~= "B" then
        error("Opponent team must be A or B, got: " .. tostring(opp_team))
    end
    
    -- Validate correct mapping
    if my_team == "A" and opp_team ~= "B" then
        error("Team A opponent should be B")
    end
    
    if my_team == "B" and opp_team ~= "A" then
        error("Team B opponent should be A")
    end
end

test_opponent_team()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "opponent_team test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_in_zone_function() {
    // Test is_in_zone function
    let test_script = r#"
function test_is_in_zone()
    -- Test with penalty area zone
    local my_team = my_team_name()
    local zone_suffix = string.lower(my_team)
    local penalty_zone = "penalty_area_" .. zone_suffix
    
    -- Get my position
    local my_pos = my_position()
    
    -- Test calling is_in_zone
    local result = is_in_zone(penalty_zone, my_pos.x, my_pos.z)
    
    -- Result should be boolean
    if type(result) ~= "boolean" then
        error("is_in_zone should return boolean")
    end
    
    -- Test without coordinates (should use my position)
    local result2 = is_in_zone(penalty_zone)
    if type(result2) ~= "boolean" then
        error("is_in_zone without coords should return boolean")
    end
    
    -- Test with non-existent zone
    local result3 = is_in_zone("non_existent_zone")
    if result3 ~= false then
        error("is_in_zone with non-existent zone should return false")
    end
end

test_is_in_zone()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "is_in_zone test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_am_i_in_opponent_penalty_area() {
    // Test am_i_in_opponent_penalty_area function
    let test_script = r#"
function test_penalty_area()
    local result = am_i_in_opponent_penalty_area()
    
    -- Should return boolean
    if type(result) ~= "boolean" then
        error("am_i_in_opponent_penalty_area should return boolean")
    end
end

test_penalty_area()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "am_i_in_opponent_penalty_area test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_common_behavior_v3_executes() {
    // Test that common_behavior_v3 executes without errors
    let test_script = r#"
function make_decision()
    return common_behavior_v3()
end
"#;

    let mut game = create_test_game_with_preambles(test_script);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "common_behavior_v3 execution failed: {:?}",
        player_state.last_error
    );
    
    // Should have a decision
    assert!(
        player_state.current_decision.is_some(),
        "common_behavior_v3 should produce a decision"
    );
}

#[test]
fn test_goalkeeper_behavior_executes() {
    // Test that goalkeeper_behavior executes without errors
    let test_script = r#"
function make_decision()
    return goalkeeper_behavior()
end
"#;

    let mut game = create_test_game_with_preambles(test_script);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "goalkeeper_behavior execution failed: {:?}",
        player_state.last_error
    );
    
    // Should have a decision
    assert!(
        player_state.current_decision.is_some(),
        "goalkeeper_behavior should produce a decision"
    );
}
