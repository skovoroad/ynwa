//! Helpers shared by unit tests.

use crate::rng::{DefaultRngManager, RngConfig, RngManager};
use std::cell::RefCell;

/// Deterministic manager: zero variation, fixed seed.
pub(crate) fn deterministic_rng() -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(42))))
}

/// Draws raw values from a fixed sequence, repeated cyclically.
pub(crate) struct SequenceRngManager {
    values: Vec<f32>,
    index: RefCell<usize>,
}

impl SequenceRngManager {
    pub(crate) fn new(values: Vec<f32>) -> Self {
        assert!(
            !values.is_empty(),
            "sequence must contain at least one value"
        );
        Self {
            values,
            index: RefCell::new(0),
        }
    }

    fn next_raw(&self) -> f32 {
        let mut index = self.index.borrow_mut();
        let value = self.values[*index % self.values.len()];
        *index += 1;
        value
    }
}

impl RngManager for SequenceRngManager {
    fn next(&self) -> f32 {
        self.next_raw()
    }

    fn randomize(&self, base: f32, variation_pct: f32) -> f32 {
        base * (1.0 + (self.next_raw() - 0.5) * 2.0 * variation_pct)
    }

    fn randomize_range(&self, max: f32) -> f32 {
        (self.next_raw() - 0.5) * 2.0 * max
    }
}
