use macroquad::prelude::*;
use ynwa_core::football::create_football_field;
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
    let field = create_football_field();

    loop {
        // Green field background
        clear_background(Color::new(0.13, 0.55, 0.13, 1.0));

        // Render field
        render_field(&field);

        next_frame().await
    }
}

fn render_field(field: &ynwa_core::field::Field) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let field_length = field.length().get::<meter>(); // 100m (Team A → Team B)
    let field_width = field.width().get::<meter>();   // 60m

    // Calculate scale with margins
    let margin = 50.0;
    // Vertical orientation: width→screenX, length→screenY
    let scale_x = (screen_w - 2.0 * margin) / field_width;
    let scale_y = (screen_h - 2.0 * margin) / field_length;
    let scale = scale_x.min(scale_y);

    // Convert field coords to screen coords
    // Field Z (width, cross-field) → Screen X
    // Field X (length, Team A→B) → Screen Y
    let to_screen_x = |field_z: f32| margin + field_z * scale;
    let to_screen_y = |field_x: f32| margin + field_x * scale;

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
