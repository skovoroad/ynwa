// Integration tests: verify stdlib functions

use ynwa_core::game::{Decision, GameStage, REGION_START_POSITION};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::System;
use ynwa_script_tests::{
    create_test_game_football_field_with_preambles, create_test_game_with_all_preambles,
    create_test_game_with_full_preambles_and_stage, create_test_game_with_preambles,
    request_decisions_for_all,
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
fn test_kick_to_opponent_goal() {
    // kick_to_opponent_goal() must aim at the center of the opponent goal zone.
    // We derive expected coordinates from the actual field, not from hardcoded constants,
    // so the test stays valid if field dimensions change.
    let script = r#"
        function make_decision()
            return kick_to_opponent_goal()
        end
    "#;
    let mut game = create_test_game_football_field_with_preambles(script);
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

fn run_dispatch_test(
    player_script: &str,
    stage: ynwa_core::game::GameStage,
) -> ynwa_core::game::PlayerState {
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
    game.state.player_states[0].position =
        ynwa_core::field::zones::Point3D::from_meters(50.0, 0.0, 30.0);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        matches!(
            game.state().player_states[0].current_decision,
            Some(Decision::Stop)
        ),
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
    game.state.player_states[0].position =
        ynwa_core::field::zones::Point3D::from_meters(1.0, 0.0, 1.0);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        matches!(
            game.state().player_states[0].current_decision,
            Some(Decision::Kick(_))
        ),
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
        matches!(
            game.state().player_states[0].current_decision,
            Some(Decision::Run(_))
        ),
        "Expected Run, got: {:?}",
        game.state().player_states[0].current_decision
    );
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
    assert!(matches!(
        check_m20_n25(12.0 * cell_w, 19.0 * cell_h),
        Some(Decision::Stop)
    ));
}

#[test]
fn test_is_in_region_max_boundary_exclusive() {
    let cell_w = 100.0_f32 / 26.0;
    let cell_h = 60.0_f32 / 44.0;
    // One epsilon past max_x and max_z — must be outside regardless of float precision
    assert!(matches!(
        check_m20_n25(14.0 * cell_w + 0.001, 25.0 * cell_h + 0.001),
        Some(Decision::Kick(_))
    ));
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
fn test_region_notation_in_context() {
    // Regions carry a "display_notation" field.
    // Team A: plain notation, e.g. "J10:K11" or "M3" (single cell).
    // Team B: "display (team)" format, e.g. "R42 (M3)".
    // The test game uses Team A, so display_notation is plain.
    let script = r#"
function make_decision()
    local r = my_regions()["start"]
    if not r then error("no start position") end
    if not r.display_notation then error("display_notation field missing") end
    if type(r.display_notation) ~= "string" then error("display_notation must be a string") end
    return {action = "stop"}
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        game.state().player_states[0].last_error.is_none(),
        "region notation test failed: {:?}",
        game.state().player_states[0].last_error
    );
}

// --- get_own_goal / default_goalkeeper_cover_position ---

#[test]
fn test_get_own_goal_team_a() {
    // Team A own goal is goal_a (small Z). Assert min_z < field_length/2.
    let script = r#"
function make_decision()
    local g = get_own_goal()
    assert(g ~= nil, "get_own_goal() returned nil")
    assert(g.min_z < GAME_DATA.field.length / 2, "team A own goal must be at low Z end")
    return {action = "stop"}
end
"#;
    let mut game = create_test_game_football_field_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        game.state().player_states[0].last_error.is_none(),
        "{:?}",
        game.state().player_states[0].last_error
    );
}

#[test]
fn test_get_own_goal_team_b() {
    // Team B zones are pre-flipped: goal_b in their view also has small Z (near their goal line).
    // Also verify get_own_goal() != get_opponent_goal() — they return different zones.
    let script = r#"
function make_decision()
    local own  = get_own_goal()
    local opp  = get_opponent_goal()
    assert(own ~= nil, "get_own_goal() returned nil")
    assert(opp ~= nil, "get_opponent_goal() returned nil")
    assert(own.min_z < GAME_DATA.field.length / 2,
        "team B own goal must be at low Z in their view, got " .. own.min_z)
    assert(opp.min_z > GAME_DATA.field.length / 2,
        "team B opponent goal must be at high Z in their view, got " .. opp.min_z)
    -- own and opponent goals must be at different Z positions
    assert(math.abs(own.min_z - opp.min_z) > 1.0, "own and opponent goals must differ")
    return {action = "stop"}
end
"#;
    use ynwa_core::region::GridCell;

    let field = ynwa_football::field_builder::create_football_field();
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    let player = ynwa_core::game::PlayerDef::new(
        ynwa_core::team::Team::B,
        1,
        "GK B".to_string(),
        script.to_string(),
        std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    );
    let mut game = create_test_game_with_all_preambles(vec![player]);
    game.state.player_states[0].needs_decision = true;
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        game.state().player_states[0].last_error.is_none(),
        "{:?}",
        game.state().player_states[0].last_error
    );
}

#[test]
fn test_goalkeeper_cover_position_clamps_to_goal() {
    // default_goalkeeper_cover_position: target X is clamped to own goal width, Z is defence position Z.
    // Ball at X=0 (far left) → target X == goal.min_x; ball at X=field.width (far right) → goal.max_x.
    let script = r#"
function make_decision()
    local goal = get_own_goal()
    local defence = my_regions()["defence"]
    local defence_z = (defence.min_z + defence.max_z) / 2

    -- Ball at extreme left: target X must clamp to goal.min_x
    context.ball.position.x = -999
    local d = default_goalkeeper_cover_position()
    assert(d.action == "run", "expected run")
    assert(math.abs(d.target.x - goal.min_x) < 0.01,
        "left clamp: expected " .. goal.min_x .. " got " .. d.target.x)
    assert(math.abs(d.target.z - defence_z) < 0.01,
        "Z must equal defence Z")

    -- Ball at extreme right: target X must clamp to goal.max_x
    context.ball.position.x = 999
    d = default_goalkeeper_cover_position()
    assert(math.abs(d.target.x - goal.max_x) < 0.01,
        "right clamp: expected " .. goal.max_x .. " got " .. d.target.x)

    -- Ball at goal center: target X equals ball X
    local center_x = (goal.min_x + goal.max_x) / 2
    context.ball.position.x = center_x
    d = default_goalkeeper_cover_position()
    assert(math.abs(d.target.x - center_x) < 0.01,
        "center: expected " .. center_x .. " got " .. d.target.x)

    return {action = "stop"}
end
"#;
    use ynwa_core::field::zones::Point3D;
    use ynwa_core::region::GridCell;

    let field = ynwa_football::field_builder::create_football_field();
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    // defence position: row 1 (goal line area)
    let defence_region = grid_dims
        .create_region(GridCell::new(13, 1).unwrap(), GridCell::new(14, 2).unwrap())
        .unwrap();

    let player = ynwa_core::game::PlayerDef::new(
        ynwa_core::team::Team::A,
        1,
        "GK".to_string(),
        script.to_string(),
        {
            let mut r = std::collections::HashMap::new();
            r.insert(REGION_START_POSITION.to_string(), start_region);
            r.insert("defence".to_string(), defence_region);
            r
        },
    );

    let mut game = create_test_game_with_all_preambles(vec![player]);
    game.state.player_states[0].position = Point3D::from_meters(5.0, 0.0, 5.0);
    game.state.player_states[0].needs_decision = true;
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        game.state().player_states[0].last_error.is_none(),
        "{:?}",
        game.state().player_states[0].last_error
    );
}

// --- Multi-player helpers ---

fn make_player(
    team: ynwa_core::team::Team,
    number: u32,
    script: &str,
) -> ynwa_core::game::PlayerDef {
    use ynwa_core::region::GridCell;
    let field = ynwa_football::field_builder::create_football_field();
    let start_region = field
        .grid_dimensions()
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    ynwa_core::game::PlayerDef::new(
        team,
        number,
        format!("Player {number}"),
        script.to_string(),
        std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
}

fn place_player(game: &mut ynwa_core::game::Game, index: usize, x: f32, z: f32) {
    game.state.player_states[index].position =
        ynwa_core::field::zones::Point3D::from_meters(x, 0.0, z);
}

/// Runs only player 0's script and returns its state.
fn run_caller(mut game: ynwa_core::game::Game) -> ynwa_core::game::PlayerState {
    for state in game.state.player_states.iter_mut() {
        state.needs_decision = false;
    }
    game.state.player_states[0].needs_decision = true;
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    game.state().player_states[0].clone()
}

// --- count_teammates_around / count_opponents_around ---

#[test]
fn test_count_players_around() {
    let script = r#"
function make_decision()
    local pos = {x = 10, z = 10}
    assert(count_teammates_around(pos, 5.0) == 2,
        "strict radius includes me and excludes a teammate at exactly 5 m")
    assert(count_teammates_around(pos, 10.0) == 3, "all teammates within a larger radius")
    assert(count_opponents_around(pos, 5.0) == 1, "an opponent at exactly 5 m is excluded")
    assert(count_opponents_around(pos, 10.001) == 2, "both opponents within a larger radius")
    return {action = "stop"}
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::A, 3, ""),
        make_player(Team::B, 1, ""),
        make_player(Team::B, 2, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 10.0, 10.0);
    place_player(&mut game, 1, 13.0, 10.0);
    place_player(&mut game, 2, 15.0, 10.0);
    place_player(&mut game, 3, 10.0, 14.0);
    place_player(&mut game, 4, 10.0, 20.0);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

// --- teammate_position / opponent_position ---

#[test]
fn test_position_lookups_and_unknown_numbers() {
    let script = r#"
function make_decision()
    local tm = teammate_position(2)
    assert(math.abs(tm.x - 30) < 0.01 and math.abs(tm.z - 30) < 0.01, "teammate_position(2)")
    local opp = opponent_position(1)
    assert(math.abs(opp.x - 10) < 0.01 and math.abs(opp.z - 50) < 0.01, "opponent_position(1)")
    assert(not pcall(teammate_position, 99), "unknown teammate number must raise")
    assert(not pcall(opponent_position, 99), "unknown opponent number must raise")
    return {action = "stop"}
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::B, 1, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 1, 30.0, 30.0);
    place_player(&mut game, 2, 10.0, 50.0);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

// --- ball_owner_number ---

#[test]
fn test_ball_owner_number_for_teammate() {
    use ynwa_core::team::Team;
    let script = r#"
function make_decision()
    assert(ball_owner_number() == 7, "owner number must be 7, got " .. tostring(ball_owner_number()))
    return {action = "stop"}
end
"#;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 7, ""),
        make_player(Team::B, 3, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    game.state.ball_state.possessed_by = Some(1);
    game.state.ball_state.last_possessing_team = Some(Team::A);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

#[test]
fn test_ball_owner_number_for_opponent() {
    use ynwa_core::team::Team;
    let script = r#"
function make_decision()
    assert(ball_owner_number() == 3, "owner number must be 3, got " .. tostring(ball_owner_number()))
    return {action = "stop"}
end
"#;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 7, ""),
        make_player(Team::B, 3, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    game.state.ball_state.possessed_by = Some(2);
    game.state.ball_state.last_possessing_team = Some(Team::B);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

#[test]
fn test_ball_owner_number_is_nil_for_in_flight_pass() {
    use ynwa_core::team::Team;
    let script = r#"
function make_decision()
    assert(ball_owner_number() == nil, "in-flight pass has no owner number")
    assert(get_ball_owner_team() == "A", "passing team is still reported")
    return {action = "stop"}
end
"#;
    let players = vec![make_player(Team::A, 1, script), make_player(Team::A, 7, "")];
    let mut game = create_test_game_with_all_preambles(players);
    game.state.ball_state.possessed_by = None;
    game.state.ball_state.last_possessing_team = Some(Team::A);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

// --- free player predicates ---

#[test]
fn test_is_teammate_free() {
    let script = r#"
function make_decision()
    assert(not is_teammate_free(2), "an opponent 4.9 m away marks the teammate as covered")
    assert(is_teammate_free(3), "an opponent at exactly 5 m leaves the teammate free")
    return {action = "stop"}
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::A, 3, ""),
        make_player(Team::B, 1, ""),
        make_player(Team::B, 2, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 0.0, 0.0);
    place_player(&mut game, 1, 20.0, 20.0);
    place_player(&mut game, 2, 40.0, 40.0);
    place_player(&mut game, 3, 24.9, 20.0);
    place_player(&mut game, 4, 45.0, 40.0);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

#[test]
fn test_is_opponent_free() {
    let script = r#"
function make_decision()
    assert(not is_opponent_free(1), "a teammate 4.9 m away marks the opponent as covered")
    assert(is_opponent_free(2), "a teammate at exactly 5 m leaves the opponent free")
    return {action = "stop"}
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::A, 3, ""),
        make_player(Team::B, 1, ""),
        make_player(Team::B, 2, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 0.0, 0.0);
    place_player(&mut game, 1, 24.9, 20.0);
    place_player(&mut game, 2, 45.0, 40.0);
    place_player(&mut game, 3, 20.0, 20.0);
    place_player(&mut game, 4, 40.0, 40.0);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

// --- directional predicates ---

#[test]
fn test_directional_predicates() {
    let script = r#"
function make_decision()
    assert(is_teammate_ahead(2), "teammate 2 is ahead")
    assert(is_teammate_behind(3), "teammate 3 is behind")
    assert(is_teammate_left(4), "teammate 4 is left")
    assert(is_teammate_right(5), "teammate 5 is right")
    assert(not is_teammate_ahead(6), "an offset of exactly 0.5 m is not ahead")
    assert(is_opponent_ahead(1), "opponent 1 is ahead")
    assert(is_opponent_behind(2), "opponent 2 is behind")
    assert(is_opponent_left(3), "opponent 3 is left")
    assert(is_opponent_right(4), "opponent 4 is right")

    -- Ball predicates have no tolerance: a 0.1 m offset is enough to decide.
    local my_z = my_position().z
    context.ball.position.z = my_z + 0.1
    assert(is_ball_ahead(), "ball above my Z is ahead")
    assert(not is_ball_behind(), "ball above my Z is not behind")
    context.ball.position.z = my_z - 0.1
    assert(is_ball_behind(), "ball below my Z is behind")
    assert(not is_ball_ahead(), "ball below my Z is not ahead")
    context.ball.position.z = my_z
    assert(not is_ball_ahead() and not is_ball_behind(), "ball level with me is neither")
    return {action = "stop"}
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::A, 3, ""),
        make_player(Team::A, 4, ""),
        make_player(Team::A, 5, ""),
        make_player(Team::A, 6, ""),
        make_player(Team::B, 1, ""),
        make_player(Team::B, 2, ""),
        make_player(Team::B, 3, ""),
        make_player(Team::B, 4, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 10.0, 10.0);
    place_player(&mut game, 1, 10.0, 11.0);
    place_player(&mut game, 2, 10.0, 9.0);
    place_player(&mut game, 3, 9.0, 10.0);
    place_player(&mut game, 4, 11.0, 10.0);
    place_player(&mut game, 5, 10.0, 10.5);
    place_player(&mut game, 6, 10.0, 11.0);
    place_player(&mut game, 7, 10.0, 9.0);
    place_player(&mut game, 8, 9.0, 10.0);
    place_player(&mut game, 9, 11.0, 10.0);
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
}

// --- kick_to_position ---

#[test]
fn test_kick_to_position_default_reason() {
    let script = r#"
function make_decision()
    return kick_to_position({x = 12.5, z = 34.5})
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    let Some(Decision::Kick(target)) = &state.current_decision else {
        panic!("Expected Kick, got: {:?}", state.current_decision);
    };
    use uom::si::length::meter;
    assert!((target.x.get::<meter>() - 12.5).abs() < 0.01);
    assert!((target.z.get::<meter>() - 34.5).abs() < 0.01);
    assert_eq!(
        state.decision_reason.as_deref(),
        Some("kick_to_position(12.5,34.5)")
    );
}

#[test]
fn test_kick_to_position_explicit_reason() {
    let script = r#"
function make_decision()
    return kick_to_position({x = 1.0, z = 2.0}, "pass_to_#2")
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert!(matches!(state.current_decision, Some(Decision::Kick(_))));
    assert_eq!(state.decision_reason.as_deref(), Some("pass_to_#2"));
}

// --- run_to_start_position ---

#[test]
fn test_run_to_start_position() {
    let script = r#"
function make_decision()
    return run_to_start_position()
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert!(matches!(state.current_decision, Some(Decision::Run(_))));
    assert_eq!(
        state.decision_reason.as_deref(),
        Some("run_to_start_position")
    );
}

#[test]
fn test_run_to_start_position_without_region_stops() {
    let script = r#"
function make_decision()
    context.me.regions["start"] = nil
    return run_to_start_position()
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert!(matches!(state.current_decision, Some(Decision::Stop)));
    assert_eq!(state.decision_reason.as_deref(), Some("no_start_position"));
}

// --- default_goalkeeper_with_ball ---

#[test]
fn test_default_goalkeeper_with_ball_passes_to_nearest_eligible_teammate() {
    let script = r#"
function make_decision()
    return default_goalkeeper_with_ball()
end
"#;
    use ynwa_core::team::Team;
    let players = vec![
        make_player(Team::A, 1, script),
        make_player(Team::A, 2, ""),
        make_player(Team::A, 3, ""),
        make_player(Team::A, 4, ""),
    ];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 10.0, 10.0);
    place_player(&mut game, 1, 10.0, 30.0); // 20 m
    place_player(&mut game, 2, 10.0, 26.0); // 16 m, nearest eligible
    place_player(&mut game, 3, 10.0, 20.0); // 10 m, too close
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    let Some(Decision::Kick(target)) = &state.current_decision else {
        panic!("Expected Kick, got: {:?}", state.current_decision);
    };
    use uom::si::length::meter;
    assert!((target.z.get::<meter>() - 26.0).abs() < 0.01);
    assert_eq!(state.decision_reason.as_deref(), Some("pass_to_#3"));
}

#[test]
fn test_default_goalkeeper_with_ball_falls_back_to_goal() {
    let script = r#"
function make_decision()
    return default_goalkeeper_with_ball()
end
"#;
    use ynwa_core::team::Team;
    let players = vec![make_player(Team::A, 1, script), make_player(Team::A, 2, "")];
    let mut game = create_test_game_with_all_preambles(players);
    place_player(&mut game, 0, 10.0, 10.0);
    place_player(&mut game, 1, 10.0, 20.0); // 10 m, below the 15 m threshold
    let state = run_caller(game);
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert!(matches!(state.current_decision, Some(Decision::Kick(_))));
    assert!(
        state
            .decision_reason
            .as_deref()
            .unwrap_or("")
            .starts_with("kick_to_goal("),
        "expected a goal kick, got {:?}",
        state.decision_reason
    );
}

// --- is_in_opponent_penalty_area (covers the internal is_in_region_obj) ---

fn check_opponent_penalty_area(x: f32, z: f32) -> Option<Decision> {
    use ynwa_core::field::zones::Point3D;
    let script = r#"
function make_decision()
    if is_in_opponent_penalty_area() then
        return {action = "stop"}
    else
        return {action = "kick", target = {x = 0, z = 0}}
    end
end
"#;
    let mut game = create_test_game_football_field_with_preambles(script);
    game.state.player_states[0].position = Point3D::from_meters(x, 0.0, z);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    game.state().player_states[0].current_decision.clone()
}

#[test]
fn test_is_in_opponent_penalty_area() {
    use uom::si::length::meter;
    use ynwa_core::field::zones::ZoneGeometry;
    use ynwa_core::team::Team;

    let field = ynwa_football::field_builder::create_football_field();
    let zone = field
        .get_zone("penalty_area", Some(Team::B))
        .expect("penalty_area_b must exist");
    let ZoneGeometry::Rectangle(ref rect) = zone.geometry else {
        panic!("penalty area must be a Rectangle");
    };
    let center_x = (rect.min.x.get::<meter>() + rect.max.x.get::<meter>()) / 2.0;
    let center_z = (rect.min.z.get::<meter>() + rect.max.z.get::<meter>()) / 2.0;

    assert!(matches!(
        check_opponent_penalty_area(center_x, center_z),
        Some(Decision::Stop)
    ));
    assert!(matches!(
        check_opponent_penalty_area(0.1, 0.1),
        Some(Decision::Kick(_))
    ));
}

// --- internal and removed helpers are not visible in the player scope ---

#[test]
fn test_internal_and_removed_helpers_are_nil_in_player_scope() {
    let script = r#"
function make_decision()
    assert(parse_col == nil, "parse_col must be internal")
    assert(parse_notation == nil, "parse_notation must be internal")
    assert(is_in_region_obj == nil, "is_in_region_obj must be internal")
    assert(run_to_region_obj == nil, "run_to_region_obj must be internal")
    assert(get_teammate_by_number == nil, "get_teammate_by_number must be internal")
    assert(am_i_ball_owner == nil, "am_i_ball_owner must be internal")
    assert(pass_to_teammate == nil, "pass_to_teammate must be removed")
    assert(pass_to_players_by_numbers == nil, "pass_to_players_by_numbers must be removed")
    assert(kick_to_cell == nil, "kick_to_cell must be removed")
    assert(kick_to_region == nil, "kick_to_region must be removed")
    return {action = "stop"}
end
"#;
    let mut game = create_test_game_with_preambles(script);
    request_decisions_for_all(&mut game);
    let dm = ScriptedDecisionMaker::new(&game).unwrap();
    let mut ds = DecisionSystem::new().with_decision_maker(Box::new(dm));
    ds.update(&mut game, 1.0);
    assert!(
        game.state().player_states[0].last_error.is_none(),
        "{:?}",
        game.state().player_states[0].last_error
    );
}
