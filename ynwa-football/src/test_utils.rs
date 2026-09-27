//! Helpers shared by unit tests.

use crate::game_manager::FootballGameManager;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use ynwa_core::field::zones::Point3D;
use ynwa_core::field::Field;
use ynwa_core::game::{
    BallDef, Decision, DecisionTarget, Game, GameConfig, GameStage, PlayerDef, RefereeDef,
    ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::journal::{CollectJournalRecorder, EventsCollection};
use ynwa_core::region::GridCell;
use ynwa_core::rng::{DefaultRngManager, RngConfig, RngManager};
use ynwa_core::systems::{
    ActionSystem, BallPossessionSystem, DecisionError, DecisionMaker, DecisionSystem,
    PhysicsSystem, PlayerReactionSystem,
};
use ynwa_core::team::Team;
use ynwa_core::world::World;

/// Fixed step and length of the short game recorded by the journal and replay tests.
pub(crate) const FIXED_DT: f32 = 0.1;
pub(crate) const STEPS: u64 = 150;

/// Deterministic manager: zero variation, fixed seed.
pub(crate) fn deterministic_rng() -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(42))))
}

/// Attaches an accumulating journal recorder to the game and returns its shared collection.
pub(crate) fn attach_journal(game: &mut Game) -> Rc<RefCell<EventsCollection>> {
    let collection = Rc::new(RefCell::new(EventsCollection::default()));
    game.set_journal_sink(Box::new(CollectJournalRecorder::new(Rc::clone(
        &collection,
    ))));
    collection
}

/// One striker near the centre spot: chases the ball and kicks it at the far sideline.
pub(crate) fn single_player_config() -> GameConfig {
    let field = Field::from_meters(100.0, 60.0, 26, 16);
    let grid_dims = field.grid_dimensions();
    let centre_region = grid_dims
        .create_region(GridCell::new(12, 8).unwrap(), GridCell::new(12, 8).unwrap())
        .unwrap();

    let player = PlayerDef::new(
        Team::A,
        1,
        "Runner".to_string(),
        String::new(),
        HashMap::from([
            (REGION_START_POSITION.to_string(), centre_region.clone()),
            ("kick off opp".to_string(), centre_region),
        ]),
    )
    .with_reaction_rate(100)
    .with_speed_rate(100)
    .with_shot_power(100)
    .with_shot_accuracy(100);

    GameConfig {
        field,
        players: vec![player],
        ball: BallDef {
            initial_position: Point3D::from_meters(50.0, 0.0, 30.0),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

/// Drives the player to the ball and kicks it over the sideline.
pub(crate) struct ChaseAndKick;

impl DecisionMaker for ChaseAndKick {
    fn make_decision(
        &mut self,
        game: &Game,
        player_index: usize,
    ) -> Result<(Decision, Option<String>), DecisionError> {
        let decision = if game.state.ball_state.possessed_by == Some(player_index) {
            Decision::Kick(Point3D::from_meters(0.0, 0.0, 30.0))
        } else {
            Decision::Run(DecisionTarget::Ball)
        };
        Ok((decision, Some("test".to_string())))
    }
}

/// Full football pipeline with an attached accumulating journal; used to record a playthrough.
pub(crate) fn build_recording_world_with(
    config: GameConfig,
    stage: GameStage,
    decision_maker: Box<dyn DecisionMaker>,
) -> (World, Rc<RefCell<EventsCollection>>) {
    let game = Game::with_stage(config, stage, deterministic_rng());
    let mut world = World::new(game);
    world.add_system(Box::new(FootballGameManager::new()));
    world.add_system(Box::new(PlayerReactionSystem));
    world.add_system(Box::new(BallPossessionSystem::new()));
    world.add_system(Box::new(
        DecisionSystem::new().with_decision_maker(decision_maker),
    ));
    world.add_system(Box::new(ActionSystem::new()));
    world.add_system(Box::new(PhysicsSystem::new()));

    let collection = attach_journal(world.game_mut());
    (world, collection)
}

/// The short single-player game recorded by the journal and replay tests.
pub(crate) fn build_recording_world() -> (World, Rc<RefCell<EventsCollection>>) {
    build_recording_world_with(
        single_player_config(),
        GameStage::Setup("kick off".to_string()),
        Box::new(ChaseAndKick),
    )
}
