use macroquad::prelude::*;
use ynwa_core::game::{Decision, DecisionTarget, GameConfig, GameState};

pub fn draw_control_panel(
    panel_x: f32,
    game_config: &GameConfig,
    game_state: &GameState,
    is_paused: bool,
) {
    let panel_x = panel_x + 20.0;
    let mut y_offset = 40.0;
    let line_height = 30.0;

    draw_text(
        &format!("Time: {:.1}s", game_state.elapsed_time),
        panel_x,
        y_offset,
        24.0,
        WHITE,
    );
    y_offset += line_height;

    let status = if is_paused { "PAUSED" } else { "Running" };
    let status_color = if is_paused { YELLOW } else { GREEN };
    draw_text(
        &format!("Status: {}", status),
        panel_x,
        y_offset,
        24.0,
        status_color,
    );
    y_offset += line_height * 1.5;

    draw_text("Space - pause/resume", panel_x, y_offset, 20.0, LIGHTGRAY);
    y_offset += line_height * 2.0;

    draw_player_decisions_table(panel_x, y_offset, game_config, game_state);
}

fn draw_player_decisions_table(
    x: f32,
    start_y: f32,
    game_config: &GameConfig,
    game_state: &GameState,
) {
    let mut y = start_y;
    let line_height = 20.0;
    let header_color = Color::new(0.9, 0.9, 0.9, 1.0);
    let text_color = Color::new(0.8, 0.8, 0.8, 1.0);

    draw_text("Player Decisions:", x, y, 20.0, header_color);
    y += line_height * 1.5;

    draw_text("#", x, y, 16.0, header_color);
    draw_text("Decision", x + 30.0, y, 16.0, header_color);
    draw_text("Time", x + 250.0, y, 16.0, header_color);
    y += line_height;

    for (i, player_def) in game_config.players.iter().enumerate() {
        let player_state = &game_state.player_states[i];

        let number_text = format!("{}", player_def.number);
        draw_text(&number_text, x, y, 14.0, text_color);

        let decision_text = match &player_state.current_decision {
            Some(Decision::Run(target)) => match target {
                DecisionTarget::Region(region) => {
                    format!("Run to {:?}", region.to_grid_notation())
                }
                DecisionTarget::GridCell(cell) => {
                    let col_label = ynwa_core::GridCell::column_to_label(cell.col);
                    format!("Run to {}{}", col_label, cell.row)
                }
                DecisionTarget::Point(_) => "Run to point".to_string(),
            },
            Some(Decision::Stop) => "Stop".to_string(),
            Some(Decision::Kick(_)) => "Kick".to_string(),
            None => {
                // Check if there's an error to display
                if let Some(error) = &player_state.last_error {
                    format!("ERROR: {}", error)
                } else {
                    "—".to_string()
                }
            }
        };
        
        // Add decision reason if available
        let full_decision_text = if let Some(reason) = &player_state.decision_reason {
            format!("{} ({})", decision_text, reason)
        } else {
            decision_text
        };
        
        draw_text(&full_decision_text, x + 30.0, y, 14.0, text_color);

        let time_text = if player_state.current_decision.is_some() {
            format!("{:.1}s", player_state.last_decision_time)
        } else {
            "—".to_string()
        };
        draw_text(&time_text, x + 250.0, y, 14.0, text_color);

        y += line_height;

        if y > screen_height() - 20.0 {
            break;
        }
    }
}

pub fn draw_separator(x: f32, screen_height: f32) {
    draw_line(
        x,
        0.0,
        x,
        screen_height,
        2.0,
        Color::new(0.2, 0.2, 0.2, 1.0),
    );
}
