//! Deterministic scenario runner: step a loaded world until the first stop criterion fires.
//!
//! The world is stepped at the scenario's `dt`. After every step the criteria are evaluated in
//! declaration order and the first satisfied one becomes the [`StopReason`]. Football events are
//! decoded incrementally from the journal, so an `OnEvent` criterion sees each event exactly once.

use std::cell::RefCell;
use std::rc::Rc;

use ynwa_core::game::GameState;
use ynwa_core::journal::{CollectJournalRecorder, EventsCollection, JournalEntry};
use ynwa_core::world::World;
use ynwa_football::decode_football_events;
use ynwa_football::events::FootballEvent;

use crate::criterion::{stage_matcher, EventMatcher, StopCriterion, StopReason};
use crate::dto::{RunPlanDto, StopCriterionDto};
use crate::loader::ScenarioError;

/// Everything a completed run yields: the journal, the decoded football events, why the run
/// stopped, how many steps it took and the final game state.
pub struct RunOutcome {
    pub journal: Vec<JournalEntry>,
    pub football_events: Vec<(f32, FootballEvent)>,
    pub stop_reason: StopReason,
    pub steps: u64,
    pub final_state: GameState,
}

/// Runs `world` at `run.dt` until the first stop criterion is satisfied.
///
/// Any player left with a script error (`last_error`) fails the run.
pub fn run(world: World, run: &RunPlanDto) -> Result<RunOutcome, ScenarioError> {
    let criteria = compile_criteria(run)?;
    let mut world = world;

    let collection = Rc::new(RefCell::new(EventsCollection::default()));
    world
        .game_mut()
        .set_journal_sink(Box::new(CollectJournalRecorder::new(collection.clone())));

    let mut steps: u64 = 0;
    let mut decoded_upto: usize = 0;
    let mut football_events: Vec<(f32, FootballEvent)> = Vec::new();

    let stop_reason = loop {
        world.step(run.dt);
        steps += 1;

        let new_events = {
            let collection = collection.borrow();
            let entries = collection.entries();
            let new_events = decode_football_events(&entries[decoded_upto..]);
            decoded_upto = entries.len();
            new_events
        };
        football_events.extend(new_events.iter().cloned());

        let state = world.game().state();
        if let Some(criterion) = criteria.iter().find(|criterion| {
            criterion.is_satisfied(&state.stage, state.elapsed_time, steps, &new_events)
        }) {
            break criterion.to_reason();
        }
    };

    if let Some((index, error)) = world
        .game()
        .state()
        .player_states
        .iter()
        .enumerate()
        .find_map(|(index, player)| player.last_error.as_ref().map(|error| (index, error)))
    {
        return Err(ScenarioError::new(format!(
            "player {index} reported an error: {error}"
        )));
    }

    world.game_mut().finish_journal()?;
    let journal = collection.borrow().entries().to_vec();
    let final_state = world.game().state().clone();

    Ok(RunOutcome {
        journal,
        football_events,
        stop_reason,
        steps,
        final_state,
    })
}

fn compile_criteria(run: &RunPlanDto) -> Result<Vec<StopCriterion>, ScenarioError> {
    run.stop
        .iter()
        .map(|criterion| match criterion {
            StopCriterionDto::Stage {
                stage,
                setup_reason,
            } => Ok(StopCriterion::OnStage(stage_matcher(
                *stage,
                setup_reason.clone(),
            ))),
            StopCriterionDto::Event { event, team } => Ok(StopCriterion::OnEvent(
                EventMatcher::from_def(*event, *team).map_err(ScenarioError::new)?,
            )),
            StopCriterionDto::Steps { steps } => Ok(StopCriterion::OnSteps(*steps)),
            StopCriterionDto::Time { time } => Ok(StopCriterion::OnTime(*time)),
        })
        .collect()
}

#[cfg(test)]
#[path = "tests/runner_tests.rs"]
mod tests;
