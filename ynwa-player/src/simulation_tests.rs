use super::SimulationControl;

const RATE: f32 = 60.0;

fn accumulator(simulation: &SimulationControl) -> f32 {
    simulation.accumulator
}

#[test]
fn increase_speed_doubles_and_clamps_to_max() {
    let cases = [(1, 2.0), (2, 4.0), (3, 8.0), (4, 8.0)];

    for (presses, expected) in cases {
        let mut simulation = SimulationControl::new(RATE);
        for _ in 0..presses {
            simulation.increase_speed();
        }
        assert_eq!(simulation.speed(), expected, "{presses} presses");
    }
}

#[test]
fn decrease_speed_halves_and_clamps_to_min() {
    let cases = [(1, 0.5), (2, 0.25), (3, 0.25)];

    for (presses, expected) in cases {
        let mut simulation = SimulationControl::new(RATE);
        for _ in 0..presses {
            simulation.decrease_speed();
        }
        assert_eq!(simulation.speed(), expected, "{presses} presses");
    }
}

#[test]
fn accumulate_adds_frame_time_unaffected_by_speed() {
    let cases = [(0, 0.5), (1, 0.5), (3, 0.5)];

    for (presses, expected) in cases {
        let mut simulation = SimulationControl::new(RATE);
        for _ in 0..presses {
            simulation.increase_speed();
        }
        simulation.accumulate(0.25);
        simulation.accumulate(0.25);
        assert_eq!(accumulator(&simulation), expected, "{presses} presses");
    }
}

#[test]
fn accumulate_adds_frame_time_in_replay_mode() {
    let mut simulation = SimulationControl::for_replay(1.0 / RATE, 10);
    simulation.increase_speed();
    simulation.accumulate(0.5);
    assert_eq!(accumulator(&simulation), 0.5);
}

#[test]
fn paused_simulation_ignores_accumulated_time() {
    let mut simulation = SimulationControl::new(RATE);
    simulation.increase_speed();
    simulation.toggle_pause();
    simulation.accumulate(0.5);
    assert_eq!(accumulator(&simulation), 0.0);
}
