//! Stop-criterion domain shared by the scenario model and the runner.
//!
//! Scenario files declare stop criteria as data ([`StopCriterionDto`](crate::dto::StopCriterionDto));
//! [`StopCriterion`] is the compiled form the runner checks after every step, and [`StopReason`]
//! records which criterion ended the run. The matchers narrow the stage, football event or journal
//! event a criterion reacts to.

use ynwa_core::game::{GameConfig, GameStage};
use ynwa_core::journal::JournalEntry;
use ynwa_core::team::Team;
use ynwa_football::events::FootballEvent;

use crate::dto::{EventKindDto, JournalStopPatternDto, StageNameDto};
use crate::loader::ScenarioError;

/// Target stage of an `OnStage` criterion. `Setup` with no reason matches any Setup reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StageMatcher {
    Play,
    GameOver,
    Setup { reason: Option<String> },
}

impl StageMatcher {
    fn matches(&self, stage: &GameStage) -> bool {
        match (self, stage) {
            (StageMatcher::Play, GameStage::Play) => true,
            (StageMatcher::GameOver, GameStage::GameOver) => true,
            (StageMatcher::Setup { reason }, GameStage::Setup(actual)) => {
                reason.as_ref().is_none_or(|expected| expected == actual)
            }
            _ => false,
        }
    }
}

/// Football event matcher. A team is carried only by the events that have one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventMatcher {
    Goal { team: Option<Team> },
    Touchline { team: Option<Team> },
    GoalLine { team: Option<Team> },
    GameEnd,
}

impl EventMatcher {
    /// Builds a matcher from a scenario file's event kind and optional team. `GameEnd` has no
    /// team, so a team together with `GameEnd` is rejected.
    pub fn from_def(event: EventKindDto, team: Option<Team>) -> Result<Self, String> {
        match event {
            EventKindDto::Goal => Ok(EventMatcher::Goal { team }),
            EventKindDto::Touchline => Ok(EventMatcher::Touchline { team }),
            EventKindDto::GoalLine => Ok(EventMatcher::GoalLine { team }),
            EventKindDto::GameEnd if team.is_none() => Ok(EventMatcher::GameEnd),
            EventKindDto::GameEnd => Err("event 'GameEnd' does not carry a team".to_string()),
        }
    }

    pub(crate) fn matches_football(&self, event: &FootballEvent) -> bool {
        match (self, event) {
            (EventMatcher::Goal { team }, FootballEvent::Goal(actual)) => accepts(*team, *actual),
            (EventMatcher::Touchline { team }, FootballEvent::Touchline(_, actual)) => {
                accepts(*team, *actual)
            }
            (EventMatcher::GoalLine { team }, FootballEvent::GoalLine(_, actual)) => {
                accepts(*team, *actual)
            }
            (EventMatcher::GameEnd, FootballEvent::GameEnd) => true,
            _ => false,
        }
    }
}

fn accepts(expected: Option<Team>, actual: Team) -> bool {
    expected.is_none_or(|team| team == actual)
}

/// Journal-event matcher for an `OnJournal` criterion. It carries the same per-type pattern as an
/// `[[expect.journal]]` entry, without a timestamp.
#[derive(Debug, Clone, PartialEq)]
pub struct JournalMatcher {
    pattern: JournalStopPatternDto,
}

impl JournalMatcher {
    pub fn new(pattern: JournalStopPatternDto) -> Self {
        Self { pattern }
    }

    /// First entry matching the pattern.
    fn find<'a>(
        &self,
        entries: &'a [JournalEntry],
        config: &GameConfig,
    ) -> Result<Option<&'a JournalEntry>, ScenarioError> {
        for entry in entries {
            if crate::compare::journal_pattern_matches(self.pattern.entry_pattern(), entry, config)?
            {
                return Ok(Some(entry));
            }
        }
        Ok(None)
    }
}

/// A compiled stop criterion checked after every step.
#[derive(Debug, Clone, PartialEq)]
pub enum StopCriterion {
    OnStage(StageMatcher),
    OnEvent(EventMatcher),
    OnJournal(JournalMatcher),
    OnSteps(u64),
    OnTime(f32),
}

impl StopCriterion {
    /// Why the criterion fired after a step, or `None` when it did not. `new_events` holds the
    /// football events decoded from the journal entries appended during that step; `new_entries`
    /// holds those same entries for journal matchers.
    pub(crate) fn evaluate(
        &self,
        stage: &GameStage,
        elapsed: f32,
        steps: u64,
        new_events: &[(f32, FootballEvent)],
        new_entries: &[JournalEntry],
        config: &GameConfig,
    ) -> Result<Option<StopReason>, ScenarioError> {
        let reason = match self {
            StopCriterion::OnStage(matcher) => matcher
                .matches(stage)
                .then(|| StopReason::Stage(matcher.clone())),
            StopCriterion::OnEvent(matcher) => new_events
                .iter()
                .any(|(_, event)| matcher.matches_football(event))
                .then_some(StopReason::Event(*matcher)),
            StopCriterion::OnJournal(matcher) => matcher
                .find(new_entries, config)?
                .map(|entry| StopReason::Journal(entry.clone())),
            StopCriterion::OnSteps(limit) => (steps >= *limit).then_some(StopReason::Steps(*limit)),
            StopCriterion::OnTime(limit) => (elapsed >= *limit).then_some(StopReason::Time(*limit)),
        };
        Ok(reason)
    }
}

/// The criterion that ended a run, preserving the parameters it was declared with. A journal stop
/// keeps the event that triggered it.
#[derive(Debug, Clone, PartialEq)]
pub enum StopReason {
    Stage(StageMatcher),
    Event(EventMatcher),
    Journal(JournalEntry),
    Steps(u64),
    Time(f32),
}

/// Converts a scenario stage name and optional Setup reason into a [`StageMatcher`].
pub fn stage_matcher(stage: StageNameDto, setup_reason: Option<String>) -> StageMatcher {
    match stage {
        StageNameDto::Play => StageMatcher::Play,
        StageNameDto::GameOver => StageMatcher::GameOver,
        StageNameDto::Setup => StageMatcher::Setup {
            reason: setup_reason,
        },
    }
}

#[cfg(test)]
#[path = "tests/criterion_tests.rs"]
mod tests;
