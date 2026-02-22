// Integration tests: verify stdlib functions

use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::System;
use ynwa_script_tests::{create_test_game_with_preambles, request_decisions_for_all};

// Stub make_decision() that does nothing - required by game engine
const MAKE_DECISION_STUB: &str = r#"
function make_decision()
    return {action = "stop"}
end
"#;

#[test]
fn test_core_preamble_functions() {
    // Test all functions from core preamble
    let test_script = r#"
function test_core_functions()
    -- Test core functions
    local my_pos = my_position()
    if not my_pos or not my_pos.x or not my_pos.z then
        error("my_position() failed")
    end
    
    local ball_pos = ball_position()
    if not ball_pos or not ball_pos.x or not ball_pos.z then
        error("ball_position() failed")
    end
    
    local teammates = get_teammates()
    if type(teammates) ~= "table" then
        error("get_teammates() failed")
    end
    
    local opponents = get_opponents()
    if type(opponents) ~= "table" then
        error("get_opponents() failed")
    end
    
    local idx = my_index()
    if type(idx) ~= "number" then
        error("my_index() failed")
    end
    
    -- Test decision factories
    local _ = stop()
    local _ = kick_to(50, 30)
    local _ = run_to_point(40, 20)
    local _ = run_to_random_position()
end

test_core_functions()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Core functions test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_distance_function() {
    // Test the distance calculation function
    let test_script = r#"
function test_distance()
    local pos1 = {x = 0.0, z = 0.0}
    local pos2 = {x = 3.0, z = 4.0}
    local dist = distance(pos1, pos2)
    
    -- Distance should be 5.0 (3-4-5 triangle)
    if math.abs(dist - 5.0) > 0.01 then
        error("Distance calculation failed: expected 5.0, got " .. tostring(dist))
    end
end

test_distance()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Distance test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_ball_owned_by_my_team() {
    // Test ball ownership checking for team
    let test_script = r#"
function test_ball_ownership()
    -- Ball is free by default in test, so should return false
    local owned = is_ball_owned_by_my_team()
    if type(owned) ~= "boolean" then
        error("is_ball_owned_by_my_team() should return boolean")
    end
    
    -- In default test setup, ball should be free
    if owned then
        error("Ball should be free in test setup")
    end
end

test_ball_ownership()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles(&script);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));

    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "is_ball_owned_by_my_team test failed: {:?}",
        player_state.last_error
    );
}

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

    request_decisions_for_all(&mut game);

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

    request_decisions_for_all(&mut game);

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

    request_decisions_for_all(&mut game);

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
