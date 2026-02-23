// Integration tests: verify stdlib functions

use ynwa_core::game::{Decision, GameStage};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::System;
use ynwa_script_tests::{
    create_test_game_football_field_with_preambles, create_test_game_with_full_preambles_and_stage,
    create_test_game_with_preambles, load_test_script, request_decisions_for_all,
};

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
    
    local idx = my_index()
    if type(idx) ~= "number" then
        error("my_index() failed")
    end
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
fn test_get_setup_position_runs_to_start_region() {
    // During Setup stage, get_setup_position() from the team preamble calls
    // default_get_setup_position() from stdlib and returns a Run decision
    // towards the center of the "start position" region.
    // The test game places the player's start_region at cells (10,10)-(11,11).

    // Script has no get_setup_position() — team preamble definition is used.
    let script = r#"
function make_decision()
    return {action = "stop"}
end
"#;

    let mut game = create_test_game_with_full_preambles_and_stage(
        script,
        GameStage::Setup("start".to_string()),
    );

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "get_setup_position() error: {:?}",
        player_state.last_error
    );

    // Should produce a Run decision (not Stop) — player needs to reach start position
    assert!(
        matches!(player_state.current_decision, Some(Decision::Run(_))),
        "Expected Run decision from get_setup_position(), got: {:?}",
        player_state.current_decision
    );
}

#[test]
fn test_kick_to_opponent_goal() {
    // kick_to_opponent_goal() must aim at the center of the opponent goal zone.
    // We derive expected coordinates from the actual field, not from hardcoded constants,
    // so the test stays valid if field dimensions change.
    let script = load_test_script("kick_to_opponent_goal.lua");
    let mut game = create_test_game_football_field_with_preambles(&script);
    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "kick_to_opponent_goal() error: {:?}",
        player_state.last_error
    );

    let Some(ynwa_core::game::Decision::Kick(target)) = &player_state.current_decision else {
        panic!(
            "Expected Kick decision, got: {:?}",
            player_state.current_decision
        );
    };

    use uom::si::length::meter;
    use ynwa_core::field::zones::ZoneGeometry;
    use ynwa_core::team::Team;

    // Team A player → opponent goal is goal_b
    let goal_zone = game
        .config()
        .field
        .get_zone("goal", Some(Team::B))
        .expect("goal_b must exist on football field");
    let ZoneGeometry::Rectangle(ref rect) = goal_zone.geometry else {
        panic!("goal zone must be a Rectangle");
    };

    let target_x = target.x.get::<meter>();
    let target_z = target.z.get::<meter>();
    let goal_min_x = rect.min.x.get::<meter>();
    let goal_max_x = rect.max.x.get::<meter>();
    let goal_min_z = rect.min.z.get::<meter>();
    let goal_max_z = rect.max.z.get::<meter>();

    let expected_x = (goal_min_x + goal_max_x) / 2.0;
    let expected_z = (goal_min_z + goal_max_z) / 2.0;

    assert!(
        (target_x - expected_x).abs() < 0.1,
        "kick target x={target_x:.4} must equal goal center x={expected_x:.4}"
    );
    assert!(
        (target_z - expected_z).abs() < 0.1,
        "kick target z={target_z:.4} must equal goal center z={expected_z:.4}"
    );
}
