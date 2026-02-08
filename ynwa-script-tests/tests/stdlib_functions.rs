// Integration tests: verify stdlib functions

use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::System;
use ynwa_script_tests::create_test_game_with_preambles;

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

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

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

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

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

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

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
