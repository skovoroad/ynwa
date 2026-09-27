//! Replay equivalence and football-event decoding: a recorded run and its replay must end in the
//! same physically significant state, the recording must decode to the expected football events,
//! and the replayed ball must satisfy the conditions those events were detected under.

use crate::events::{FootballEvent, BALL_RADIUS, GAME_DURATION};
use crate::field_builder::create_football_field;
use crate::replay::{create_football_replay_world, decode_football_events};
use crate::test_utils::{
    assert_equivalent, build_recording_world, build_recording_world_with, single_player_config,
    FIXED_DT, STEPS,
};
use std::collections::HashMap;
use uom::si::length::meter;
use ynwa_core::field::zones::Point3D;
use ynwa_core::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, GameState, PlayerDef,
    RefereeDef, ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::record::{Record, RecordHeader};
use ynwa_core::region::GridCell;
use ynwa_core::systems::{DecisionError, DecisionMaker};
use ynwa_core::team::Team;
use ynwa_core::world::World;

/// Runs the short recordable game to completion; returns the finished world and its record.
fn record_short_game() -> (World, Record) {
    let (mut world, collection) = build_recording_world();
    for _ in 0..STEPS {
        world.step(FIXED_DT);
    }
    world.game_mut().finish_journal().unwrap();

    let header = RecordHeader {
        config: single_player_config(),
        initial_stage: GameStage::Setup("kick off".to_string()),
        fixed_dt: FIXED_DT,
    };
    let record = collection.borrow_mut().take_record(header);
    (world, record)
}

#[test]
fn replay_reproduces_physical_state() {
    let (original, record) = record_short_game();
    let total_steps = record.total_steps;

    let mut replay = create_football_replay_world(record).unwrap();
    for _ in 0..total_steps {
        replay.step(FIXED_DT);
    }

    assert_equivalent(original.game().state(), replay.game().state());
}

#[test]
fn recording_decodes_to_expected_football_events() {
    let (_original, record) = record_short_game();

    let events = decode_football_events(&record.journal);

    assert_eq!(
        events.len(),
        1,
        "expected a single touchline event: {events:?}"
    );
    assert!(matches!(&events[0].1, FootballEvent::Touchline(_, Team::A)));
    assert!(events[0].0 > 0.0);
}

/// For every recorded event, the replayed ball must satisfy the condition it was detected under.
#[test]
fn replay_ball_matches_recorded_event_conditions() {
    let (original, record) = record_short_game();
    let width = original.game().config().field.width().get::<meter>();
    let length = original.game().config().field.length().get::<meter>();
    let events = decode_football_events(&record.journal);
    assert!(
        !events.is_empty(),
        "scenario must record at least one event"
    );
    let total_steps = record.total_steps;

    let mut replay = create_football_replay_world(record).unwrap();
    let mut steps = 0;
    for (timestamp, event) in &events {
        while replay.game().state.elapsed_time < *timestamp && steps < total_steps {
            replay.step(FIXED_DT);
            steps += 1;
        }
        let ball = replay.game().state.ball_state.position;
        let x = ball.x.get::<meter>();
        let z = ball.z.get::<meter>();
        match event {
            FootballEvent::Touchline(_, _) => assert!(
                x + BALL_RADIUS < 0.0 || x - BALL_RADIUS > width,
                "ball not over the touchline at {timestamp}: x = {x}"
            ),
            FootballEvent::GoalLine(_, _) => assert!(
                z + BALL_RADIUS < 0.0 || z - BALL_RADIUS > length,
                "ball not over the goal line at {timestamp}: z = {z}"
            ),
            FootballEvent::Goal(_) => assert!(
                z - BALL_RADIUS >= length || z + BALL_RADIUS <= 0.0,
                "ball not past the goal line at {timestamp}: z = {z}"
            ),
            FootballEvent::GameEnd => {
                assert!(replay.game().state.elapsed_time >= GAME_DURATION);
            }
        }
    }
}

#[test]
fn decode_reads_every_event_type_in_order() {
    let events = [
        FootballEvent::Goal(Team::B),
        FootballEvent::Touchline(Point3D::from_meters(-1.0, 0.0, 30.0), Team::A),
        FootballEvent::GoalLine(Point3D::from_meters(2.0, 0.0, -0.5), Team::B),
        FootballEvent::GameEnd,
    ];
    let journal: Vec<JournalEntry> = events
        .iter()
        .enumerate()
        .map(|(index, event)| football_external(0.1 * (index as f32 + 1.0), event.clone()))
        .collect();

    let decoded = decode_football_events(&journal);

    assert_eq!(decoded.len(), events.len());
    for (index, (timestamp, event)) in decoded.iter().enumerate() {
        assert_eq!(*event, events[index]);
        assert_eq!(*timestamp, 0.1 * (index as f32 + 1.0));
    }
}

#[test]
fn decode_skips_foreign_kinds_and_corrupt_payloads() {
    let valid = FootballEvent::GameEnd;
    let journal = vec![
        JournalEntry {
            timestamp: 0.1,
            event: JournalEvent::External {
                kind: "other".to_string(),
                data: serde_json::json!({ "x": 1 }),
            },
        },
        JournalEntry {
            timestamp: 0.2,
            event: JournalEvent::External {
                kind: "football_event".to_string(),
                data: serde_json::json!("not an event"),
            },
        },
        football_external(0.3, valid.clone()),
        JournalEntry {
            timestamp: 0.4,
            event: JournalEvent::DecisionsReset,
        },
    ];

    assert_eq!(decode_football_events(&journal), vec![(0.3, valid)]);
}

#[test]
fn decode_of_empty_journal_is_empty() {
    assert!(decode_football_events(&[]).is_empty());
}

#[test]
fn decode_of_journal_without_football_events_is_empty() {
    let journal = vec![
        JournalEntry {
            timestamp: 0.1,
            event: JournalEvent::DecisionsReset,
        },
        JournalEntry {
            timestamp: 0.2,
            event: JournalEvent::StageChange {
                stage: GameStage::Play,
            },
        },
    ];

    assert!(decode_football_events(&journal).is_empty());
}

const GOAL_SCENARIO_STEPS: u64 = 150;

/// One striker standing on the ball, told to shoot at the opponent goal.
fn goal_scenario_config() -> GameConfig {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();
    let width = field.width().get::<meter>();
    let start_region = grid_dims
        .create_region(
            GridCell::new(13, 20).unwrap(),
            GridCell::new(13, 20).unwrap(),
        )
        .unwrap();
    let ball_position = start_region.center(grid_dims, width);

    let player = PlayerDef::new(
        Team::A,
        1,
        "Striker".to_string(),
        String::new(),
        HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
    .with_reaction_rate(100)
    .with_speed_rate(100)
    .with_shot_power(100)
    .with_shot_accuracy(100);

    GameConfig {
        field,
        players: vec![player],
        ball: BallDef {
            initial_position: ball_position,
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

/// Chases the ball and, in possession, kicks it at the opponent goal (Team A attacks +Z).
struct ShootAtOpponentGoal;

impl DecisionMaker for ShootAtOpponentGoal {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        let decision = if game.state.ball_state.possessed_by == Some(player_index) {
            let field = &game.config().field;
            Decision::Kick(Point3D::from_meters(
                field.width().get::<meter>() / 2.0,
                0.0,
                field.length().get::<meter>(),
            ))
        } else {
            Decision::Run(DecisionTarget::Ball)
        };
        Ok((decision, Some("shoot".to_string())))
    }
}

#[test]
fn replay_reproduces_state_after_a_goal() {
    let config = goal_scenario_config();
    let (mut world, collection) = build_recording_world_with(
        config.clone(),
        GameStage::Play,
        Box::new(ShootAtOpponentGoal),
    );
    for _ in 0..GOAL_SCENARIO_STEPS {
        world.step(FIXED_DT);
    }
    world.game_mut().finish_journal().unwrap();

    let header = RecordHeader {
        config,
        initial_stage: GameStage::Play,
        fixed_dt: FIXED_DT,
    };
    let record = collection.borrow_mut().take_record(header);

    let events = decode_football_events(&record.journal);
    assert!(
        events
            .iter()
            .any(|(_, event)| matches!(event, FootballEvent::Goal(_))),
        "scenario must score: {events:?}"
    );
    assert_eq!(score(world.game().state(), Team::A), 1.0);

    let total_steps = record.total_steps;
    let mut replay = create_football_replay_world(record).unwrap();
    for _ in 0..total_steps {
        replay.step(FIXED_DT);
    }

    assert_equivalent(world.game().state(), replay.game().state());
    assert_eq!(score(replay.game().state(), Team::A), 1.0);
}

fn football_external(timestamp: f32, event: FootballEvent) -> JournalEntry {
    JournalEntry {
        timestamp,
        event: JournalEvent::External {
            kind: crate::game_manager::FOOTBALL_EVENT_KIND.to_string(),
            data: serde_json::to_value(event).unwrap(),
        },
    }
}

fn score(state: &GameState, team: Team) -> f64 {
    state
        .team_stats
        .get(&team)
        .map(|stats| stats.get("score"))
        .unwrap_or(0.0)
}
