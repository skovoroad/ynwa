/// Simulation state and control
pub struct SimulationControl {
    pub rate: f32,
    pub paused: bool,
    pub accumulator: f32,
}

impl SimulationControl {
    pub fn new(rate: f32) -> Self {
        Self {
            rate,
            paused: false,
            accumulator: 0.0,
        }
    }

    pub fn delta(&self) -> f32 {
        1.0 / self.rate
    }

    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    pub fn increase_rate(&mut self) {
        self.rate = (self.rate * 2.0).min(100.0);
    }

    pub fn decrease_rate(&mut self) {
        self.rate = (self.rate / 2.0).max(1.0);
    }

    pub fn accumulate(&mut self, delta_time: f32) {
        if !self.paused {
            self.accumulator += delta_time;
        }
    }

    pub fn should_step(&self) -> bool {
        self.accumulator >= self.delta()
    }

    pub fn consume_step(&mut self) {
        self.accumulator -= self.delta();
    }
}
