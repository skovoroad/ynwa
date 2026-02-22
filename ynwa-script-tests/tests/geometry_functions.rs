use ynwa_core::field::zones::{Circle, Rectangle, ZoneGeometry};
use ynwa_core::field::Zone;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::team::Team;
use ynwa_core::System;
use ynwa_script_tests::create_test_game_with_preambles_and_zones;

const MAKE_DECISION_STUB: &str = r#"
function make_decision()
    return {action = "stop"}
end
"#;

#[test]
fn test_is_point_in_rectangle() {
    let test_script = r#"
function test_rectangle_geometry()
    local rect = {min_x = 10, max_x = 30, min_z = 20, max_z = 40}
    
    if not is_point_in_rectangle(20, 30, rect) then
        error("Point (20, 30) should be inside rectangle")
    end
    
    if is_point_in_rectangle(5, 30, rect) then
        error("Point (5, 30) should be outside rectangle")
    end
    
    if not is_point_in_rectangle(10, 20, rect) then
        error("Point (10, 20) on edge should be inside rectangle")
    end
end

test_rectangle_geometry()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, vec![]);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Rectangle geometry test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_circle() {
    let test_script = r#"
function test_circle_geometry()
    local circle = {center_x = 50, center_z = 30, radius = 10}
    
    if not is_point_in_circle(50, 30, circle) then
        error("Center point should be inside circle")
    end
    
    if not is_point_in_circle(55, 30, circle) then
        error("Point (55, 30) should be inside circle")
    end
    
    if is_point_in_circle(65, 30, circle) then
        error("Point (65, 30) should be outside circle")
    end
    
    if not is_point_in_circle(60, 30, circle) then
        error("Point on edge should be inside circle")
    end
end

test_circle_geometry()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, vec![]);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Circle geometry test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_arc() {
    let test_script = r#"
function test_arc_geometry()
    local arc = {center_x = 50, center_z = 30, radius = 10, start_angle = 0, end_angle = 90}
    
    if not is_point_in_arc(55, 35, arc) then
        error("Point (55, 35) should be inside arc (right angle and radius)")
    end
    
    if is_point_in_arc(45, 35, arc) then
        error("Point (45, 35) has wrong angle, should be outside arc")
    end
    
    if is_point_in_arc(65, 30, arc) then
        error("Point (65, 30) is too far, should be outside arc")
    end
end

test_arc_geometry()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, vec![]);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Arc geometry test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_penalty_area() {
    let zones = vec![
        Zone::new(
            "penalty_area",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 20.0, 16.5, 40.0)),
        ),
        Zone::new(
            "penalty_area",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(83.5, 20.0, 100.0, 40.0)),
        ),
    ];

    let test_script = r#"
function test_penalty_area()
    if not is_point_in_penalty_area(10, 30, "a") then
        error("Point (10, 30) should be in team A penalty area")
    end
    
    if not is_point_in_penalty_area(90, 30, "b") then
        error("Point (90, 30) should be in team B penalty area")
    end
    
    if is_point_in_penalty_area(50, 30, "a") then
        error("Point (50, 30) should not be in team A penalty area")
    end
end

test_penalty_area()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Penalty area test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_goal_area() {
    let zones = vec![
        Zone::new(
            "goal_area",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 28.0, 5.5, 32.0)),
        ),
        Zone::new(
            "goal_area",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(94.5, 28.0, 100.0, 32.0)),
        ),
    ];

    let test_script = r#"
function test_goal_area()
    if not is_point_in_goal_area(3, 30, "a") then
        error("Point (3, 30) should be in team A goal area")
    end
    
    if not is_point_in_goal_area(97, 30, "b") then
        error("Point (97, 30) should be in team B goal area")
    end
    
    if is_point_in_goal_area(50, 30, "a") then
        error("Point (50, 30) should not be in team A goal area")
    end
end

test_goal_area()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Goal area test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_half() {
    let zones = vec![
        Zone::new(
            "half",
            Some(Team::A),
            ZoneGeometry::Rectangle(Rectangle::from_meters(0.0, 0.0, 50.0, 60.0)),
        ),
        Zone::new(
            "half",
            Some(Team::B),
            ZoneGeometry::Rectangle(Rectangle::from_meters(50.0, 0.0, 100.0, 60.0)),
        ),
    ];

    let test_script = r#"
function test_half_field()
    if not is_point_in_half(25, 30, "a") then
        error("Point (25, 30) should be in team A half")
    end
    
    if not is_point_in_half(75, 30, "b") then
        error("Point (75, 30) should be in team B half")
    end
    
    if not is_point_in_half(50, 30, "a") then
        error("Point (50, 30) on midline should be in team A half")
    end
end

test_half_field()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Half field test failed: {:?}",
        player_state.last_error
    );
}

#[test]
fn test_is_point_in_center_circle() {
    let zones = vec![Zone::new(
        "center_circle",
        None,
        ZoneGeometry::Circle(Circle::from_meters(50.0, 30.0, 9.15)),
    )];

    let test_script = r#"
function test_center_circle()
    if not is_point_in_center_circle(50, 30) then
        error("Center point should be in center circle")
    end
    
    if not is_point_in_center_circle(55, 30) then
        error("Point (55, 30) should be inside center circle")
    end
    
    if is_point_in_center_circle(65, 30) then
        error("Point (65, 30) should be outside center circle")
    end
end

test_center_circle()
"#;

    let script = format!("{}{}", test_script, MAKE_DECISION_STUB);
    let mut game = create_test_game_with_preambles_and_zones(&script, zones);

    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0);

    let decision_maker =
        ScriptedDecisionMaker::new(&game).expect("Failed to create decision maker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let player_state = &game.state().player_states[0];
    assert!(
        player_state.last_error.is_none(),
        "Center circle test failed: {:?}",
        player_state.last_error
    );
}
