/// Simulation state and control.
///
/// Real-time tempo (`rate`, frames per second used to accumulate frame time) is separated from
/// the simulation step (`step_delta`): replay runs in real time but advances the recorded step.
pub struct SimulationControl {
    rate: f32,
    step_delta: f32,
    pub paused: bool,
    accumulator: f32,
    steps_done: u64,
    max_steps: Option<u64>,
}

impl SimulationControl {
    pub fn new(rate: f32) -> Self {
        Self {
            rate,
            step_delta: 1.0 / rate,
            paused: false,
            accumulator: 0.0,
            steps_done: 0,
            max_steps: None,
        }
    }

    /// Replay runs in real time (`fixed_dt = 1/60` means 60 fps) but steps by the recorded `fixed_dt`.
    pub fn for_replay(fixed_dt: f32, total_steps: u64) -> Self {
        Self {
            rate: 1.0 / fixed_dt,
            step_delta: fixed_dt,
            paused: false,
            accumulator: 0.0,
            steps_done: 0,
            max_steps: Some(total_steps),
        }
    }

    pub fn step_delta(&self) -> f32 {
        self.step_delta
    }

    /// Frame-time interval that must accumulate before one step is due.
    pub fn pace(&self) -> f32 {
        1.0 / self.rate
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn increase_rate(&mut self) {
        self.set_rate((self.rate * 2.0).min(100.0));
    }

    pub fn decrease_rate(&mut self) {
        self.set_rate((self.rate / 2.0).max(1.0));
    }

    fn set_rate(&mut self, rate: f32) {
        self.rate = rate;
        self.step_delta = 1.0 / rate;
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
