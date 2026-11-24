//! Journal of the essential events of a playthrough.
//!
//! The journal is the canonical in-memory source for replay: it stores only events that
//! cannot be recomputed cheaply (decisions, random-branch outcomes, stage changes), not
//! frame-by-frame state. Determinism of a recording is guaranteed within a single platform
//! only; cross-platform determinism is a separate task.

use crate::field::zones::{Point3D, Velocity3D};
use crate::game::{Decision, GameStage, GameState};
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
        team: Team,
        number: u32,
        decision: Decision,
        reason: Option<String>,
    },
    PossessionChange {
        possessed_by: Option<usize>,
        possessed_by_team: Option<Team>,
        possessed_by_number: Option<u32>,
        last_possessing_team: Option<Team>,
    },
    KickOutcome {
        player_index: usize,
        team: Team,
        number: u32,
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

    /// Finalizes the recording (trailer, flush). `final_state` is the state reached after the
    /// last recorded step. Deliberately returns no `Record`.
    fn finish(self: Box<Self>, final_state: &GameState) -> Result<(), String> {
        let _ = final_state;
        Ok(())
    }
}

/// Sink for an ordinary game: recording is off.
pub struct NullJournalSink;

impl JournalSink for NullJournalSink {
    fn push(&mut self, _timestamp: f32, _event: JournalEvent) {}
}

/// Sink that forwards events into a shared buffer and keeps no state of its own.
///
/// The buffer is shared between the sink (inside `Game`) and the caller, so `Game` can own
/// `Box<dyn JournalSink>` without a lifetime parameter while the caller reads events as they are
/// recorded. Assembling the final `Record` is the caller's responsibility.
pub struct CollectJournalRecorder {
    entries: Rc<RefCell<Vec<JournalEntry>>>,
}

impl CollectJournalRecorder {
    pub fn new(entries: Rc<RefCell<Vec<JournalEntry>>>) -> Self {
        Self { entries }
    }
}

impl JournalSink for CollectJournalRecorder {
    fn push(&mut self, timestamp: f32, event: JournalEvent) {
        self.entries
            .borrow_mut()
            .push(JournalEntry { timestamp, event });
    }
}

/// Sink that forwards every event to two sinks, for example a file and an in-memory buffer.
pub struct TeeJournalSink {
    first: Box<dyn JournalSink>,
    second: Box<dyn JournalSink>,
}

impl TeeJournalSink {
    pub fn new(first: Box<dyn JournalSink>, second: Box<dyn JournalSink>) -> Self {
        Self { first, second }
    }
}

impl JournalSink for TeeJournalSink {
    fn push(&mut self, timestamp: f32, event: JournalEvent) {
        self.first.push(timestamp, event.clone());
        self.second.push(timestamp, event);
    }

    /// Finalizes both sinks, reporting the first sink's error when both fail.
    fn finish(self: Box<Self>, final_state: &GameState) -> Result<(), String> {
        let Self { first, second } = *self;
        let first_result = first.finish(final_state);
        let second_result = second.finish(final_state);
        first_result.and(second_result)
    }
}
