//! Helpers shared by unit tests.

use ynwa_core::rng::{DefaultRngManager, RngConfig, RngManager};

/// Deterministic manager: zero variation, fixed seed.
pub(crate) fn deterministic_rng() -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(42))))
}
