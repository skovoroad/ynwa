use super::*;
use crate::field::Field;
use crate::game::{BallDef, GameConfig, PlayerDef, RefereeDef};
use crate::region::GridCell;
use crate::team::Team;

fn create_test_game() -> Game {
    create_test_game_with_player_stats(100, 50, 50, 50, 50)
}

fn create_test_game_with_player_stats(
    reaction_rate: u32,
    speed_rate: u32,
    tackle_rate: u32,
    shot_power: u32,
    shot_accuracy: u32,
) -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(1, 1).unwrap()).unwrap();

    let players = vec![PlayerDef::new(
        Team::A,
        1,
        "Test Player".to_string(),
        "function make_decision() return {} end".to_string(),
        start_region,
    )
    .with_reaction_rate(reaction_rate)
    .with_speed_rate(speed_rate)
    .with_tackle_rate(tackle_rate)
    .with_shot_power(shot_power)
    .with_shot_accuracy(shot_accuracy)];

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::new(config)
}

fn create_test_game_with_two_players() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 11);
    let grid_dims = field.grid_dimensions();

    let start_region = grid_dims.create_region(GridCell::new(1, 1).unwrap(), GridCell::new(1, 1).unwrap()).unwrap();

    let players = vec![
        PlayerDef::new(
            Team::A,
            1,
            "Player 1".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region.clone(),
        ),
        PlayerDef::new(
            Team::A,
            2,
            "Player 2".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region,
        ),
    ];

    let config = GameConfig {
        field,
        players,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    };

    Game::new(config)
}

#[test]
fn test_action_system_processes_stop_decision() {
    let mut game = create_test_game();
    let mut system = ActionSystem::new();

    game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 3.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Stop);
    game.state.player_states[0].decision_processed = false;

    system.update(&mut game, 0.0);

    assert_eq!(
        game.state.player_states[0]
            .velocity
            .x
            .get::<meter_per_second>(),
        0.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .y
            .get::<meter_per_second>(),
        0.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .z
            .get::<meter_per_second>(),
        0.0
    );
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_action_system_processes_run_to_point_decision() {
    let mut game = create_test_game();
    let mut system = ActionSystem::new();

    let target = Point3D::from_meters(10.0, 0.0, 0.0);
    game.state.player_states[0].position = Point3D::from_meters(0.0, 0.0, 0.0);
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::Point(target)));
    game.state.player_states[0].decision_processed = false;

    system.update(&mut game, 0.0);

    assert!(
        game.state.player_states[0]
            .velocity
            .x
            .get::<meter_per_second>()
            > 0.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .y
            .get::<meter_per_second>(),
        0.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .z
            .get::<meter_per_second>(),
        0.0
    );
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_action_system_processes_run_to_cell_decision() {
    let mut game = create_test_game();
    let mut system = ActionSystem::new();

    let cell = GridCell::new(5, 5).unwrap();
    game.state.player_states[0].current_decision =
        Some(Decision::Run(DecisionTarget::GridCell(cell)));
    game.state.player_states[0].decision_processed = false;

    system.update(&mut game, 0.0);

    let velocity = &game.state.player_states[0].velocity;
    let vx = velocity.x.get::<meter_per_second>();
    let vy = velocity.y.get::<meter_per_second>();
    let vz = velocity.z.get::<meter_per_second>();
    let speed = (vx * vx + vy * vy + vz * vz).sqrt();

    assert!(speed > 0.0);
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_action_system_skips_processed_decisions() {
    let mut game = create_test_game();
    let mut system = ActionSystem::new();

    game.state.player_states[0].current_decision = Some(Decision::Stop);
    game.state.player_states[0].decision_processed = true;
    game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(5.0, 3.0, 0.0);

    system.update(&mut game, 0.0);

    assert_eq!(
        game.state.player_states[0]
            .velocity
            .x
            .get::<meter_per_second>(),
        5.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .y
            .get::<meter_per_second>(),
        3.0
    );
}

#[test]
fn test_action_system_skips_no_decision() {
    let mut game = create_test_game();
    let mut system = ActionSystem::new();

    game.state.player_states[0].current_decision = None;
    game.state.player_states[0].velocity = Velocity3D::from_meters_per_second(2.0, 1.0, 0.0);

    system.update(&mut game, 0.0);

    assert_eq!(
        game.state.player_states[0]
            .velocity
            .x
            .get::<meter_per_second>(),
        2.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .y
            .get::<meter_per_second>(),
        1.0
    );
}

#[test]
fn test_calculate_velocity_normalized() {
    let from = Point3D::from_meters(0.0, 0.0, 0.0);
    let to = Point3D::from_meters(10.0, 0.0, 0.0);

    let velocity = calculate_velocity(&from, &to, 50);

    let vx = velocity.x.get::<meter_per_second>();
    let vy = velocity.y.get::<meter_per_second>();
    let speed = (vx * vx + vy * vy).sqrt();
    assert!((speed - 5.0).abs() < 0.01);
}

#[test]
fn test_kick_with_perfect_accuracy_no_variation() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    // Set player as ball owner
    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);

    // Kick straight along X axis
    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    // Use RNG with no variation (0.5)
    let mut system = ActionSystem::with_rng(|| 0.5);
    system.update(&mut game, 0.0);

    // Check ball velocity
    let ball_vel = &game.state.ball_state.velocity;
    let vx = ball_vel.x.get::<meter_per_second>();
    let vz = ball_vel.z.get::<meter_per_second>();

    // shot_power=100, rng=0.5 → base velocity (no variation)
    // shot_accuracy=100, rng=0.5 → no deviation, straight along X
    let expected_velocity = 100.0 / KICK_POWER_DIVISOR;
    assert!((vx - expected_velocity).abs() < 0.01);
    assert!(vz.abs() < 0.01);

    // Possession should be released
    assert_eq!(game.state.ball_state.possessed_by, None);

    // Decision should be processed
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_kick_with_min_power_variation() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);

    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    // First call for power (0.0 = min), second for accuracy (0.5 = no deviation)
    use std::cell::Cell;
    let call_count = Cell::new(0);
    let mut system = ActionSystem::with_rng(move || {
        let count = call_count.get();
        call_count.set(count + 1);
        if count == 0 {
            0.0
        } else {
            0.5
        }
    });
    system.update(&mut game, 0.0);

    let ball_vel = &game.state.ball_state.velocity;
    let vx = ball_vel.x.get::<meter_per_second>();

    // shot_power=100, rng=0.0 → min variation
    let expected_velocity = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MIN;
    assert!((vx - expected_velocity).abs() < 0.01);
    assert_eq!(game.state.ball_state.possessed_by, None);
}

#[test]
fn test_kick_with_max_power_variation() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);

    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    // First call for power (1.0 = max), second for accuracy (0.5 = no deviation)
    use std::cell::Cell;
    let call_count = Cell::new(0);
    let mut system = ActionSystem::with_rng(move || {
        let count = call_count.get();
        call_count.set(count + 1);
        if count == 0 {
            1.0
        } else {
            0.5
        }
    });
    system.update(&mut game, 0.0);

    let ball_vel = &game.state.ball_state.velocity;
    let vx = ball_vel.x.get::<meter_per_second>();

    // shot_power=100, rng=1.0 → max variation
    let expected_velocity = (100.0 / KICK_POWER_DIVISOR) * KICK_POWER_VARIATION_MAX;
    assert!((vx - expected_velocity).abs() < 0.01);
    assert_eq!(game.state.ball_state.possessed_by, None);
}

#[test]
fn test_kick_with_poor_accuracy_max_deviation() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 10);

    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);

    let target = Point3D::from_meters(60.0, 30.0, 0.0); // Kick along X
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    // First call for power (0.5), second for accuracy (1.0 = max positive deviation)
    use std::cell::Cell;
    let call_count = Cell::new(0);
    let mut system = ActionSystem::with_rng(move || {
        let count = call_count.get();
        call_count.set(count + 1);
        if count == 0 {
            0.5
        } else {
            1.0
        }
    });
    system.update(&mut game, 0.0);

    let ball_vel = &game.state.ball_state.velocity;
    let vx = ball_vel.x.get::<meter_per_second>();
    let vz = ball_vel.z.get::<meter_per_second>();

    // shot_accuracy=10, rng=1.0 → +45 degrees deviation
    // 45° rotation: cos(45°)≈0.707, sin(45°)≈0.707
    let base_velocity = 100.0 / KICK_POWER_DIVISOR;
    let expected_vx = base_velocity * 0.707;
    let expected_vz = base_velocity * 0.707;
    assert!((vx - expected_vx).abs() < 0.1);
    assert!((vz - expected_vz).abs() < 0.1);
    assert_eq!(game.state.ball_state.possessed_by, None);
}

#[test]
fn test_kick_without_possession_ignored() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    // Player does NOT own the ball
    game.state.ball_state.possessed_by = None;
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
    game.state.ball_state.velocity = Velocity3D::from_meters_per_second(1.0, 0.0, 2.0);

    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    let mut system = ActionSystem::with_rng(|| 0.5);
    system.update(&mut game, 0.0);

    // Ball velocity should not change (kick ignored)
    let ball_vel = &game.state.ball_state.velocity;
    assert_eq!(ball_vel.x.get::<meter_per_second>(), 1.0);
    assert_eq!(ball_vel.z.get::<meter_per_second>(), 2.0);

    // Decision should still be marked as processed
    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_kick_by_different_player_ignored() {
    let mut game = create_test_game_with_two_players();

    // Ball owned by player 0
    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);
    game.state.ball_state.velocity = Velocity3D::default();

    // Player 1 tries to kick
    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[1].current_decision = Some(Decision::Kick(target));
    game.state.player_states[1].decision_processed = false;

    let mut system = ActionSystem::with_rng(|| 0.5);
    system.update(&mut game, 0.0);

    // Ball velocity should not change (player 1 doesn't own ball)
    let ball_vel = &game.state.ball_state.velocity;
    assert_eq!(ball_vel.x.get::<meter_per_second>(), 0.0);
    assert_eq!(ball_vel.z.get::<meter_per_second>(), 0.0);

    // Possession should remain with player 0
    assert_eq!(game.state.ball_state.possessed_by, Some(0));

    // Decision should still be marked as processed
    assert!(game.state.player_states[1].decision_processed);
}

#[test]
fn test_kick_preserves_player_velocity() {
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    // Set player as ball owner
    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 30.0, 0.0);

    // Give player some velocity (running)
    let player_velocity = Velocity3D::from_meters_per_second(3.0, 0.0, 2.0);
    game.state.player_states[0].velocity = player_velocity;

    // Player kicks while running
    let target = Point3D::from_meters(60.0, 30.0, 0.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    let mut system = ActionSystem::with_rng(|| 0.5);
    system.update(&mut game, 0.0);

    // Ball should move
    let ball_vel = &game.state.ball_state.velocity;
    assert!(ball_vel.x.get::<meter_per_second>() > 0.0);

    // Possession should be released
    assert_eq!(game.state.ball_state.possessed_by, None);

    // Player velocity should be UNCHANGED (continues running)
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .x
            .get::<meter_per_second>(),
        3.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .y
            .get::<meter_per_second>(),
        0.0
    );
    assert_eq!(
        game.state.player_states[0]
            .velocity
            .z
            .get::<meter_per_second>(),
        2.0
    );

    assert!(game.state.player_states[0].decision_processed);
}

#[test]
fn test_kick_resets_possession_cooldown_timer() {
    // Regression test: after a kick, last_possession_change_time must be updated to the
    // current timestamp so that BallPossessionSystem's cooldown prevents the kicker from
    // immediately re-acquiring the ball on the next tick.
    let mut game = create_test_game_with_player_stats(100, 50, 50, 100, 100);

    game.state.ball_state.possessed_by = Some(0);
    game.state.ball_state.position = Point3D::from_meters(50.0, 0.0, 30.0);
    // Simulate an old possession change time (well before kick)
    game.state.ball_state.last_possession_change_time = 0.0;

    let target = Point3D::from_meters(50.0, 0.0, 60.0);
    game.state.player_states[0].current_decision = Some(Decision::Kick(target));
    game.state.player_states[0].decision_processed = false;

    let kick_timestamp = 5.0_f32;
    let mut system = ActionSystem::with_rng(|| 0.5);
    system.update(&mut game, kick_timestamp);

    // Possession released
    assert_eq!(game.state.ball_state.possessed_by, None);

    // Cooldown timer must equal the kick timestamp so BallPossessionSystem
    // won't re-assign the ball until at least POSSESSION_COOLDOWN seconds later.
    assert_eq!(
        game.state.ball_state.last_possession_change_time,
        kick_timestamp,
        "last_possession_change_time must be updated on kick to prevent immediate re-possession"
    );
}
