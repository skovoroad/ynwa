//! Helpers shared by unit tests.

use std::cell::RefCell;
use std::rc::Rc;
use ynwa_core::game::Game;
use ynwa_core::journal::{CollectJournalRecorder, EventsCollection};
use ynwa_core::rng::{DefaultRngManager, RngConfig, RngManager};

/// Deterministic manager: zero variation, fixed seed.
pub(crate) fn deterministic_rng() -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(42))))
}

/// Attaches an accumulating journal recorder to the game and returns its shared collection.
pub(crate) fn attach_journal(game: &mut Game) -> Rc<RefCell<EventsCollection>> {
    let collection = Rc::new(RefCell::new(EventsCollection::default()));
    game.set_journal_sink(Box::new(CollectJournalRecorder::new(Rc::clone(
        &collection,
    ))));
    collection
}
