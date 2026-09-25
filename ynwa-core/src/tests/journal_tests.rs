//! Tests for the journal, the record model and the `Game` journal API.

use crate::field::zones::{Point3D, Velocity3D};
use crate::field::Field;
use crate::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, PlayerDef, RefereeDef,
    REGION_START_POSITION,
};
use crate::journal::{
    CollectJournalRecorder, EventsCollection, JournalEntry, JournalEvent, JournalSink,
    NullJournalSink,
};
use crate::record::RecordHeader;
use crate::region::GridCell;
use crate::team::Team;
use crate::test_utils::{attach_journal, deterministic_rng};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn test_config() -> GameConfig {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();

    GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            String::new(),
            HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: crate::game::ScriptingConfig::empty(),
    }
}

fn test_game() -> Game {
    Game::new(test_config(), deterministic_rng())
}

fn test_header() -> RecordHeader {
    RecordHeader {
        config: test_config(),
        initial_stage: GameStage::Setup("kick off".to_string()),
        fixed_dt: 0.016,
    }
}

fn shared_collection() -> (Rc<RefCell<EventsCollection>>, CollectJournalRecorder) {
    let collection = Rc::new(RefCell::new(EventsCollection::default()));
    let recorder = CollectJournalRecorder::new(Rc::clone(&collection));
    (collection, recorder)
}

fn decision_event(player_index: usize) -> JournalEvent {
    JournalEvent::DecisionAssigned {
        player_index,
        decision: Decision::Run(DecisionTarget::GridCell(GridCell::new(1, 1).unwrap())),
        reason: Some("test".to_string()),
    }
}

#[test]
fn null_sink_ignores_events_and_finishes_ok() {
    let mut sink: Box<dyn JournalSink> = Box::new(NullJournalSink);

    sink.push(0.0, decision_event(0));
    sink.finish_step(0.0);

    assert!(sink.finish().is_ok());
}

#[test]
fn collect_recorder_forwards_events_to_collection() {
    let (collection, mut recorder) = shared_collection();

    recorder.push(0.016, decision_event(0));
    recorder.push(0.032, JournalEvent::DecisionsReset);

    let entries = collection.borrow().entries().to_vec();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].timestamp, 0.016);
    assert_eq!(entries[0].event, decision_event(0));
    assert_eq!(entries[1].event, JournalEvent::DecisionsReset);
}

#[test]
fn collect_recorder_publishes_step_count_only_on_finish() {
    let (collection, mut recorder) = shared_collection();

    recorder.finish_step(0.016);
    recorder.finish_step(0.032);
    recorder.finish_step(0.048);
    assert_eq!(collection.borrow().total_steps(), 0);

    Box::new(recorder).finish().unwrap();
    assert_eq!(collection.borrow().total_steps(), 3);
}

#[test]
fn take_record_moves_events_and_step_count() {
    let mut collection = EventsCollection::default();
    collection.push(JournalEntry {
        timestamp: 0.1,
        event: decision_event(1),
    });
    collection.push(JournalEntry {
        timestamp: 0.2,
        event: JournalEvent::DecisionsReset,
    });
    collection.set_total_steps(42);

    let record = collection.take_record(test_header());

    assert_eq!(record.total_steps, 42);
    assert_eq!(record.journal.len(), 2);
    assert_eq!(record.journal[0].timestamp, 0.1);
    assert_eq!(record.header.fixed_dt, 0.016);
    assert_eq!(
        record.header.initial_stage,
        GameStage::Setup("kick off".to_string())
    );
    assert!(collection.entries().is_empty());
}

#[test]
fn game_records_into_attached_sink() {
    let mut game = test_game();
    let collection = attach_journal(&mut game);

    game.record(
        0.5,
        JournalEvent::StageChange {
            stage: GameStage::Play,
        },
    );
    game.finish_step(0.5);
    game.finish_journal().unwrap();

    let borrowed = collection.borrow();
    assert_eq!(borrowed.entries().len(), 1);
    assert_eq!(borrowed.entries()[0].timestamp, 0.5);
    assert_eq!(borrowed.total_steps(), 1);
}

#[test]
fn finish_journal_detaches_sink() {
    let mut game = test_game();
    let collection = attach_journal(&mut game);

    game.finish_journal().unwrap();
    game.record(0.9, JournalEvent::DecisionsReset);
    game.finish_step(0.9);

    assert!(collection.borrow().entries().is_empty());
    assert_eq!(collection.borrow().total_steps(), 0);
}

#[test]
fn default_game_recording_is_a_no_op() {
    let mut game = test_game();

    game.record(0.0, JournalEvent::DecisionsReset);
    game.finish_step(0.0);

    assert!(game.finish_journal().is_ok());
}

#[test]
fn journal_entries_round_trip_through_json() {
    let events = [
        decision_event(2),
        JournalEvent::PossessionChange {
            possessed_by: Some(4),
            last_possessing_team: Some(Team::B),
        },
        JournalEvent::KickOutcome {
            player_index: 7,
            ball_velocity: Velocity3D::from_meters_per_second(1.5, 0.0, -2.5),
        },
        JournalEvent::StageChange {
            stage: GameStage::Setup("throw in".to_string()),
        },
        JournalEvent::RestartSet {
            restart_position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
            restart_team: Some(Team::A),
        },
        JournalEvent::DecisionsReset,
        JournalEvent::StatUpdate {
            team: Team::A,
            key: "score".to_string(),
            delta: 1.0,
        },
        JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::json!({"Goal": "A"}),
        },
    ];

    for event in events {
        let entry = JournalEntry {
            timestamp: 1.25,
            event,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let restored: JournalEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, entry);
    }
}

struct FailingSink;

impl JournalSink for FailingSink {
    fn push(&mut self, _timestamp: f32, _event: JournalEvent) {}

    fn finish(self: Box<Self>) -> Result<(), String> {
        Err("write failed".to_string())
    }
}

#[test]
fn finish_journal_propagates_sink_error() {
    let mut game = test_game();
    game.set_journal_sink(Box::new(FailingSink));

    assert_eq!(game.finish_journal().unwrap_err(), "write failed");
}
