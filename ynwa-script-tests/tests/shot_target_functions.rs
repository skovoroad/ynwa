/// Tests for goal shot target functions
use ynwa_core::field::zones::{Rectangle, ZoneGeometry};
use ynwa_core::field::Zone;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::team::Team;
use ynwa_core::System;
use ynwa_script_tests::{create_test_game_with_preambles_and_zones, request_decisions_for_all};

const MAKE_DECISION_STUB: &str = r#"
function make_decision()
    return {action = "stop"}
end
"#;

#[test]
fn test_get_random_shot_target_to_goal_team_a() {
    // Create goal zones for both teams
    let zones = vec![
        Zone::new(
            "goal",
            Some(Team::A),
            // Team A goal: x from -2.0 to 0.0, z from 27.0 to 33.0
            ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.0, 0.0, 33.0)),
        ),
        Zone::new(
            "goal",
            Some(Team::B),
            // Team B goal: x from 100.0 to 102.0, z from 27.0 to 33.0
            ZoneGeometry::Rectangle(Rectangle::from_meters(100.0, 27.0, 102.0, 33.0)),
        ),
    ];

    let test_script = r#"
function test_shot_target()
    -- Test multiple random shots to Team A goal
    for i = 1, 10 do
        local target = get_random_shot_target_to_goal("A")
        
        if target == nil then
            error("get_random_shot_target_to_goal returned nil for team A")
        end
        
        -- Check that target is on the front line of Team A goal (x = 0.0)
        if math.abs(target.x - 0.0) > 0.01 then
            error(string.format("Shot target x=%f is not on front line (expected 0.0)", target.x))
        end
        
        -- Check that target is within goal width (z: 27.0 to 33.0)
        if target.z < 27.0 or target.z > 33.0 then
            error(string.format("Shot target z=%f is outside goal width (27.0-33.0)", target.z))
        end
        
        -- Check y coordinate is 0
        if target.y ~= 0 then
            error(string.format("Shot target y=%f should be 0", target.y))
        end
    end
end

test_shot_target()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Team A shot target test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_get_random_shot_target_to_goal_team_b() {
    let zones = vec![
        Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.0, 0.0, 33.0)),
        ),
        Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(100.0, 27.0, 102.0, 33.0)),
        ),
    ];

    let test_script = r#"
function test_shot_target()
    -- Test multiple random shots to Team B goal
    for i = 1, 10 do
        local target = get_random_shot_target_to_goal("B")
        
        if target == nil then
            error("get_random_shot_target_to_goal returned nil for team B")
        end
        
        -- Check that target is on the front line of Team B goal (x = 100.0)
        if math.abs(target.x - 100.0) > 0.01 then
            error(string.format("Shot target x=%f is not on front line (expected 100.0)", target.x))
        end
        
        -- Check that target is within goal width (z: 27.0 to 33.0)
        if target.z < 27.0 or target.z > 33.0 then
            error(string.format("Shot target z=%f is outside goal width (27.0-33.0)", target.z))
        end
        
        -- Check y coordinate is 0
        if target.y ~= 0 then
            error(string.format("Shot target y=%f should be 0", target.y))
        end
    end
end

test_shot_target()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Team B shot target test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_get_random_shot_target_case_insensitive() {
    let zones = vec![
        Zone::new(
            "goal",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.0, 0.0, 33.0)),
        ),
        Zone::new(
            "goal",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(100.0, 27.0, 102.0, 33.0)),
        ),
    ];

    let test_script = r#"
function test_case_insensitive()
    -- Test lowercase
    local target_a = get_random_shot_target_to_goal("a")
    if target_a == nil then
        error("Failed with lowercase 'a'")
    end
    
    local target_b = get_random_shot_target_to_goal("b")
    if target_b == nil then
        error("Failed with lowercase 'b'")
    end
    
    -- Test uppercase (already tested in other tests)
    local target_A = get_random_shot_target_to_goal("A")
    if target_A == nil then
        error("Failed with uppercase 'A'")
    end
    
    local target_B = get_random_shot_target_to_goal("B")
    if target_B == nil then
        error("Failed with uppercase 'B'")
    end
end

test_case_insensitive()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Case insensitive test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_get_random_shot_target_randomness() {
    let zones = vec![Zone::new(
        "goal",
        Some(Team::A),
        ZoneGeometry::Rectangle(Rectangle::from_meters(-2.0, 27.0, 0.0, 33.0)),
    )];

    let test_script = r#"
function test_randomness()
    -- Generate multiple targets and check they are different
    local targets = {}
    local unique_count = 0
    
    for i = 1, 20 do
        local target = get_random_shot_target_to_goal("A")
        local key = string.format("%.2f", target.z)
        
        if not targets[key] then
            targets[key] = true
            unique_count = unique_count + 1
        end
    end
    
    -- With 20 random samples in 6-meter range, we should get at least 5 unique values
    if unique_count < 5 then
        error(string.format("Only %d unique targets in 20 samples - not random enough", unique_count))
    end
end

test_randomness()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    request_decisions_for_all(&mut game);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Randomness test failed: {:?}",
        player_state.last_error
    );
}
