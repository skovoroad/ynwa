mod cli;
mod input;
mod renderer;
mod simulation;
mod ui;

use macroquad::prelude::*;
use std::path::PathBuf;
use uom::si::length::meter;
use ynwa_core::{
    json_journal_file_reader, json_journal_file_writer, FileJournalRecorder, GameStage, Record,
    RecordHeader, RecordReader, World,
};
use ynwa_football::events::FootballEvent;
use ynwa_football::{create_football_replay_world, create_football_world, decode_football_events};
use ynwa_repository::FsTeamRepository;

use cli::{games_dir, next_record_path, parse, Cli, Mode};
use input::handle_input;
use renderer::render_field;
use simulation::SimulationControl;
use ui::{draw_control_panel, draw_separator};

/// Real-time tempo of play and recording (simulation steps per second).
const PLAYBACK_RATE: f32 = 60.0;

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
    let args: Vec<String> = std::env::args().skip(1).collect();
    let cli = match parse(&args) {
        Ok(cli) => cli,
        Err(message) => exit_with_error(&message),
    };

    let mut is_fullscreen = false;
    set_fullscreen(is_fullscreen);

    match cli.mode {
        Mode::Play => run_play(&cli, &mut is_fullscreen).await,
        Mode::Record => run_record(&cli, &mut is_fullscreen).await,
        Mode::Replay(path) => run_replay(path, &mut is_fullscreen).await,
    }
}

async fn run_play(cli: &Cli, is_fullscreen: &mut bool) {
    let repo = FsTeamRepository::new(&cli.teams_path);
    let mut world = load_or_exit(
        create_football_world(&repo, &cli.preambles_path),
        "load game",
    );
    print_loaded(&world);

    let field_ratio = field_width_ratio(&world);
    let mut simulation = SimulationControl::new(PLAYBACK_RATE);

    loop {
        if handle_input(&mut simulation, is_fullscreen, false) {
            break;
        }

        advance(&mut world, &mut simulation);
        render_scene(&world, &simulation, field_ratio, None);

        next_frame().await
    }
}

async fn run_record(cli: &Cli, is_fullscreen: &mut bool) {
    let repo = FsTeamRepository::new(&cli.teams_path);
    let mut world = load_or_exit(
        create_football_world(&repo, &cli.preambles_path),
        "load game",
    );
    print_loaded(&world);

    let field_ratio = field_width_ratio(&world);
    let mut simulation = SimulationControl::new(PLAYBACK_RATE);
    let fixed_dt = simulation.step_delta();

    let dir = load_or_exit(
        games_dir(cli.out_dir.as_deref()),
        "prepare recordings directory",
    );
    let path = load_or_exit(next_record_path(&dir), "choose recording path");
    let writer = load_or_exit(json_journal_file_writer(&path), "create recording file");
    let header = RecordHeader::from_game(world.game(), fixed_dt);
    world
        .game_mut()
        .set_journal_sink(Box::new(FileJournalRecorder::new(
            Box::new(writer),
            &header,
        )));
    println!("Recording to {}", path.display());

    loop {
        if handle_input(&mut simulation, is_fullscreen, true) {
            // Closed early: leave the file a valid JSON Lines prefix.
            let _ = world.game_mut().finish_journal();
            break;
        }

        advance(&mut world, &mut simulation);

        if world.game().state().stage == GameStage::GameOver {
            load_or_exit(world.game_mut().finish_journal(), "finalize recording");
            println!("Recording saved to {}", path.display());
            break;
        }

        render_scene(&world, &simulation, field_ratio, None);

        next_frame().await
    }
}

async fn run_replay(path: PathBuf, is_fullscreen: &mut bool) {
    let mut reader = load_or_exit(json_journal_file_reader(&path), "open recording");
    let record = load_or_exit(reader.read(), "read recording");

    let fixed_dt = record.header.fixed_dt;
    let events = decode_football_events(&record.journal);
    let total_steps = replay_steps(&record, fixed_dt);
    if total_steps == 0 {
        eprintln!("Warning: recording contains no replayable steps");
    } else if record.total_steps == 0 {
        eprintln!(
            "Warning: recording has no footer (interrupted?); replaying {} steps up to the last event",
            total_steps
        );
    }
    let mut world = load_or_exit(create_football_replay_world(record), "create replay world");

    println!("Replaying {} ({} steps)", path.display(), total_steps);

    let field_ratio = field_width_ratio(&world);
    let mut simulation = SimulationControl::for_replay(fixed_dt, total_steps);
    let mut visible_events: Vec<(f32, FootballEvent)> = Vec::new();
    let mut event_cursor = 0usize;

    loop {
        if handle_input(&mut simulation, is_fullscreen, true) {
            break;
        }

        advance(&mut world, &mut simulation);

        let elapsed = world.game().state().elapsed_time;
        while event_cursor < events.len() && events[event_cursor].0 <= elapsed {
            visible_events.push(events[event_cursor].clone());
            event_cursor += 1;
        }

        render_scene(&world, &simulation, field_ratio, Some(&visible_events));

        next_frame().await
    }
}

/// Number of steps to replay.
///
/// A normally finished recording carries the count in its footer. A recording interrupted while
/// being written has no footer, so it is replayed up to its last recorded event instead.
fn replay_steps(recording: &Record, fixed_dt: f32) -> u64 {
    if recording.total_steps > 0 {
        return recording.total_steps;
    }
    if fixed_dt <= 0.0 {
        return 0;
    }
    let last_timestamp = recording
        .journal
        .last()
        .map_or(0.0, |entry| entry.timestamp);
    (last_timestamp / fixed_dt).ceil() as u64
}

/// Accumulates frame time and advances the world by all due simulation steps.
fn advance(world: &mut World, simulation: &mut SimulationControl) {
    simulation.accumulate(get_frame_time());

    while simulation.should_step() {
        world.step(simulation.step_delta());
        simulation.consume_step();
    }
}

fn print_loaded(world: &World) {
    println!(
        "Loaded game with {} players",
        world.game().config().players.len()
    );
}

fn field_width_ratio(world: &World) -> f32 {
    let field = &world.game().config().field;
    field.width().get::<meter>() / field.length().get::<meter>()
}

fn render_scene(
    world: &World,
    simulation: &SimulationControl,
    field_width_ratio: f32,
    events: Option<&[(f32, FootballEvent)]>,
) {
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

    render_field(
        world.game().config(),
        world.game().state(),
        field_area_width,
        screen_h,
    );

    draw_separator(field_area_width, screen_h);

    if control_panel_width > 50.0 {
        draw_control_panel(
            control_panel_x,
            world.game().config(),
            world.game().state(),
            simulation.paused,
            events,
        );
    }
}

fn load_or_exit<T>(result: Result<T, String>, action: &str) -> T {
    match result {
        Ok(value) => value,
        Err(error) => {
            eprintln!("Error: failed to {}: {}", action, error);
            std::process::exit(1);
        }
    }
}

fn exit_with_error(message: &str) -> ! {
    eprintln!("Error: {}", message);
    std::process::exit(1);
}
