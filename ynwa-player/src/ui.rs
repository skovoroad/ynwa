use macroquad::prelude::*;

pub fn draw_control_panel(
    panel_x: f32,
    elapsed_time: f32,
    is_paused: bool,
) {
    let panel_x = panel_x + 20.0;
    let mut y_offset = 40.0;
    let line_height = 30.0;
    
    draw_text(
        &format!("Time: {:.1}s", elapsed_time),
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
    
    draw_text(
        "Space - pause/resume",
        panel_x,
        y_offset,
        20.0,
        LIGHTGRAY,
    );
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
