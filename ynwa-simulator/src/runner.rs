use std::path::PathBuf;

use ynwa_core::rng::{DefaultRngManager, RngConfig, RngManager};
use ynwa_core::world::World;
use ynwa_core::{json_journal_file_writer, FileJournalRecorder, RecordHeader};
use ynwa_football::FootballWorldBuilder;
use ynwa_repository::FsTeamRepository;

use crate::cli::{recording_path, Cli};

/// Plays one match in model time and writes its recording, returning the recording path.
pub fn run(cli: &Cli) -> Result<PathBuf, String> {
    let mut world = build_world(cli)?;
    let path = recording_path(cli.out.as_deref(), &cli.team_a_id, &cli.team_b_id)?;
    println!("Recording to {}", path.display());

    let writer = json_journal_file_writer(&path)?;
    let header = RecordHeader::from_game(world.game(), cli.timestep);
    world
        .game_mut()
        .set_journal_sink(Box::new(FileJournalRecorder::new(
            Box::new(writer),
            &header,
        )));

    advance_to(&mut world, cli.timestep, cli.duration)?;
    report_script_errors(&world);
    println!("Recording saved to {}", path.display());
    Ok(path)
}

/// Steps until at least `duration` seconds of model time have elapsed, then finalizes the journal.
fn advance_to(world: &mut World, timestep: f32, duration: f32) -> Result<(), String> {
    while world.game().state().elapsed_time < duration {
        world.step(timestep);
    }
    world.game_mut().finish_journal()
}

/// A strict build: unlike the playable game, a broken team script aborts the run instead of
/// silently falling back to placeholder decisions.
fn build_world(cli: &Cli) -> Result<World, String> {
    let repo = FsTeamRepository::new(&cli.teams_path);
    FootballWorldBuilder::new(&repo, &cli.preambles_path)
        .with_teams(&cli.team_a_id, &cli.team_b_id)
        .with_rng(player_rng(cli))
        .build()
}

fn player_rng(cli: &Cli) -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(
        cli.temperature,
        cli.seed,
    )))
}

/// Runtime decision errors do not abort a batch run, but they must not go unnoticed either.
fn report_script_errors(world: &World) {
    let game = world.game();
    for (player, state) in game
        .config()
        .players
        .iter()
        .zip(&game.state().player_states)
    {
        if let Some(error) = &state.last_error {
            eprintln!(
                "Warning: player {:?} #{}: {}",
                player.team, player.number, error
            );
        }
    }
}

#[cfg(test)]
#[path = "runner_tests.rs"]
mod tests;
