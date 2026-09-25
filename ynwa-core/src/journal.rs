//! Journal of the essential events of a playthrough.
//!
//! The journal is the canonical in-memory source for replay: it stores only events that
//! cannot be recomputed cheaply (decisions, random-branch outcomes, stage changes), not
//! frame-by-frame state. Determinism of a recording is guaranteed within a single platform
//! only; cross-platform determinism is a separate task.

use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, GameStage};
use crate::record::{Record, RecordHeader};
use crate::team::Team;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::rc::Rc;

/// A journaled event together with the absolute step time it happened at.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub timestamp: f32,
    pub event: JournalEvent,
}

/// Events sufficient to reproduce a playthrough by re-simulating physics only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum JournalEvent {
    DecisionAssigned {
        player_index: usize,
        decision: Decision,
        reason: Option<String>,
    },
    PossessionChange {
        possessed_by: Option<usize>,
        last_possessing_team: Option<Team>,
    },
    KickOutcome {
        player_index: usize,
        ball_velocity: Velocity3D,
    },
    StageChange {
        stage: GameStage,
    },
    RestartSet {
        restart_position: Option<Point3D>,
        restart_team: Option<Team>,
    },
    DecisionsReset,
    StatUpdate {
        team: Team,
        key: String,
        delta: f64,
    },
    /// Escape hatch for sport-specific semantic events (football writes its events here).
    /// Core never interprets `data`.
    External {
        kind: String,
        data: serde_json::Value,
    },
}

/// Receives journaled events. Where the events are stored is up to the implementation.
pub trait JournalSink {
    fn push(&mut self, timestamp: f32, event: JournalEvent);

    /// Called exactly once per simulation step, so a sink counts steps on its own.
    fn finish_step(&mut self, timestamp: f32) {
        let _ = timestamp;
    }

    /// Finalizes the recording (trailer, flush). Deliberately returns no `Record`.
    fn finish(self: Box<Self>) -> Result<(), String> {
        Ok(())
    }
}

/// Sink for an ordinary game: recording is off.
pub struct NullJournalSink;

impl JournalSink for NullJournalSink {
    fn push(&mut self, _timestamp: f32, _event: JournalEvent) {}
}

/// Storage for journaled events and the step count of a playthrough.
///
/// Shared between the sink (inside `Game`) and the caller, so `Game` can own
/// `Box<dyn JournalSink>` without a lifetime parameter.
#[derive(Default)]
pub struct EventsCollection {
    entries: Vec<JournalEntry>,
    total_steps: u64,
}

impl EventsCollection {
    pub fn push(&mut self, entry: JournalEntry) {
        self.entries.push(entry);
    }

    pub fn entries(&self) -> &[JournalEntry] {
        &self.entries
    }

    pub fn total_steps(&self) -> u64 {
        self.total_steps
    }

    pub fn set_total_steps(&mut self, steps: u64) {
        self.total_steps = steps;
    }

    /// Takes the collected events and step count, completing them with a header.
    pub fn take_record(&mut self, header: RecordHeader) -> Record {
        Record {
            header,
            total_steps: self.total_steps,
            journal: std::mem::take(&mut self.entries),
        }
    }
}

/// Sink that forwards events into a shared collection and keeps no state of its own.
pub struct CollectJournalRecorder {
    collection: Rc<RefCell<EventsCollection>>,
    steps: u64,
}

impl CollectJournalRecorder {
    pub fn new(collection: Rc<RefCell<EventsCollection>>) -> Self {
        Self {
            collection,
            steps: 0,
        }
    }
}

impl JournalSink for CollectJournalRecorder {
    fn push(&mut self, timestamp: f32, event: JournalEvent) {
        self.collection
            .borrow_mut()
            .push(JournalEntry { timestamp, event });
    }

    fn finish_step(&mut self, _timestamp: f32) {
        self.steps += 1;
    }

    fn finish(self: Box<Self>) -> Result<(), String> {
        self.collection.borrow_mut().set_total_steps(self.steps);
        Ok(())
    }
}
