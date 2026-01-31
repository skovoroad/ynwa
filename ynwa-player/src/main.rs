use macroquad::prelude::*;
use ynwa_core::create_football_game_config;
use ynwa_core::field::zones::ZoneGeometry;
use uom::si::length::meter;

fn window_conf() -> Conf {
    Conf {
        window_title: "YNWA - Football Manager".to_owned(),
        window_width: 1000,
        window_height: 700,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let game_config = create_football_game_config();
    let field = &game_config.field;
    
    // Calculate field proportions from actual dimensions
    let field_width_ratio = field.width().get::<meter>() / field.length().get::<meter>();

    loop {
        let screen_h = screen_height();
        
        // Field area: maintain natural proportions based on screen height
        let margin = 20.0;
        let field_render_height = screen_h - 2.0 * margin;
        let field_render_width = field_render_height * field_width_ratio;
        let field_area_width = field_render_width + 2.0 * margin;

        // Gray background for control panel (rest of screen)
        clear_background(Color::new(0.3, 0.3, 0.3, 1.0));

        // Green field area (left side, based on natural proportions)
        draw_rectangle(0.0, 0.0, field_area_width, screen_h, Color::new(0.13, 0.55, 0.13, 1.0));

        // Render field
        render_field(field, field_area_width, screen_h);

        next_frame().await
    }
}

fn render_field(field: &ynwa_core::field::Field, field_area_width: f32, screen_h: f32) {
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
}
