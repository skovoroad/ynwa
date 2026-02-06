mod input;
mod renderer;
mod simulation;
mod ui;

use macroquad::prelude::*;
use std::env;
use std::path::Path;
use uom::si::length::meter;
use ynwa_core::create_football_world_from_file;

use input::handle_input;
use renderer::render_field;
use simulation::SimulationControl;
use ui::{draw_control_panel, draw_separator};

fn window_conf() -> Conf {
    Conf {
        window_title: "YNWA - Football Manager".to_owned(),
        fullscreen: true,
        window_resizable: true,
        high_dpi: true,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Получаем путь к конфигу из аргументов
    let args: Vec<String> = env::args().collect();
    let config_path = if args.len() > 1 {
        Path::new(&args[1])
    } else {
        Path::new("config/default_game.toml")
    };

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

    let mut simulation = SimulationControl::new(60.0);

    loop {
        if handle_input(&mut simulation, &mut is_fullscreen) {
            break;
        }

        simulation.accumulate(get_frame_time());

        while simulation.should_step() {
            world.step(simulation.delta());
            simulation.consume_step();
        }

        render_scene(&world, &simulation, field_width_ratio);

        next_frame().await
    }
}

fn render_scene(world: &ynwa_core::World, simulation: &SimulationControl, field_width_ratio: f32) {
    let screen_w = screen_width();
    let screen_h = screen_height();

    let margin = 20.0;
    let available_height = screen_h - 2.0 * margin;
    let available_width = available_height * field_width_ratio;
    let field_area_width = available_width + 2.0 * margin;

    let control_panel_x = field_area_width;
    let control_panel_width = screen_w - field_area_width;

    clear_background(Color::new(0.3, 0.3, 0.3, 1.0));

    draw_rectangle(
        0.0,
        0.0,
        field_area_width,
        screen_h,
        Color::new(0.13, 0.55, 0.13, 1.0),
    );

    render_field(world.game().config(), world.game().state(), field_area_width, screen_h);

    draw_separator(field_area_width, screen_h);

    if control_panel_width > 50.0 {
        draw_control_panel(
            control_panel_x,
            world.game().config(),
            world.game().state(),
            simulation.paused,
        );
    }
}
