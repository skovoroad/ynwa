//! Replay of a recorded football match: a reduced world that re-simulates physics only.
//!
//! The replay world deliberately omits Lua, the decision systems and the football game manager;
//! journaled events are fed in by [`ReplayDriver`] instead, so only motion is recomputed.

use crate::events::FootballEvent;
use crate::game_manager::FOOTBALL_EVENT_KIND;
use ynwa_core::field::zones::Velocity3D;
use ynwa_core::game::{Game, GameConfig, GameStage, GameState};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::record::Record;
use ynwa_core::replay::ReplayDriver;
use ynwa_core::rng::{DefaultRngManager, RngConfig};
use ynwa_core::system::System;
use ynwa_core::systems::PhysicsSystem;
use ynwa_core::world::World;

/// Pins the ball the way `FootballGameManager` does at the start of every `Setup` tick.
///
/// Runs before [`ReplayDriver`], so the ball is fixed before the tick's events are applied.
pub struct ReplaySetupBallPlacer;

impl System for ReplaySetupBallPlacer {
    fn update(&mut self, game: &mut Game, _timestamp: f32) {
        if !matches!(game.state.stage, GameStage::Setup(_)) {
            return;
        }
        game.state.ball_state.position = game
            .state
            .restart_position
            .unwrap_or(game.config().ball.initial_position);
        game.state.ball_state.velocity = Velocity3D::default();
        game.state.ball_state.possessed_by = None;
        game.state.ball_state.last_possessing_team = None;
    }
}

/// Builds a world that replays `record` by re-simulating physics only.
///
/// The caller steps the world `record.header.timestep` for `record.footer.total_steps` times,
/// reading those values before the record is consumed.
pub fn create_football_replay_world(record: Record) -> Result<World, String> {
    create_football_replay_world_from_state(
        record.header.config,
        record.header.initial_state,
        record.journal,
        0,
    )
}

/// Builds a replay world resumed from a mid-match `state` snapshot.
///
/// `next_index` is the journal cursor matching `state.elapsed_time`, so entries whose effect is
/// already reflected in `state` are skipped. The physics system is initialized at
/// `state.elapsed_time` so the next step integrates over exactly one timestep.
pub fn create_football_replay_world_from_state(
    config: GameConfig,
    state: GameState,
    journal: Vec<JournalEntry>,
    next_index: usize,
) -> Result<World, String> {
    let last_update = state.elapsed_time;

    // Replay never consults RNG: random-branch outcomes are already in the journal.
    let rng = Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(0))));
    let mut game = Game::new(config, rng);
    game.apply_state(state)?;

    let mut world = World::new(game);
    world.add_system(Box::new(ReplaySetupBallPlacer));
    world.add_system(Box::new(ReplayDriver::from_index(journal, next_index)));
    world.add_system(Box::new(PhysicsSystem::from_timestamp(last_update)));
    Ok(world)
}

/// Decodes the football events a recording carries in its `External` journal entries.
/// Entries with another `kind` and entries whose payload cannot be decoded are skipped.
pub fn decode_football_events(entries: &[JournalEntry]) -> Vec<(f32, FootballEvent)> {
    entries
        .iter()
        .filter_map(|entry| match &entry.event {
            JournalEvent::External { kind, data } if kind == FOOTBALL_EVENT_KIND => {
                serde_json::from_value::<FootballEvent>(data.clone())
                    .ok()
                    .map(|event| (entry.timestamp, event))
            }
            _ => None,
        })
        .collect()
}
