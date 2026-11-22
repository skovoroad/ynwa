use super::SimulationControl;

const RATE: f32 = 60.0;

fn accumulator(simulation: &SimulationControl) -> f32 {
    simulation.accumulator
}

#[test]
fn accumulate_uses_normal_speed_by_default() {
    let cases = [(0.5, 0.5), (0.25, 0.25)];

    for (delta_time, expected) in cases {
        let mut simulation = SimulationControl::new(RATE);
        simulation.accumulate(delta_time);
        assert_eq!(accumulator(&simulation), expected);
    }
}

#[test]
fn accumulate_scales_frame_time_by_speed() {
    let cases = [
        (4.0, 0.5, 2.0),
        (2.0, 0.25, 0.5),
        (4.0, 0.125, 0.5),
        (1.0, 0.5, 0.5),
    ];

    for (speed, delta_time, expected) in cases {
        let mut simulation = SimulationControl::new(RATE);
        simulation.set_speed(speed);
        simulation.accumulate(delta_time);
        assert_eq!(
            accumulator(&simulation),
            expected,
            "speed {speed}, delta {delta_time}"
        );
    }
}

#[test]
fn accumulate_applies_speed_in_replay_mode() {
    let mut simulation = SimulationControl::for_replay(1.0 / RATE, 10);
    simulation.set_speed(4.0);
    simulation.accumulate(0.5);
    assert_eq!(accumulator(&simulation), 2.0);
}

#[test]
fn scaled_accumulation_raises_should_step() {
    let mut simulation = SimulationControl::new(RATE);
    simulation.set_speed(4.0);
    simulation.accumulate(simulation.pace() / 4.0);
    assert!(simulation.should_step());
}

#[test]
fn paused_simulation_ignores_speed() {
    let mut simulation = SimulationControl::new(RATE);
    simulation.set_speed(4.0);
    simulation.toggle_pause();
    simulation.accumulate(0.5);
    assert_eq!(accumulator(&simulation), 0.0);
}
