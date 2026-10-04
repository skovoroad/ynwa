use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

use super::*;
use crate::scenario::JournalMatch;
use ynwa_core::field::Field;
use ynwa_core::game::{BallDef, PlayerDef, RefereeDef, ScriptingConfig, REGION_START_POSITION};
use ynwa_core::region::GridCell;
use ynwa_core::{Point3D, Score, Velocity3D};

/// Minimal `Game` with the given `{team, number}` players, used to test resolution and snapshot
/// conversion without touching the filesystem.
fn test_game(players: &[(Team, u32)]) -> Game {
    let field = Field::from_meters(68.0, 104.615_38, 26, 40);
    let grid_dims = field.grid_dimensions();
    let defs = players
        .iter()
        .map(|&(team, number)| {
            let region = grid_dims
                .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
                .unwrap();
            PlayerDef::new(
                team,
                number,
                format!("Player {team:?}{number}"),
                String::new(),
                HashMap::from([(REGION_START_POSITION.to_string(), region)]),
            )
        })
        .collect();

    let config = GameConfig {
        field,
        players: defs,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    };
    Game::with_stage(
        config,
        GameStage::Play,
        Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(0)))),
    )
}

fn parse_initial_state(source: &str) -> InitialState {
    InitialState::parse(source).expect("initial state must parse")
}

#[test]
fn resolves_player_by_team_and_number() {
    let game = test_game(&[(Team::A, 3), (Team::B, 7)]);

    assert_eq!(resolve_player_index(game.config(), Team::A, 3).unwrap(), 0);
    assert_eq!(resolve_player_index(game.config(), Team::B, 7).unwrap(), 1);
}

#[test]
fn unknown_player_reference_is_rejected() {
    let game = test_game(&[(Team::A, 1)]);

    let error = resolve_player_index(game.config(), Team::A, 99).unwrap_err();
    assert!(error.message().contains("99"), "unexpected error: {error}");

    let error = resolve_player_index(game.config(), Team::B, 1).unwrap_err();
    assert!(error.message().contains("B"), "unexpected error: {error}");
}

#[test]
fn duplicate_player_reference_is_rejected() {
    let game = test_game(&[(Team::A, 1), (Team::A, 1)]);

    let error = resolve_player_index(game.config(), Team::A, 1).unwrap_err();
    assert!(
        error.message().contains("more than once"),
        "unexpected error: {error}"
    );
}

#[test]
fn builds_snapshot_with_resolved_ball_owner_and_players() {
    let game = test_game(&[(Team::A, 1), (Team::B, 2)]);
    let initial = parse_initial_state(
        r#"
stage = "Play"

[ball]
position = { x = 34.0, y = 0.0, z = 10.0 }
velocity = { x = 1.5, y = 0.0, z = -2.5 }
possessed_by = { team = "B", number = 2 }
last_possessing_team = "A"

[[players]]
team = "A"
number = 1
position = { x = 35.0, y = 0.0, z = 11.0 }
"#,
    );

    let snapshot = build_snapshot(&initial, &game).unwrap();

    assert_eq!(snapshot.stage, Some(GameStage::Play));
    let ball = snapshot.ball.expect("ball snapshot");
    assert_eq!(ball.position, Some(Point3D::from_meters(34.0, 0.0, 10.0)));
    assert_eq!(
        ball.velocity,
        Some(Velocity3D::from_meters_per_second(1.5, 0.0, -2.5))
    );
    assert_eq!(ball.possessed_by, Some(1));
    assert_eq!(ball.last_possessing_team, Some(Team::A));
    assert_eq!(
        snapshot.players,
        vec![SnapshotPlayer {
            index: 0,
            position: Point3D::from_meters(35.0, 0.0, 11.0),
        }]
    );
    assert!(snapshot.setup.is_none());
    assert!(snapshot.score.is_none());
}

#[test]
fn free_and_absent_ball_map_to_no_possession() {
    let game = test_game(&[(Team::A, 1), (Team::B, 2)]);

    let free = parse_initial_state(
        "stage = \"Play\"\n[ball]\npossessed_by = \"none\"\nlast_possessing_team = \"none\"\n",
    );
    let ball = build_snapshot(&free, &game).unwrap().ball.unwrap();
    assert_eq!(ball.possessed_by, None);
    assert_eq!(ball.last_possessing_team, None);

    let absent = parse_initial_state("stage = \"Play\"\n");
    assert!(build_snapshot(&absent, &game).unwrap().ball.is_none());
}

#[test]
fn setup_stage_maps_reason_and_restart() {
    let game = test_game(&[(Team::A, 1)]);
    let initial = parse_initial_state(
        r#"
stage = "Setup"
setup_reason = "throw in"

[setup]
restart_position = { x = 5.0, y = 0.0, z = 6.0 }
restart_team = "A"
"#,
    );

    let snapshot = build_snapshot(&initial, &game).unwrap();

    assert_eq!(
        snapshot.stage,
        Some(GameStage::Setup("throw in".to_string()))
    );
    assert_eq!(
        snapshot.setup,
        Some(SnapshotSetup {
            restart_position: Some(Point3D::from_meters(5.0, 0.0, 6.0)),
            restart_team: Some(Team::A),
        })
    );
}

#[test]
fn omitted_stage_defaults_to_kick_off() {
    let game = test_game(&[(Team::A, 1)]);

    let snapshot = build_snapshot(&parse_initial_state(""), &game).unwrap();

    assert_eq!(
        snapshot.stage,
        Some(GameStage::Setup("kick off".to_string()))
    );
}

#[test]
fn restart_team_none_maps_to_absent() {
    let game = test_game(&[(Team::A, 1)]);
    let initial = parse_initial_state("stage = \"Setup\"\n[setup]\nrestart_team = \"none\"\n");

    let setup = build_snapshot(&initial, &game).unwrap().setup.unwrap();

    assert_eq!(setup.restart_team, None);
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn loads_scenario_directory_into_world() {
    let loaded = load_scenario(&fixture("valid")).expect("valid scenario must load");

    assert_eq!(loaded.name, "valid");
    let state = loaded.world.game().state();
    assert_eq!(state.stage, GameStage::Play);
    assert_eq!(
        state.ball_state.position,
        Point3D::from_meters(34.0, 0.0, 10.0)
    );
    assert_eq!(
        state.ball_state.velocity,
        Velocity3D::from_meters_per_second(1.5, 0.0, -2.5)
    );
    assert_eq!(state.ball_state.possessed_by, Some(1));
    assert_eq!(state.ball_state.last_possessing_team, Some(Team::B));
    assert_eq!(
        state.player_states[0].position,
        Point3D::from_meters(35.0, 0.0, 11.0)
    );

    assert_eq!(loaded.run.dt, 0.1);
    assert_eq!(loaded.expect.journal_match, JournalMatch::Subsequence);
    assert_eq!(
        loaded.final_state.unwrap().score,
        Some(Score { a: 0, b: 0 })
    );
}

/// Temporary directory removed on drop, so error-path tests can build partial scenario trees.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "ynwa_integration_{label}_{}_{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&path);
        std::fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn copy_dir_all(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create destination dir");
    for entry in std::fs::read_dir(from).expect("read source dir") {
        let entry = entry.expect("dir entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("file type").is_dir() {
            copy_dir_all(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

fn load_error(dir: &Path) -> ScenarioError {
    load_scenario(dir).err().expect("load must fail")
}

#[test]
fn missing_initial_state_is_error() {
    let error = load_error(&fixture("does_not_exist"));

    assert!(
        error.message().contains("initial_state.toml"),
        "unexpected error: {error}"
    );
}

#[test]
fn missing_scenario_file_is_error() {
    let dir = TempDir::new("missing_scenario");
    std::fs::write(dir.path().join("initial_state.toml"), "stage = \"Play\"\n").unwrap();

    let error = load_error(dir.path());

    assert!(
        error.message().contains("scenario.toml"),
        "unexpected error: {error}"
    );
}

#[test]
fn invalid_initial_state_is_error() {
    let dir = TempDir::new("invalid_initial");
    std::fs::write(dir.path().join("initial_state.toml"), "stage = 12\n").unwrap();

    assert!(load_scenario(dir.path()).is_err());
}

#[test]
fn preamble_syntax_error_fails_the_load() {
    let dir = TempDir::new("broken_preamble");
    copy_dir_all(&fixture("valid"), dir.path());
    std::fs::write(
        dir.path().join("teams/team_a/preamble.lua"),
        "this is not valid lua !!!",
    )
    .unwrap();

    assert!(load_scenario(dir.path()).is_err());
}
