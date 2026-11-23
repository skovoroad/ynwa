use macroquad::prelude::*;

use crate::simulation::SimulationControl;
use crate::ui::EVENT_PANEL_SIZE;

/// Scroll position of the replay events panel, counted in rows back from the newest event.
#[derive(Default)]
pub struct EventPanelScroll {
    offset: usize,
}

impl EventPanelScroll {
    /// Rows scrolled back from the newest event; `0` follows newly arrived events.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Applies mouse-wheel and `Up`/`Down` input, clamped to the available history.
    /// Held arrow keys scroll continuously.
    fn update(&mut self, total_entries: usize) {
        let mut delta = 0i64;
        let (_, wheel) = mouse_wheel();
        if wheel > 0.0 {
            delta += 1;
        }
        if wheel < 0.0 {
            delta -= 1;
        }
        if is_key_down(KeyCode::Up) {
            delta += 1;
        }
        if is_key_down(KeyCode::Down) {
            delta -= 1;
        }
        if delta == 0 {
            return;
        }

        let max_offset = total_entries.saturating_sub(EVENT_PANEL_SIZE);
        self.offset = (self.offset as i64 + delta).clamp(0, max_offset as i64) as usize;
    }
}

/// Returns `true` when the user requested to close the window.
///
/// `event_scroll` is present only in replay, where it tracks the events panel position.
pub fn handle_input(
    simulation: &mut SimulationControl,
    is_fullscreen: &mut bool,
    event_scroll: Option<(&mut EventPanelScroll, usize)>,
) -> bool {
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
        simulation.increase_speed();
    }

    if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
        simulation.decrease_speed();
    }

    if let Some((scroll, total_entries)) = event_scroll {
        scroll.update(total_entries);
    }

    false // Continue
}
