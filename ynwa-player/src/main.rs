use macroquad::prelude::*;
use std::path::Path;
use uom::si::length::meter;
use ynwa_core::create_football_world_from_file;
use ynwa_core::field::zones::ZoneGeometry;

fn window_conf() -> Conf {
    Conf {
        window_title: "YNWA - Football Manager".to_owned(),
        fullscreen: true, // Start in fullscreen mode by default
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let config_path = Path::new("config/default_game.toml");
    let mut world =
        create_football_world_from_file(config_path).expect("Failed to load game configuration");

    println!(
        "Loaded game with {} players",
        world.game().config().players.len()
    );

    let field = &world.game().config().field;
    let field_width_ratio = field.width().get::<meter>() / field.length().get::<meter>();

    let mut is_fullscreen = false;
    set_fullscreen(is_fullscreen);

    // Simulation parameters
    let mut simulation_rate: f32 = 10.0; // steps per second
    let mut simulation_paused = false;
    let mut accumulator = 0.0;

    loop {
        if is_key_pressed(KeyCode::Escape) {
            break;
        }

        if is_key_pressed(KeyCode::F11) {
            is_fullscreen = !is_fullscreen;
            set_fullscreen(is_fullscreen);
        }

        // Toggle pause with Space
        if is_key_pressed(KeyCode::Space) {
            simulation_paused = !simulation_paused;
        }

        // Adjust simulation rate with +/-
        if is_key_pressed(KeyCode::Equal) || is_key_pressed(KeyCode::KpAdd) {
            simulation_rate = (simulation_rate * 2.0).min(100.0);
        }
        if is_key_pressed(KeyCode::Minus) || is_key_pressed(KeyCode::KpSubtract) {
            simulation_rate = (simulation_rate / 2.0).max(1.0);
        }

        // Accumulate frame time for fixed timestep simulation
        if !simulation_paused {
            accumulator += get_frame_time();
        }

        // Step simulation at fixed rate
        let simulation_delta = 1.0 / simulation_rate;
        while accumulator >= simulation_delta {
            world.step(simulation_delta);
            accumulator -= simulation_delta;
        }

        // Screen layout
        let screen_w = screen_width();
        let screen_h = screen_height();
        
        // Calculate field area (left side with padding)
        let margin = 20.0;
        let available_height = screen_h - 2.0 * margin;
        let available_width = available_height * field_width_ratio;
        let field_area_width = available_width + 2.0 * margin;
        
        // Control panel (right side)
        let control_panel_x = field_area_width;
        let control_panel_width = screen_w - field_area_width;

        // Clear background
        clear_background(Color::new(0.3, 0.3, 0.3, 1.0));

        // Green field area (left side)
        draw_rectangle(
            0.0,
            0.0,
            field_area_width,
            screen_h,
            Color::new(0.13, 0.55, 0.13, 1.0),
        );

        // Render field
        render_field(world.game().config(), field_area_width, screen_h);

        // Separator line between field and control panel
        draw_line(
            field_area_width,
            0.0,
            field_area_width,
            screen_h,
            2.0,
            Color::new(0.2, 0.2, 0.2, 1.0),
        );

        // Control panel info (top of right panel)
        if control_panel_width > 50.0 {
            let elapsed_time = world.game().state().elapsed_time;
            let status = if simulation_paused { "PAUSED" } else { "Running" };
            
            let panel_x = control_panel_x + 20.0;
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
            
            draw_text(
                &format!("Status: {}", status),
                panel_x,
                y_offset,
                24.0,
                if simulation_paused { YELLOW } else { GREEN },
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

        next_frame().await
    }
}

fn render_field(game_config: &ynwa_core::game::GameConfig, field_area_width: f32, screen_h: f32) {
    let field = &game_config.field;
    let field_length = field.length().get::<meter>();
    let field_width = field.width().get::<meter>();

    // Calculate scale to fit in field area
    let margin = 20.0;
    let scale_x = (field_area_width - 2.0 * margin) / field_width;
    let scale_y = (screen_h - 2.0 * margin) / field_length;
    let scale = scale_x.min(scale_y);

    // Center field in area
    let field_render_width = field_width * scale;
    let field_render_height = field_length * scale;
    let offset_x = (field_area_width - field_render_width) / 2.0;
    let offset_y = (screen_h - field_render_height) / 2.0;

    // Convert field coords to screen coords
    let to_screen_x = |field_z: f32| offset_x + field_z * scale;
    let to_screen_y = |field_x: f32| offset_y + field_x * scale;

    let white = Color::new(1.0, 1.0, 1.0, 1.0);

    // Draw field boundary (vertical)
    draw_rectangle_lines(
        to_screen_x(0.0),
        to_screen_y(0.0),
        field_width * scale,
        field_length * scale,
        2.0,
        white,
    );

    // Draw grid cells in checkerboard pattern
    let grid_dims = field.grid_dimensions();
    let cell_size = field.cell_size();

    let green_light = Color::new(0.14, 0.56, 0.14, 1.0);
    let green_dark = Color::new(0.12, 0.54, 0.12, 1.0);

    for row in 0..grid_dims.rows {
        for col in 0..grid_dims.columns {
            // Checkerboard pattern: alternate based on sum of indices
            let is_light = (row + col) % 2 == 0;
            let color = if is_light { green_light } else { green_dark };

            // Cell position in field coordinates (0-based for drawing)
            let cell_x = row as f32 * cell_size;
            let cell_z = col as f32 * cell_size;

            // Draw filled rectangle
            draw_rectangle(
                to_screen_x(cell_z),
                to_screen_y(cell_x),
                cell_size * scale,
                cell_size * scale,
                color,
            );
        }
    }

    // Draw grid labels
    let label_color = Color::new(0.9, 0.9, 0.9, 0.8);
    let font_size = 12.0;

    // Column labels (A, B, C, ... at top)
    for col in 0..grid_dims.columns {
        let col_label = ynwa_core::GridCell::column_to_label(col + 1); // 1-based
        let cell_z = col as f32 * cell_size;
        let x_pos = to_screen_x(cell_z + cell_size / 2.0);
        let y_pos = to_screen_y(0.0) - 5.0; // Above field

        let text_dims = measure_text(&col_label, None, font_size as u16, 1.0);
        draw_text(
            &col_label,
            x_pos - text_dims.width / 2.0,
            y_pos,
            font_size,
            label_color,
        );
    }

    // Row labels (1, 2, 3, ... at left)
    for row in 0..grid_dims.rows {
        let row_label = (row + 1).to_string(); // 1-based
        let cell_x = row as f32 * cell_size;
        let x_pos = to_screen_x(0.0) - 5.0; // Left of field
        let y_pos = to_screen_y(cell_x + cell_size / 2.0);

        let text_dims = measure_text(&row_label, None, font_size as u16, 1.0);
        draw_text(
            &row_label,
            x_pos - text_dims.width,
            y_pos + text_dims.height / 2.0 - 2.0,
            font_size,
            label_color,
        );
    }

    // Draw center line (horizontal)
    let half_length = field_length / 2.0;
    draw_line(
        to_screen_x(0.0),
        to_screen_y(half_length),
        to_screen_x(field_width),
        to_screen_y(half_length),
        1.0,
        white,
    );

    // Draw zones
    for ((_name, _team), zone) in field.zones() {
        match &zone.geometry {
            ZoneGeometry::Rectangle(rect) => {
                let min_x = rect.min.x.get::<meter>();
                let min_z = rect.min.z.get::<meter>();
                let max_x = rect.max.x.get::<meter>();
                let max_z = rect.max.z.get::<meter>();

                // Z→screenX, X→screenY
                draw_rectangle_lines(
                    to_screen_x(min_z),
                    to_screen_y(min_x),
                    (max_z - min_z) * scale,
                    (max_x - min_x) * scale,
                    1.0,
                    white,
                );
            }
            ZoneGeometry::Circle(circle) => {
                let cx = circle.center.x.get::<meter>();
                let cz = circle.center.z.get::<meter>();
                let radius = circle.radius.get::<meter>();

                draw_circle_lines(to_screen_x(cz), to_screen_y(cx), radius * scale, 1.0, white);
            }
            ZoneGeometry::Point(point) => {
                let px = point.position.x.get::<meter>();
                let pz = point.position.z.get::<meter>();

                draw_circle(to_screen_x(pz), to_screen_y(px), 3.0, white);
            }
            ZoneGeometry::Arc(_) => {
                // Skip arcs for minimal version
            }
        }
    }

    // Draw players
    let player_radius = 8.0;
    let team_a_color = RED;
    let team_b_color = BLUE;
    let text_color = WHITE;

    for player_def in &game_config.players {
        // Get player's start position region
        if let Some(start_region) = player_def.regions.get("start position") {
            // Get center of the region
            let center = start_region.center(field.grid_dimensions(), field.width().get::<meter>());
            let px = center.x.get::<meter>();
            let pz = center.z.get::<meter>();

            // Choose color based on team
            let color = match player_def.team {
                ynwa_core::team::Team::A => team_a_color,
                ynwa_core::team::Team::B => team_b_color,
            };

            // Draw player circle
            draw_circle(to_screen_x(pz), to_screen_y(px), player_radius, color);

            // Draw player number
            let number_text = player_def.number.to_string();
            let font_size = 14.0;
            let text_dims = measure_text(&number_text, None, font_size as u16, 1.0);

            draw_text(
                &number_text,
                to_screen_x(pz) - text_dims.width / 2.0,
                to_screen_y(px) + text_dims.height / 2.0 - 2.0,
                font_size,
                text_color,
            );
        }
    }
}
