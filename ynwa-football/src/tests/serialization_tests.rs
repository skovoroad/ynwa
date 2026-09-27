//! Stage-6 serialization tests: the end-to-end file round trip, replay independence from RNG and
//! a multi-player ball contest. All runs are compared step by step through the shared
//! [`test_utils`](crate::test_utils) infrastructure.

use crate::replay::create_football_replay_world;
use crate::test_utils::{
    assert_equivalent, build_recording_world_with, build_recording_world_with_rng, rng_with,
    run_lockstep, single_player_config, ChaseAndKick, FIXED_DT, STEPS,
};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use uom::si::velocity::meter_per_second;
use ynwa_core::codec::{FileJournalRecorder, RecordReader};
use ynwa_core::field::zones::{Point3D, Velocity3D};
use ynwa_core::field::Field;
use ynwa_core::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, PlayerDef, RefereeDef,
    ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::journal::JournalEvent;
use ynwa_core::physics_util::kick_speed;
use ynwa_core::record::RecordHeader;
use ynwa_core::record_io::{json_journal_file_reader, json_journal_file_writer};
use ynwa_core::region::GridCell;
use ynwa_core::systems::{DecisionError, DecisionMaker};
use ynwa_core::team::Team;

fn temp_path(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("ynwa-football-serialization-{name}-{nanos}.jsonl"))
}

/// Writes a short recorded run straight to a file, then reads it back and replays it.
///
/// The record read from the file must equal one assembled through `CollectJournalRecorder` in a
/// parallel run of the same scenario (JSON round-trips `f32` without loss), and the replay must
/// reproduce the original physical state.
#[test]
fn file_round_trip_reproduces_the_recorded_run() {
    let config = single_player_config();
    let stage = GameStage::Setup("kick off".to_string());
    let path = temp_path("round-trip");

    let (mut file_world, _) =
        build_recording_world_with(config.clone(), stage.clone(), Box::new(ChaseAndKick));
    let header = RecordHeader::from_game(file_world.game(), FIXED_DT);
    let writer = json_journal_file_writer(&path).unwrap();
    file_world
        .game_mut()
        .set_journal_sink(Box::new(FileJournalRecorder::new(
            Box::new(writer),
            &header,
        )));
    for _ in 0..STEPS {
        file_world.step(FIXED_DT);
    }
    file_world.game_mut().finish_journal().unwrap();

    let (mut collected_world, collection) =
        build_recording_world_with(config, stage, Box::new(ChaseAndKick));
    let collected_header = RecordHeader::from_game(collected_world.game(), FIXED_DT);
    for _ in 0..STEPS {
        collected_world.step(FIXED_DT);
    }
    collected_world.game_mut().finish_journal().unwrap();
    let collected = collection.borrow_mut().take_record(collected_header);

    let mut reader = json_journal_file_reader(&path).unwrap();
    let from_file = reader.read().unwrap();
    std::fs::remove_file(&path).unwrap();

    assert_eq!(
        from_file, collected,
        "the file round trip must preserve the record exactly"
    );

    let total_steps = from_file.total_steps;
    let mut replay = create_football_replay_world(from_file).unwrap();
    for _ in 0..total_steps {
        replay.step(FIXED_DT);
    }

    assert_equivalent(collected_world.game().state(), replay.game().state());
}

fn velocity_magnitude(velocity: &Velocity3D) -> f32 {
    let x = velocity.x.get::<meter_per_second>();
    let y = velocity.y.get::<meter_per_second>();
    let z = velocity.z.get::<meter_per_second>();
    (x * x + y * y + z * z).sqrt()
}

/// The original run randomizes the kick, while the replay world is built with a different seed and
/// never consults RNG; the contract state must still match on every step.
#[test]
fn replay_is_independent_of_rng() {
    let (mut original, collection) = build_recording_world_with_rng(
        single_player_config(),
        GameStage::Setup("kick off".to_string()),
        Box::new(ChaseAndKick),
        rng_with(1.0, 7),
    );
    let header = RecordHeader::from_game(original.game(), FIXED_DT);

    let (original_before, _) = run_lockstep(&mut original, STEPS);
    original.game_mut().finish_journal().unwrap();
    let original_final = original.game().state().clone();

    let record = collection.borrow_mut().take_record(header);
    let total_steps = record.total_steps;

    let kicked = record
        .journal
        .iter()
        .find_map(|entry| match &entry.event {
            JournalEvent::KickOutcome { ball_velocity, .. } => Some(*ball_velocity),
            _ => None,
        })
        .expect("scenario must produce a kick");
    let base_speed = kick_speed(100);
    assert!(
        (velocity_magnitude(&kicked) - base_speed).abs() > 1e-3,
        "kick spread must actually fire: {} vs base {}",
        velocity_magnitude(&kicked),
        base_speed
    );

    let mut replay = create_football_replay_world(record).unwrap();
    let (replay_before, _) = run_lockstep(&mut replay, total_steps);
    let replay_final = replay.game().state().clone();

    for (original, replayed) in original_before.iter().zip(&replay_before) {
        assert_equivalent(original, replayed);
    }
    assert_equivalent(&original_final, &replay_final);
}

const CONTEST_STEPS: u64 = 150;

/// Two players per team, all starting on the centre spot of a tiny field, so both teams stay within
/// the possession radius and contest the ball repeatedly.
fn contest_config() -> GameConfig {
    let field = Field::from_meters(10.0, 10.0, 10, 10);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(5, 5).unwrap(), GridCell::new(5, 5).unwrap())
        .unwrap();

    let players = [(Team::A, 1), (Team::A, 2), (Team::B, 3), (Team::B, 4)]
        .into_iter()
        .map(|(team, number)| {
            PlayerDef::new(
                team,
                number,
                format!("Contest {number}"),
                String::new(),
                HashMap::from([(REGION_START_POSITION.to_string(), start_region.clone())]),
            )
            .with_tackle_rate(100)
            .with_reaction_rate(100)
            .with_speed_rate(100)
        })
        .collect();

    GameConfig {
        field,
        players,
        ball: BallDef {
            initial_position: Point3D::from_meters(5.0, 0.0, 5.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

/// Every player chases the ball, so possession keeps changing hands between the two teams.
struct ChaseBall;

impl DecisionMaker for ChaseBall {
    fn make_decision(
        &mut self,
        _game: &Game,
        _player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        Ok((
            Decision::Run(DecisionTarget::Ball),
            Some("chase".to_string()),
        ))
    }
}

/// Several players of both teams contest the ball; the replay must reproduce the original on every
/// step, and the recording must show repeated possession changes across both teams.
#[test]
fn replay_reproduces_a_multi_player_ball_contest() {
    let (mut original, collection) =
        build_recording_world_with(contest_config(), GameStage::Play, Box::new(ChaseBall));
    let header = RecordHeader::from_game(original.game(), FIXED_DT);

    let (original_before, _) = run_lockstep(&mut original, CONTEST_STEPS);
    original.game_mut().finish_journal().unwrap();
    let original_final = original.game().state().clone();

    let record = collection.borrow_mut().take_record(header);
    let total_steps = record.total_steps;

    let changes: Vec<&JournalEvent> = record
        .journal
        .iter()
        .map(|entry| &entry.event)
        .filter(|event| matches!(event, JournalEvent::PossessionChange { .. }))
        .collect();
    assert_eq!(
        changes.len(),
        15,
        "one possession change per second of play is expected: {changes:?}"
    );

    let mut owners = HashSet::new();
    let mut teams = HashSet::new();
    for change in &changes {
        if let JournalEvent::PossessionChange {
            possessed_by,
            last_possessing_team,
        } = change
        {
            owners.insert(*possessed_by);
            teams.insert(*last_possessing_team);
        }
    }
    assert!(
        owners.len() >= 2,
        "the ball must change hands between players: {owners:?}"
    );
    assert!(
        teams.contains(&Some(Team::A)) && teams.contains(&Some(Team::B)),
        "both teams must win the ball: {teams:?}"
    );

    let mut replay = create_football_replay_world(record).unwrap();
    let (replay_before, _) = run_lockstep(&mut replay, total_steps);
    let replay_final = replay.game().state().clone();

    for (original, replayed) in original_before.iter().zip(&replay_before) {
        assert_equivalent(original, replayed);
    }
    assert_equivalent(&original_final, &replay_final);
}
