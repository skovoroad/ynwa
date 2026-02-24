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

// --- Dispatch table tests ---

fn run_dispatch_test(player_script: &str, stage: ynwa_core::game::GameStage) -> ynwa_core::game::PlayerState {
    use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
    use ynwa_core::System;
    let mut game = create_test_game_with_full_preambles_and_stage(player_script, stage);
    request_decisions_for_all(&mut game);
    let decision_maker = ScriptedDecisionMaker::new(&game).expect("ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);
    game.state().player_states[0].clone()
}

#[test]
fn test_dispatch_team_has_ball_runs_to_attack() {
    // Ball owned by a teammate (team_has_ball state) → team_play handler → Run decision
    let state = run_dispatch_test("", GameStage::Play);
    assert!(
        state.last_error.is_none(),
        "dispatch error: {:?}",
        state.last_error
    );
    assert!(
        matches!(state.current_decision, Some(Decision::Run(_))),
        "Expected Run, got: {:?}",
        state.current_decision
    );
}

#[test]
fn test_dispatch_player_overrides_team() {
    // player_play takes priority over team_play
    // player_play.team_has_ball returns Stop, team_play.team_has_ball would return Run
    let player_script = r#"
player_play = {
    i_have_ball       = function() return {action = "stop"} end,
    ball_is_free      = function() return {action = "stop"} end,
    team_has_ball     = function() return {action = "stop"} end,
    opponent_has_ball = function() return {action = "stop"} end,
}
"#;
    let state = run_dispatch_test(player_script, GameStage::Play);
    assert!(
        state.last_error.is_none(),
        "dispatch error: {:?}",
        state.last_error
    );
    assert!(
        matches!(state.current_decision, Some(Decision::Stop)),
        "Expected Stop (player override), got: {:?}",
        state.current_decision
    );
}

#[test]
fn test_setup_dispatch_by_reason() {
    // team_setup.start → run_to_start_position → Run
    let state = run_dispatch_test("", GameStage::Setup("start".to_string()));
    assert!(
        state.last_error.is_none(),
        "setup dispatch error: {:?}",
        state.last_error
    );
    assert!(
        matches!(state.current_decision, Some(Decision::Run(_))),
        "Expected Run from setup dispatch, got: {:?}",
        state.current_decision
    );
}

#[test]
fn test_setup_fallback_to_default() {
    // Unknown reason → falls through to default_get_setup_position → Run
    let state = run_dispatch_test("", GameStage::Setup("throw_in".to_string()));
    assert!(
        state.last_error.is_none(),
        "setup fallback error: {:?}",
        state.last_error
    );
    assert!(
        matches!(state.current_decision, Some(Decision::Run(_))),
        "Expected Run from fallback, got: {:?}",
        state.current_decision
    );
}

#[test]
fn test_is_in_region_true() {
    // Field: 100w × 60l, 26 cols × 44 rows → cell_w = 100/26 ≈ 3.846, cell_h = 60/44 ≈ 1.364
    // Player placed at (50, 0, 30) by default. Region M20:N25 should contain it.
    // col M=13, row 20: min_x=(13-1)*3.846=46.15, max_x=14*3.846=53.85
    //                   min_z=(20-1)*1.364=25.91,  max_z=25*1.364=34.09
    let script = r#"
function make_decision()
    if is_in_region("M20", "N25") then
        return {action = "stop"}
    else
        return {action = "kick", target = {x = 0, z = 0}}
    end
end
"#;
    let mut game = create_test_game_with_preambles(script);
    game.state.player_states[0].position = ynwa_core::field::zones::Point3D::from_meters(50.0, 0.0, 30.0);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        matches!(game.state().player_states[0].current_decision, Some(Decision::Stop)),
        "Expected Stop (inside region), got: {:?}",
        game.state().player_states[0].current_decision
    );
}

#[test]
fn test_is_in_region_false() {
    // Same field. Position (1.0, 0, 1.0) is in A1 — outside M20:N25.
    let script = r#"
function make_decision()
    if is_in_region("M20", "N25") then
        return {action = "stop"}
    else
        return {action = "kick", target = {x = 0, z = 0}}
    end
end
"#;
    let mut game = create_test_game_with_preambles(script);
    game.state.player_states[0].position = ynwa_core::field::zones::Point3D::from_meters(1.0, 0.0, 1.0);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        matches!(game.state().player_states[0].current_decision, Some(Decision::Kick(_))),
        "Expected Kick (outside region), got: {:?}",
        game.state().player_states[0].current_decision
    );
}

#[test]
fn test_run_to_region() {
    let script = r#"
function make_decision()
    return run_to_region("M20", "N25")
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        matches!(game.state().player_states[0].current_decision, Some(Decision::Run(_))),
        "Expected Run, got: {:?}",
        game.state().player_states[0].current_decision
    );
}

#[test]
fn test_parse_col() {
    let script = format!(
        r#"
assert(parse_col("A") == 1,  "A must be 1")
assert(parse_col("Z") == 26, "Z must be 26")
assert(parse_col("a") == 1,  "lowercase a must be 1")
assert(parse_col("AA") == 27, "AA must be 27")
assert(parse_col("AZ") == 52, "AZ must be 52")
{}
"#,
        MAKE_DECISION_STUB
    );
    let mut game = create_test_game_with_preambles(&script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(game.state().player_states[0].last_error.is_none());
}

// ── is_in_region: boundary and edge cases ────────────────────────────────────

// Helper: place player at (x, z) and check is_in_region("M20", "N25")
// Field: 100w × 60l, 26×44 → cell_w=100/26≈3.846, cell_h=60/44≈1.364
// M20:N25 → min_x=12·cw, max_x=14·cw, min_z=19·ch, max_z=25·ch
fn check_m20_n25(x: f32, z: f32) -> Option<Decision> {
    use ynwa_core::field::zones::Point3D;
    let script = r#"
function make_decision()
    if is_in_region("M20", "N25") then
        return {action = "stop"}
    else
        return {action = "kick", target = {x = 0, z = 0}}
    end
end
"#;
    let mut game = create_test_game_with_preambles(script);
    game.state.player_states[0].position = Point3D::from_meters(x, 0.0, z);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    game.state().player_states[0].current_decision.clone()
}

#[test]
fn test_is_in_region_min_boundary_inclusive() {
    let cell_w = 100.0_f32 / 26.0;
    let cell_h = 60.0_f32 / 44.0;
    // Exactly at min_x, min_z — must be inside (inclusive)
    assert!(matches!(check_m20_n25(12.0 * cell_w, 19.0 * cell_h), Some(Decision::Stop)));
}

#[test]
fn test_is_in_region_max_boundary_exclusive() {
    let cell_w = 100.0_f32 / 26.0;
    let cell_h = 60.0_f32 / 44.0;
    // One epsilon past max_x and max_z — must be outside regardless of float precision
    assert!(matches!(check_m20_n25(14.0 * cell_w + 0.001, 25.0 * cell_h + 0.001), Some(Decision::Kick(_))));
}

#[test]
fn test_is_in_region_single_cell() {
    use ynwa_core::field::zones::Point3D;
    let cell_w = 100.0_f32 / 26.0;
    let cell_h = 60.0_f32 / 44.0;
    let script = r#"
function make_decision()
    if is_in_region("A1", "A1") then
        return {action = "stop"}
    else
        return {action = "kick", target = {x = 0, z = 0}}
    end
end
"#;
    let mut game = create_test_game_with_preambles(script);
    // Center of A1
    game.state.player_states[0].position = Point3D::from_meters(0.5 * cell_w, 0.0, 0.5 * cell_h);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(matches!(
        game.state().player_states[0].current_decision,
        Some(Decision::Stop)
    ));
}

#[test]
fn test_is_in_region_invalid_notation_causes_error() {
    let script = r#"
function make_decision()
    is_in_region("1A", "Z44")
    return {action = "stop"}
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(game.state().player_states[0].last_error.is_some());
}

#[test]
fn test_parse_notation_invalid_causes_error() {
    let script = format!(
        r#"
local ok, err = pcall(parse_notation, "1A")
assert(not ok, "parse_notation('1A') must error")
local ok2, err2 = pcall(parse_notation, "")
assert(not ok2, "parse_notation('') must error")
{}
"#,
        MAKE_DECISION_STUB
    );
    let mut game = create_test_game_with_preambles(&script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(game.state().player_states[0].last_error.is_none());
}#[test]
fn test_parse_notation() {
    let script = format!(
        r#"
local col, row = parse_notation("A1")
assert(col == 1 and row == 1, "A1: col=1, row=1")

local col2, row2 = parse_notation("Z44")
assert(col2 == 26 and row2 == 44, "Z44: col=26, row=44")

local col3, row3 = parse_notation("m22")
assert(col3 == 13 and row3 == 22, "m22: col=13, row=22")
{}
"#,
        MAKE_DECISION_STUB
    );
    let mut game = create_test_game_with_preambles(&script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(game.state().player_states[0].last_error.is_none());
}
