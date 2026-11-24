use macroquad::prelude::*;

use crate::simulation::SimulationControl;

/// Scroll position of the events panel, counted in rows back from the newest event.
#[derive(Default)]
pub struct EventPanelScroll {
    offset: usize,
}

impl EventPanelScroll {
    /// Rows scrolled back from the newest event; `0` follows newly arrived events.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Applies mouse-wheel, arrow, and page-key input, clamped to the available history.
    /// Held keys scroll continuously; page keys move by one viewport.
    fn update(&mut self, total_entries: usize, visible_rows: usize) {
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
        let page = visible_rows.max(1) as i64;
        if is_key_down(KeyCode::PageUp) {
            delta += page;
        }
        if is_key_down(KeyCode::PageDown) {
            delta -= page;
        }
        if delta == 0 {
            return;
        }

        let max_offset = total_entries.saturating_sub(visible_rows);
        self.offset = (self.offset as i64 + delta).clamp(0, max_offset as i64) as usize;
    }
}

/// Events panel scroll state together with the counts needed to clamp it.
pub struct EventPanelInput<'a> {
    pub scroll: &'a mut EventPanelScroll,
    pub total_entries: usize,
    pub visible_rows: usize,
}

/// Returns `true` when the user requested to close the window.
///
/// `event_panel` tracks the events panel position in every mode.
pub fn handle_input(
    simulation: &mut SimulationControl,
    is_fullscreen: &mut bool,
    event_panel: Option<EventPanelInput<'_>>,
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

    if let Some(panel) = event_panel {
        panel.scroll.update(panel.total_entries, panel.visible_rows);
    }

    false // Continue
}
