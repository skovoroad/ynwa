//! Step-by-step (lockstep) equivalence between a recorded run and its replay.
//!
//! The stage-4 integration tests compare only the final state, so an event applied one tick late or
//! a divergence that self-corrects by the end would go unnoticed. Here both runs advance one step
//! at a time and their contract state is compared on every step.

use crate::events::{check_events, FootballEvent};
use crate::field_builder::create_football_field;
use crate::replay::{create_football_replay_world, decode_football_events};
use crate::test_utils::{assert_equivalent, build_recording_world_with, FIXED_DT};
use std::collections::{HashMap, HashSet};
use uom::si::length::meter;
use ynwa_core::field::zones::Point3D;
use ynwa_core::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, GameState, PlayerDef,
    RefereeDef, ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::record::RecordHeader;
use ynwa_core::region::GridCell;
use ynwa_core::systems::{DecisionError, DecisionMaker};
use ynwa_core::team::Team;
use ynwa_core::world::World;

const LOCKSTEP_STEPS: u64 = 150;

/// One striker on the ball near the centre. Every set-piece key is both a region and a role, so the
/// lone player always takes the restart and the game never stalls in `Setup`.
fn lockstep_config() -> GameConfig {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(
            GridCell::new(13, 20).unwrap(),
            GridCell::new(13, 20).unwrap(),
        )
        .unwrap();
    let ball_position = start_region.center(grid_dims, field.width().get::<meter>());

    let mut regions = HashMap::new();
    regions.insert(REGION_START_POSITION.to_string(), start_region.clone());
    let mut set_piece_roles = HashSet::new();
    for key in crate::SET_PIECE_KEYS {
        regions.insert((*key).to_string(), start_region.clone());
        set_piece_roles.insert((*key).to_string());
    }

    let player = PlayerDef::new(Team::A, 1, "Striker".to_string(), String::new(), regions)
        .with_reaction_rate(100)
        .with_speed_rate(100)
        .with_shot_power(100)
        .with_shot_accuracy(100)
        .with_set_piece_roles(set_piece_roles);

    GameConfig {
        field,
        players: vec![player],
        ball: BallDef {
            initial_position: ball_position,
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

/// Kicks the first possession at the opponent goal and every later possession out of play, so a
/// single run yields both a `Goal` and a `Touchline`.
struct GoalThenTouchline {
    kicks: u32,
}

impl DecisionMaker for GoalThenTouchline {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        if game.state.ball_state.possessed_by != Some(player_index) {
            return Ok((
                Decision::Run(DecisionTarget::Ball),
                Some("chase".to_string()),
            ));
        }

        self.kicks += 1;
        let field = &game.config().field;
        let target = if self.kicks == 1 {
            Point3D::from_meters(
                field.width().get::<meter>() / 2.0,
                0.0,
                field.length().get::<meter>(),
            )
        } else {
            Point3D::from_meters(
                field.width().get::<meter>() + 30.0,
                0.0,
                field.length().get::<meter>() / 2.0,
            )
        };
        Ok((Decision::Kick(target), Some(format!("kick {}", self.kicks))))
    }
}

/// Snapshots the contract state before each step, and reports `check_events` only on `Play`
/// snapshots — mirroring `FootballGameManager`, which checks events in the `Play` branch alone.
fn run_lockstep(world: &mut World, steps: u64) -> (Vec<GameState>, Vec<(f32, FootballEvent)>) {
    let mut before = Vec::new();
    let mut detected = Vec::new();
    for _ in 0..steps {
        let state = world.game().state().clone();
        if matches!(state.stage, GameStage::Play) {
            if let Some(event) = check_events(world.game()) {
                detected.push((state.elapsed_time + FIXED_DT, event));
            }
        }
        before.push(state);
        world.step(FIXED_DT);
    }
    (before, detected)
}

#[test]
fn lockstep_matches_on_every_step() {
    let config = lockstep_config();
    let (mut world, collection) = build_recording_world_with(
        config.clone(),
        GameStage::Play,
        Box::new(GoalThenTouchline { kicks: 0 }),
    );

    let (original_before, original_events) = run_lockstep(&mut world, LOCKSTEP_STEPS);
    world.game_mut().finish_journal().unwrap();
    let original_final = world.game().state().clone();

    let header = RecordHeader {
        config,
        initial_stage: GameStage::Play,
        fixed_dt: FIXED_DT,
    };
    let record = collection.borrow_mut().take_record(header);
    assert_eq!(
        record.total_steps, LOCKSTEP_STEPS,
        "sink must count the recorded steps"
    );
    let total_steps = record.total_steps;
    let journal_events = decode_football_events(&record.journal);

    let mut replay = create_football_replay_world(record).unwrap();
    let (replay_before, replay_events) = run_lockstep(&mut replay, total_steps);
    let replay_final = replay.game().state().clone();

    for (original, replayed) in original_before.iter().zip(&replay_before) {
        assert_equivalent(original, replayed);
    }
    assert_equivalent(&original_final, &replay_final);

    assert_eq!(
        original_events, replay_events,
        "check_events must agree on every Play snapshot"
    );
    assert_eq!(
        original_events, journal_events,
        "detected events must match the decoded journal"
    );

    assert!(
        journal_events
            .iter()
            .any(|(_, event)| matches!(event, FootballEvent::Goal(_))),
        "scenario must score a goal: {journal_events:?}"
    );
    assert!(
        journal_events
            .iter()
            .any(|(_, event)| matches!(event, FootballEvent::Touchline(..))),
        "scenario must put the ball out of play: {journal_events:?}"
    );
}
