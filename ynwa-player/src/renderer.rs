use macroquad::prelude::*;
use uom::si::length::meter;
use ynwa_core::field::zones::ZoneGeometry;

pub fn render_field(
    game_config: &ynwa_core::game::GameConfig,
    game_state: &ynwa_core::game::GameState,
    field_area_width: f32,
    screen_h: f32,
) {
    let field = &game_config.field;
    let field_length = field.length().get::<meter>();
    let field_width = field.width().get::<meter>();

    let margin = 20.0;
    let scale_x = (field_area_width - 2.0 * margin) / field_width;
    let scale_y = (screen_h - 2.0 * margin) / field_length;
    let scale = scale_x.min(scale_y);

    let field_render_width = field_width * scale;
    let field_render_height = field_length * scale;
    let offset_x = (field_area_width - field_render_width) / 2.0;
    let offset_y = (screen_h - field_render_height) / 2.0;

    let to_screen_x = |field_z: f32| offset_x + field_z * scale;
    let to_screen_y = |field_x: f32| offset_y + (field_length - field_x) * scale;

    let white = Color::new(1.0, 1.0, 1.0, 1.0);

    draw_field_boundary(
        &to_screen_x,
        &to_screen_y,
        field_width,
        field_length,
        scale,
        white,
    );
    draw_grid_cells(&to_screen_x, &to_screen_y, field, scale);
    draw_grid_labels(&to_screen_x, &to_screen_y, field, field_length);
    draw_center_line(&to_screen_x, &to_screen_y, field_width, field_length, white);
    draw_zones(&to_screen_x, &to_screen_y, field, scale, white);
    draw_players(&to_screen_x, &to_screen_y, game_config, game_state);
    draw_ball(&to_screen_x, &to_screen_y, game_state);
}

fn draw_field_boundary(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    field_width: f32,
    field_length: f32,
    scale: f32,
    color: Color,
) {
    draw_rectangle_lines(
        to_screen_x(0.0),
        to_screen_y(field_length),
        field_width * scale,
        field_length * scale,
        2.0,
        color,
    );
}

fn draw_grid_cells(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    field: &ynwa_core::field::Field,
    scale: f32,
) {
    let grid_dims = field.grid_dimensions();
    let cell_size = field.cell_size();

    let green_light = Color::new(0.14, 0.56, 0.14, 1.0);
    let green_dark = Color::new(0.12, 0.54, 0.12, 1.0);

    for row in 0..grid_dims.rows {
        for col in 0..grid_dims.columns {
            let is_light = (row + col) % 2 == 0;
            let color = if is_light { green_light } else { green_dark };

            let cell_x = col as f32 * cell_size;
            let cell_z = row as f32 * cell_size;

            draw_rectangle(
                to_screen_x(cell_x),
                to_screen_y(cell_z + cell_size),
                cell_size * scale,
                cell_size * scale,
                color,
            );
        }
    }
}

fn draw_grid_labels(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    field: &ynwa_core::field::Field,
    field_length: f32,
) {
    let grid_dims = field.grid_dimensions();
    let cell_size = field.cell_size();
    let label_color = Color::new(0.9, 0.9, 0.9, 0.8);
    let font_size = 12.0;

    for col in 0..grid_dims.columns {
        let col_label = ynwa_core::GridCell::column_to_label(col + 1);
        let cell_x = col as f32 * cell_size;
        let x_pos = to_screen_x(cell_x + cell_size / 2.0);
        let y_pos = to_screen_y(field_length) - 5.0;

        let text_dims = measure_text(&col_label, None, font_size as u16, 1.0);
        draw_text(
            &col_label,
            x_pos - text_dims.width / 2.0,
            y_pos,
            font_size,
            label_color,
        );
    }

    for row in 0..grid_dims.rows {
        let row_label = (row + 1).to_string();
        let cell_z = row as f32 * cell_size;
        let x_pos = to_screen_x(0.0) - 5.0;
        let y_pos = to_screen_y(cell_z + cell_size / 2.0);

        let text_dims = measure_text(&row_label, None, font_size as u16, 1.0);
        draw_text(
            &row_label,
            x_pos - text_dims.width,
            y_pos + text_dims.height / 2.0 - 2.0,
            font_size,
            label_color,
        );
    }
}

fn draw_center_line(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    field_width: f32,
    field_length: f32,
    color: Color,
) {
    let half_length = field_length / 2.0;
    draw_line(
        to_screen_x(0.0),
        to_screen_y(half_length),
        to_screen_x(field_width),
        to_screen_y(half_length),
        1.0,
        color,
    );
}

fn draw_zones(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    field: &ynwa_core::field::Field,
    scale: f32,
    color: Color,
) {
    for ((_name, _team), zone) in field.zones() {
        match &zone.geometry {
            ZoneGeometry::Rectangle(rect) => {
                let min_x = rect.min.x.get::<meter>();
                let min_z = rect.min.z.get::<meter>();
                let max_x = rect.max.x.get::<meter>();
                let max_z = rect.max.z.get::<meter>();

                draw_rectangle_lines(
                    to_screen_x(min_x),
                    to_screen_y(max_z),
                    (max_x - min_x) * scale,
                    (max_z - min_z) * scale,
                    1.0,
                    color,
                );
            }
            ZoneGeometry::Circle(circle) => {
                let cx = circle.center.x.get::<meter>();
                let cz = circle.center.z.get::<meter>();
                let radius = circle.radius.get::<meter>();

                draw_circle_lines(to_screen_x(cx), to_screen_y(cz), radius * scale, 1.0, color);
            }
            ZoneGeometry::Point(point) => {
                let px = point.position.x.get::<meter>();
                let pz = point.position.z.get::<meter>();

                draw_circle(to_screen_x(px), to_screen_y(pz), 3.0, color);
            }
            ZoneGeometry::Arc(_) => {}
        }
    }
}

fn draw_players(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    game_config: &ynwa_core::game::GameConfig,
    game_state: &ynwa_core::game::GameState,
) {
    let player_radius = 8.0;
    let team_a_color = RED;
    let team_b_color = BLUE;
    let text_color = WHITE;

    for (i, player_def) in game_config.players.iter().enumerate() {
        let player_state = &game_state.player_states[i];
        let px = player_state.position.x.get::<meter>();
        let pz = player_state.position.z.get::<meter>();

        let color = match player_def.team {
            ynwa_core::team::Team::A => team_a_color,
            ynwa_core::team::Team::B => team_b_color,
        };

        draw_circle(to_screen_x(px), to_screen_y(pz), player_radius, color);

        let number_text = player_def.number.to_string();
        let font_size = 14.0;
        let text_dims = measure_text(&number_text, None, font_size as u16, 1.0);

        draw_text(
            &number_text,
            to_screen_x(px) - text_dims.width / 2.0,
            to_screen_y(pz) + text_dims.height / 2.0 - 2.0,
            font_size,
            text_color,
        );
    }
}

fn draw_ball(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    game_state: &ynwa_core::game::GameState,
) {
    let ball_state = &game_state.ball_state;
    let bx = ball_state.position.x.get::<meter>();
    let bz = ball_state.position.z.get::<meter>();

    let ball_radius = 6.0;
    let ball_color = WHITE;
    let ball_outline_color = BLACK;

    draw_circle(
        to_screen_x(bx),
        to_screen_y(bz),
        ball_radius + 1.0,
        ball_outline_color,
    );
    draw_circle(to_screen_x(bx), to_screen_y(bz), ball_radius, ball_color);
}
