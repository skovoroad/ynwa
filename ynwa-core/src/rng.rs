//! Single source of randomness for all game systems.
//!
//! `temperature` scales the spread: `0.0` is fully deterministic, `1.0` allows the
//! full variation range. The playable game uses a non-zero temperature, tests use `0.0`.

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::cell::RefCell;

/// `seed = None` uses system entropy (playable game), `Some(seed)` makes the
/// sequence reproducible (tests).
///
/// Build it with [`RngConfig::new`], which validates `temperature`.
#[derive(Debug, Clone, Copy)]
pub struct RngConfig {
    pub temperature: f32,
    pub seed: Option<u64>,
}

impl RngConfig {
    /// # Panics
    /// Panics if `temperature` is outside `[0.0, 1.0]`.
    pub fn new(temperature: f32, seed: Option<u64>) -> Self {
        assert!(
            (0.0..=1.0).contains(&temperature),
            "temperature must be in [0.0, 1.0], got {}",
            temperature
        );
        Self { temperature, seed }
    }
}

/// Random source shared by all game systems.
///
/// Methods take `&self` so systems can draw values while holding a shared borrow of
/// `Game`; implementations provide interior mutability.
pub trait RngManager: Send {
    /// Raw value in `[0, 1]`. Always `0.5` at temperature `0.0`.
    fn next(&self) -> f32;

    /// `variation_pct = 0.25` means ±25% around `base`.
    fn randomize(&self, base: f32, variation_pct: f32) -> f32;

    /// Symmetric deviation in `[-max, +max]`.
    fn randomize_range(&self, max: f32) -> f32;
}

pub struct DefaultRngManager {
    rng: RefCell<StdRng>,
    temperature: f32,
}

impl DefaultRngManager {
    pub fn new(config: RngConfig) -> Self {
        let seed = config.seed.unwrap_or_else(rand::random);
        Self {
            rng: RefCell::new(StdRng::seed_from_u64(seed)),
            temperature: config.temperature,
        }
    }
}

impl RngManager for DefaultRngManager {
    fn next(&self) -> f32 {
        if self.temperature == 0.0 {
            return 0.5;
        }
        self.rng.borrow_mut().random()
    }

    fn randomize(&self, base: f32, variation_pct: f32) -> f32 {
        if self.temperature == 0.0 || variation_pct == 0.0 {
            return base;
        }
        base * (1.0 + self.temperature * ((self.next() - 0.5) * 2.0 * variation_pct))
    }

    fn randomize_range(&self, max: f32) -> f32 {
        if self.temperature == 0.0 || max == 0.0 {
            return 0.0;
        }
        self.temperature * (self.next() - 0.5) * 2.0 * max
    }
}

#[cfg(test)]
#[path = "tests/rng_tests.rs"]
mod tests;
