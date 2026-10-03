use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::game::{
    BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef, ScriptingConfig,
    REGION_START_POSITION,
};
use crate::region::{GridCell, Region};
use crate::snapshot::{Score, Snapshot, SnapshotBall, SnapshotPlayer, SnapshotSetup};
use crate::team::Team;
use crate::test_utils::deterministic_rng;
use std::collections::HashMap;

fn start_region(field: &Field, column: u32) -> Region {
    field
        .grid_dimensions()
        .create_region(
            GridCell::new(column, 8).unwrap(),
            GridCell::new(column, 8).unwrap(),
        )
        .unwrap()
}

/// Three players (two of team A, one of team B) so indices above zero and both teams are covered.
fn game() -> Game {
    let field = Field::from_meters(100.0, 60.0, 26, 16);
    let make_player = |team: Team, number: u32, column: u32| {
        PlayerDef::new(
            team,
            number,
            format!("{:?}{}", team, number),
            String::new(),
            HashMap::from([(
                REGION_START_POSITION.to_string(),
                start_region(&field, column),
            )]),
        )
    };
    let players = vec![
        make_player(Team::A, 1, 3),
        make_player(Team::A, 2, 5),
        make_player(Team::B, 1, 20),
    ];
    let config = GameConfig {
        field,
        players,
        ball: BallDef {
            initial_position: Point3D::from_meters(10.0, 0.0, 20.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    };
    Game::with_stage(config, GameStage::Play, deterministic_rng())
}

#[test]
fn applies_stage() {
    let mut game = game();
    let snapshot = Snapshot {
        stage: Some(GameStage::Setup("throw in".to_string())),
        ..Default::default()
    };

    game.apply_snapshot(&snapshot).unwrap();

    assert_eq!(game.state.stage, GameStage::Setup("throw in".to_string()));
}

#[test]
fn applies_ball_fields() {
    let mut game = game();
    let position = Point3D::from_meters(1.0, 2.0, 3.0);
    let velocity = Velocity3D::from_meters_per_second(4.0, 5.0, 6.0);
    let snapshot = Snapshot {
        ball: Some(SnapshotBall {
            position: Some(position),
            velocity: Some(velocity),
            possessed_by: Some(2),
            last_possessing_team: Some(Team::B),
        }),
        ..Default::default()
    };

    game.apply_snapshot(&snapshot).unwrap();

    assert_eq!(game.state.ball_state.position, position);
    assert_eq!(game.state.ball_state.velocity, velocity);
    assert_eq!(game.state.ball_state.possessed_by, Some(2));
    assert_eq!(game.state.ball_state.last_possessing_team, Some(Team::B));
}

#[test]
fn applies_player_positions() {
    let mut game = game();
    let first = Point3D::from_meters(1.0, 0.0, 1.0);
    let third = Point3D::from_meters(9.0, 0.0, 9.0);
    let snapshot = Snapshot {
        players: vec![
            SnapshotPlayer {
                index: 0,
                position: first,
            },
            SnapshotPlayer {
                index: 2,
                position: third,
            },
        ],
        ..Default::default()
    };

    game.apply_snapshot(&snapshot).unwrap();

    assert_eq!(game.state.player_states[0].position, first);
    assert_eq!(game.state.player_states[2].position, third);
}

#[test]
fn applies_setup_restart() {
    let mut game = game();
    let restart_position = Point3D::from_meters(34.0, 0.0, 5.5);
    let snapshot = Snapshot {
        setup: Some(SnapshotSetup {
            restart_position: Some(restart_position),
            restart_team: Some(Team::A),
        }),
        ..Default::default()
    };

    game.apply_snapshot(&snapshot).unwrap();

    assert_eq!(game.state.restart_position, Some(restart_position));
    assert_eq!(game.state.restart_team, Some(Team::A));
}

#[test]
fn applies_score() {
    let mut game = game();
    let snapshot = Snapshot {
        score: Some(Score { a: 2, b: 1 }),
        ..Default::default()
    };

    game.apply_snapshot(&snapshot).unwrap();

    assert_eq!(game.state.team_stats[&Team::A].get("score"), 2.0);
    assert_eq!(game.state.team_stats[&Team::B].get("score"), 1.0);
}

#[test]
fn missing_fields_leave_state_untouched() {
    let mut game = game();
    let stage = game.state.stage.clone();
    let ball_position = game.state.ball_state.position;
    let player_position = game.state.player_states[0].position;

    game.apply_snapshot(&Snapshot::default()).unwrap();

    assert_eq!(game.state.stage, stage);
    assert_eq!(game.state.ball_state.position, ball_position);
    assert_eq!(game.state.player_states[0].position, player_position);
    assert_eq!(game.state.restart_position, None);
    assert_eq!(game.state.restart_team, None);
}

#[test]
fn rejects_out_of_bounds_player_index() {
    let mut game = game();
    let snapshot = Snapshot {
        players: vec![SnapshotPlayer {
            index: 3,
            position: Point3D::default(),
        }],
        ..Default::default()
    };

    let error = game.apply_snapshot(&snapshot).unwrap_err();

    assert!(error.contains("3"), "unexpected error: {error}");
}

#[test]
fn rejects_out_of_bounds_possession_index() {
    let mut game = game();
    let snapshot = Snapshot {
        ball: Some(SnapshotBall {
            possessed_by: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    };

    let error = game.apply_snapshot(&snapshot).unwrap_err();

    assert!(error.contains("5"), "unexpected error: {error}");
}

#[test]
fn deserializes_from_toml() {
    let source = r#"
stage = "Play"
score = { A = 1, B = 2 }

[ball]
position = { x = 34.0, y = 0.0, z = 10.0 }
possessed_by = 1

[[players]]
index = 2
position = { x = 1.0, y = 0.0, z = 3.0 }
"#;

    let snapshot: Snapshot = toml::from_str(source).unwrap();

    assert_eq!(snapshot.stage, Some(GameStage::Play));
    assert_eq!(snapshot.score, Some(Score { a: 1, b: 2 }));
    assert_eq!(snapshot.ball.unwrap().possessed_by, Some(1));
    assert_eq!(snapshot.players.len(), 1);
    assert_eq!(snapshot.players[0].index, 2);
}

#[test]
fn round_trips_through_json() {
    let snapshot = Snapshot {
        stage: Some(GameStage::Setup("timeout".to_string())),
        ball: Some(SnapshotBall {
            position: Some(Point3D::from_meters(1.0, 2.0, 3.0)),
            velocity: Some(Velocity3D::from_meters_per_second(4.0, 5.0, 6.0)),
            possessed_by: Some(1),
            last_possessing_team: Some(Team::A),
        }),
        players: vec![SnapshotPlayer {
            index: 1,
            position: Point3D::from_meters(7.0, 0.0, 8.0),
        }],
        setup: Some(SnapshotSetup {
            restart_position: Some(Point3D::from_meters(9.0, 0.0, 10.0)),
            restart_team: Some(Team::B),
        }),
        score: Some(Score { a: 3, b: 0 }),
    };

    let json = serde_json::to_string(&snapshot).unwrap();

    assert_eq!(serde_json::from_str::<Snapshot>(&json).unwrap(), snapshot);
}
