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

    draw_score(panel_x, y_offset, game_state);
    y_offset += line_height * 2.0;

    draw_text("Space - pause/resume", panel_x, y_offset, 20.0, LIGHTGRAY);
    y_offset += line_height * 2.0;

    draw_player_decisions_table(panel_x, y_offset, game_config, game_state);
}

fn draw_score(x: f32, y: f32, game_state: &GameState) {
    use ynwa_core::team::Team;
    let score_a = game_state.team_stats
        .get(&Team::A)
        .map_or(0.0, |s| s.get("score")) as u32;
    let score_b = game_state.team_stats
        .get(&Team::B)
        .map_or(0.0, |s| s.get("score")) as u32;
    draw_text(
        &format!("Score:  A {}  :  {} B", score_a, score_b),
        x,
        y,
        24.0,
        WHITE,
    );
}

fn player_decision_text(
    player_state: &ynwa_core::game::PlayerState,
) -> String {
    match &player_state.current_decision {
        Some(Decision::Run(target)) => match target {
            DecisionTarget::Region(region) => {
                format!("Run to {:?}", region.to_grid_notation())
            }
            DecisionTarget::GridCell(cell) => {
                let col_label = ynwa_core::GridCell::column_to_label(cell.col);
                format!("Run to {}{}", col_label, cell.row)
            }
            DecisionTarget::Point(_) => "Run to point".to_string(),
            DecisionTarget::Ball => "Chase ball".to_string(),
        },
        Some(Decision::Stop) => "Stop".to_string(),
        Some(Decision::Kick(_)) => "Kick".to_string(),
        None => {
            if let Some(error) = &player_state.last_error {
                format!("ERROR: {}", error)
            } else {
                "—".to_string()
            }
        }
    }
}

fn draw_player_column(
    x: f32,
    start_y: f32,
    players: &[(usize, &ynwa_core::game::PlayerDef)],
    game_state: &GameState,
    label: &str,
) {
    let line_height = 26.0;
    let sub_line_height = 20.0;
    let row_gap = 4.0;
    let header_color = Color::new(0.9, 0.9, 0.9, 1.0);
    let text_color = Color::new(0.8, 0.8, 0.8, 1.0);
    let meta_color = Color::new(0.55, 0.55, 0.55, 1.0);

    let mut y = start_y;
    draw_text(label, x, y, 22.0, header_color);
    y += line_height;
    draw_text("#", x, y, 20.0, header_color);
    draw_text("Decision", x + 36.0, y, 20.0, header_color);
    y += line_height;

    for (i, player_def) in players {
        let player_state = &game_state.player_states[*i];

        draw_text(&player_def.number.to_string(), x, y, 20.0, text_color);
        draw_text(&player_decision_text(player_state), x + 36.0, y, 20.0, text_color);
        y += line_height;

        let time_str = if player_state.current_decision.is_some() {
            format!("t={:.1}s", player_state.last_decision_time)
        } else {
            "t=—".to_string()
        };
        let meta_text = if let Some(reason) = &player_state.decision_reason {
            format!("  {} | {}", time_str, reason)
        } else {
            format!("  {}", time_str)
        };
        draw_text(&meta_text, x + 36.0, y, 16.0, meta_color);
        y += sub_line_height + row_gap;

        if y > screen_height() - 20.0 {
            break;
        }
    }
}

fn draw_player_decisions_table(
    x: f32,
    start_y: f32,
    game_config: &GameConfig,
    game_state: &GameState,
) {
    let header_color = Color::new(0.9, 0.9, 0.9, 1.0);
    draw_text("Player Decisions:", x, start_y, 22.0, header_color);

    // Split players into two teams by their team field
    let team_a: Vec<(usize, &ynwa_core::game::PlayerDef)> = game_config
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.team == ynwa_core::team::Team::A)
        .collect();
    let team_b: Vec<(usize, &ynwa_core::game::PlayerDef)> = game_config
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.team == ynwa_core::team::Team::B)
        .collect();

    let col_width = (screen_width() - x) / 2.0 - 10.0;
    let table_y = start_y + 30.0;

    draw_player_column(x, table_y, &team_a, game_state, "Team A");
    draw_player_column(x + col_width, table_y, &team_b, game_state, "Team B");
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
