/// Simulation state and control.
///
/// The simulation step (`timestep`) is fixed; `speed` only scales wall-clock pacing.
pub struct SimulationControl {
    timestep: f32,
    speed: f32,
    pub paused: bool,
    accumulator: f32,
    steps_done: u64,
    max_steps: Option<u64>,
}

/// Slowest playback speed the user can select.
const MIN_SPEED: f32 = 0.25;

/// Fastest playback speed the user can select.
const MAX_SPEED: f32 = 8.0;

impl SimulationControl {
    pub fn new(rate: f32) -> Self {
        Self::with_timestep(1.0 / rate, None)
    }

    pub fn for_replay(timestep: f32, total_steps: u64) -> Self {
        Self::with_timestep(timestep, Some(total_steps))
    }

    fn with_timestep(timestep: f32, max_steps: Option<u64>) -> Self {
        Self {
            timestep,
            speed: 1.0,
            paused: false,
            accumulator: 0.0,
            steps_done: 0,
            max_steps,
        }
    }

    pub fn timestep(&self) -> f32 {
        self.timestep
    }

    pub fn steps_done(&self) -> u64 {
        self.steps_done
    }

    /// Moves the playback position to `step`, clamping to `max_steps` when set, and clears any
    /// pending frame time so the next step starts fresh.
    pub fn reposition(&mut self, step: u64) {
        self.steps_done = match self.max_steps {
            Some(max) => step.min(max),
            None => step,
        };
        self.accumulator = 0.0;
    }

    /// Frame-time interval that must accumulate before one step is due.
    pub fn pace(&self) -> f32 {
        self.timestep
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn increase_speed(&mut self) {
        self.speed = (self.speed * 2.0).min(MAX_SPEED);
    }

    pub fn decrease_speed(&mut self) {
        self.speed = (self.speed / 2.0).max(MIN_SPEED);
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn accumulate(&mut self, delta_time: f32) {
        if self.paused || self.finished() {
            return;
        }
        self.accumulator += delta_time;
    }

    pub fn should_step(&self) -> bool {
        !self.finished() && self.accumulator >= self.pace()
    }

    pub fn consume_step(&mut self) {
        self.accumulator -= self.pace();
        self.steps_done += 1;
    }

    fn finished(&self) -> bool {
        self.max_steps.is_some_and(|max| self.steps_done >= max)
    }
}

#[cfg(test)]
#[path = "simulation_tests.rs"]
mod tests;
