use macroquad::prelude::*;

use crate::simulation::SimulationControl;

pub fn handle_input(simulation: &mut SimulationControl, is_fullscreen: &mut bool) -> bool {
    if is_key_pressed(KeyCode::Escape) {
        return true; // Exit
    }

    if is_key_pressed(KeyCode::F11) {
        *is_fullscreen = !*is_fullscreen;
        set_fullscreen(*is_fullscreen);
    }

    if is_key_pressed(KeyCode::Space) {
        simulation.toggle_pause();
    }

    if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
        simulation.increase_rate();
    }

    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        simulation.decrease_rate();
    }

    false // Continue
}
