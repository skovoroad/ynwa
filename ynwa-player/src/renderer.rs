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
    use uom::si::velocity::meter_per_second;

    for (i, player_def) in game_config.players.iter().enumerate() {
        let player_state = &game_state.player_states[i];
        let px = player_state.position.x.get::<meter>();
        let pz = player_state.position.z.get::<meter>();
        let vz = player_state.velocity.z.get::<meter_per_second>();
        let vx = player_state.velocity.x.get::<meter_per_second>();
        let speed = (vx * vx + vz * vz).sqrt();

        let shirt_color = match player_def.team {
            ynwa_core::team::Team::A => Color::new(0.85, 0.1, 0.1, 1.0),
            ynwa_core::team::Team::B => Color::new(0.0, 0.45, 1.0, 1.0),
        };

        // velocity.z > 0 => moving toward +Z => up on screen => back to viewer
        let facing_back = vz > 0.5 || (vz.abs() <= 0.5 && vx == 0.0 && vz == 0.0);
        let has_ball = game_state.ball_state.possessed_by == Some(i);

        // Animation: 3-frame pixel-art style (0 = standing, 1 = left leg up, 2 = right leg up)
        // Switches frame every 0.2 seconds
        let anim_frame: i32 = if speed > 0.5 {
            ((game_state.elapsed_time / 0.2) as i32 % 2) + 1
        } else {
            0
        };

        draw_player_sprite(
            to_screen_x(px),
            to_screen_y(pz),
            shirt_color,
            facing_back,
            anim_frame,
            has_ball,
            player_def.number,
        );
    }
}

/// Draws a pixel-art style player sprite centered at (cx, cy).
/// anim_frame: 0 = standing, 1 = left leg up, 2 = right leg up
fn draw_player_sprite(
    cx: f32,
    cy: f32,
    shirt_color: Color,
    facing_back: bool,
    anim_frame: i32,
    has_ball: bool,
    number: u32,
) {
    let skin_color  = Color::new(0.95, 0.78, 0.62, 1.0);
    let shorts_color = Color::new(0.15, 0.15, 0.15, 1.0);
    let sock_color  = Color::new(0.9, 0.9, 0.9, 1.0);
    let shoe_color  = Color::new(0.1, 0.1, 0.1, 1.0);
    let hair_color  = Color::new(0.2, 0.12, 0.05, 1.0);

    // Pixel size — all coordinates are multiples of P for chunky look
    let p = 2.0_f32;

    // Layout (top of sprite = cy - 8*p, bottom = cy + 6*p)
    // Head: 3x3 pixels, centered
    let head_x = cx - p * 1.5;
    let head_y = cy - p * 8.0;
    let head_w = p * 3.0;
    let head_h = p * 3.0;

    // Torso: 5x3 pixels — wide, short
    let torso_x = cx - p * 2.5;
    let torso_y = cy - p * 5.0;
    let torso_w = p * 5.0;
    let torso_h = p * 3.0;

    // Shorts: 5x1
    let shorts_y = torso_y + torso_h;
    let shorts_h = p * 1.0;

    // Legs: each 2x3, side by side under shorts
    // anim_frame 0: both level
    // anim_frame 1: left leg raised (1p up), right leg normal
    // anim_frame 2: right leg raised (1p up), left leg normal
    let leg_w = p * 2.0;
    let leg_h = p * 3.0;
    let left_leg_x  = cx - p * 2.5;
    let right_leg_x = cx + p * 0.5;
    let base_leg_y  = shorts_y + shorts_h;

    let (left_leg_y, right_leg_y) = match anim_frame {
        1 => (base_leg_y - p, base_leg_y),      // left raised
        2 => (base_leg_y, base_leg_y - p),       // right raised
        _ => (base_leg_y, base_leg_y),           // standing
    };

    // Shoes: 2x1 under each leg
    let shoe_h = p;
    let left_shoe_y  = left_leg_y  + leg_h;
    let right_shoe_y = right_leg_y + leg_h;

    // --- Draw ---

    // Arms: 1x2 pixels on each side of torso, animated opposite to legs
    let arm_w = p * 1.0;
    let arm_h = p * 2.0;
    let arm_y = torso_y + p * 0.5;
    let (left_arm_offset, right_arm_offset) = match anim_frame {
        1 => (p * 0.5, -p * 0.5),   // left arm back, right arm forward
        2 => (-p * 0.5, p * 0.5),   // right arm back, left arm forward
        _ => (0.0, 0.0),
    };

    // Ball position depends on direction:
    // facing_back  => ball is "far" from viewer: drawn above legs (lower y = higher on screen),
    //                 significantly to the side, and rendered BEFORE legs so legs overlap it.
    // facing front => ball is "near" viewer: drawn below/beside legs, rendered AFTER everything.
    let ball_r = p * 2.2;
    let ball_pos = if facing_back {
        // Behind the player: above the feet, shifted well to the right so it's visible
        let bx = cx + torso_w / 2.0 + arm_w + p * 2.0;
        let by = base_leg_y - p;
        (bx, by)
    } else {
        // In front of the player: below the feet, slightly right
        let bx = cx + torso_w / 2.0 + arm_w + p;
        let by = base_leg_y + leg_h + shoe_h + p;
        (bx, by)
    };

    // Ball behind player (facing_back) — draw before legs so they overlap it
    if has_ball && facing_back {
        draw_football_ball(ball_pos.0, ball_pos.1, ball_r);
    }

    // Left arm
    draw_rectangle(torso_x - arm_w, arm_y + left_arm_offset, arm_w, arm_h, skin_color);
    // Right arm
    draw_rectangle(torso_x + torso_w, arm_y + right_arm_offset, arm_w, arm_h, skin_color);

    // Left leg
    draw_rectangle(left_leg_x,  left_leg_y,  leg_w, leg_h, sock_color);
    // Right leg
    draw_rectangle(right_leg_x, right_leg_y, leg_w, leg_h, sock_color);
    // Left shoe
    draw_rectangle(left_leg_x,  left_shoe_y,  leg_w, shoe_h, shoe_color);
    // Right shoe
    draw_rectangle(right_leg_x, right_shoe_y, leg_w, shoe_h, shoe_color);

    // Shorts
    draw_rectangle(torso_x, shorts_y, torso_w, shorts_h, shorts_color);

    // Torso (shirt)
    draw_rectangle(torso_x, torso_y, torso_w, torso_h, shirt_color);

    // Head
    draw_rectangle(head_x, head_y, head_w, head_h, skin_color);

    // Hair (top row of head)
    draw_rectangle(head_x, head_y, head_w, p, hair_color);

    // Eyes (front only)
    if !facing_back {
        let eye_y = head_y + p;
        draw_rectangle(head_x + p * 0.5, eye_y, p * 0.5, p * 0.5, Color::new(0.1, 0.1, 0.1, 1.0));
        draw_rectangle(head_x + p * 2.0, eye_y, p * 0.5, p * 0.5, Color::new(0.1, 0.1, 0.1, 1.0));
    }

    // Number — readable size, semi-transparent, shown next to the sprite
    let number_text = number.to_string();
    let font_size = 13.0_f32;
    let number_color = Color::new(1.0, 1.0, 1.0, 0.75);
    draw_text(
        &number_text,
        cx + torso_w / 2.0 + arm_w + 2.0,
        head_y + head_h,
        font_size,
        number_color,
    );

    // Ball in front of player (facing front) — draw after everything so it's on top
    if has_ball && !facing_back {
        draw_football_ball(ball_pos.0, ball_pos.1, ball_r);
    }
}

fn draw_football_ball(cx: f32, cy: f32, r: f32) {
    // White base
    draw_circle(cx, cy, r, WHITE);
    // Black outline
    draw_circle_lines(cx, cy, r, 1.0, BLACK);
    // Simulate pentagon patches with small filled circles
    draw_circle(cx, cy, r * 0.35, BLACK);
    let patch_r = r * 0.22;
    let patch_dist = r * 0.6;
    for i in 0..5 {
        let angle = std::f32::consts::TAU / 5.0 * i as f32 - std::f32::consts::FRAC_PI_2;
        let px = cx + angle.cos() * patch_dist;
        let py = cy + angle.sin() * patch_dist;
        draw_circle(px, py, patch_r, BLACK);
    }
}

fn draw_ball(
    to_screen_x: &dyn Fn(f32) -> f32,
    to_screen_y: &dyn Fn(f32) -> f32,
    game_state: &ynwa_core::game::GameState,
) {
    // Only draw the free ball (possessed ball is drawn at player's feet)
    if game_state.ball_state.possessed_by.is_some() {
        return;
    }

    let ball_state = &game_state.ball_state;
    let bx = ball_state.position.x.get::<meter>();
    let bz = ball_state.position.z.get::<meter>();

    draw_football_ball(to_screen_x(bx), to_screen_y(bz), 4.5);
}
