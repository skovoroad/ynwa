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

    let field_length = field.length().get::<meter>();
    let field_width = field.width().get::<meter>();

    // Calculate scale with margins
    let margin = 50.0;
    let scale_x = (screen_w - 2.0 * margin) / field_length;
    let scale_y = (screen_h - 2.0 * margin) / field_width;
    let scale = scale_x.min(scale_y);

    // Convert field coords to screen coords
    let to_screen_x = |x: f32| margin + x * scale;
    let to_screen_y = |z: f32| margin + z * scale;

    let white = Color::new(1.0, 1.0, 1.0, 1.0);

    // Draw field boundary
    draw_rectangle_lines(
        to_screen_x(0.0),
        to_screen_y(0.0),
        field_length * scale,
        field_width * scale,
        2.0,
        white,
    );

    // Draw center line
    let half_length = field_length / 2.0;
    draw_line(
        to_screen_x(half_length),
        to_screen_y(0.0),
        to_screen_x(half_length),
        to_screen_y(field_width),
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

                draw_rectangle_lines(
                    to_screen_x(min_x),
                    to_screen_y(min_z),
                    (max_x - min_x) * scale,
                    (max_z - min_z) * scale,
                    1.0,
                    white,
                );
            }
            ZoneGeometry::Circle(circle) => {
                let cx = circle.center.x.get::<meter>();
                let cz = circle.center.z.get::<meter>();
                let radius = circle.radius.get::<meter>();

                draw_circle_lines(to_screen_x(cx), to_screen_y(cz), radius * scale, 1.0, white);
            }
            ZoneGeometry::Point(point) => {
                let px = point.position.x.get::<meter>();
                let pz = point.position.z.get::<meter>();

                draw_circle(to_screen_x(px), to_screen_y(pz), 3.0, white);
            }
            ZoneGeometry::Arc(_) => {
                // Skip arcs for minimal version
            }
        }
    }
}
