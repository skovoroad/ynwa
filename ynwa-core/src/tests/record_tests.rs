//! Tests for the recording model's header construction.

use crate::field::Field;
use crate::game::{
    BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef, ScriptingConfig,
    REGION_START_POSITION,
};
use crate::record::RecordHeader;
use crate::region::GridCell;
use crate::rng::{DefaultRngManager, RngConfig};
use crate::team::Team;
use std::collections::HashMap;

fn test_config() -> GameConfig {
    let field = Field::from_meters(100.0, 60.0, 26, 44);
    let grid_dims = field.grid_dimensions();
    let start_region = grid_dims
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();

    GameConfig {
        field,
        players: vec![PlayerDef::new(
            Team::A,
            1,
            "Test Player".to_string(),
            String::new(),
            HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
        )],
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

fn test_game(stage: GameStage) -> Game {
    Game::with_stage(
        test_config(),
        stage,
        Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(1)))),
    )
}

#[test]
fn from_game_captures_config_stage_and_step() {
    let stage = GameStage::Setup("kick off".to_string());
    let game = test_game(stage.clone());

    let header = RecordHeader::from_game(&game, 0.05);

    assert_eq!(header.config, test_config());
    assert_eq!(header.initial_stage, stage);
    assert_eq!(header.fixed_dt, 0.05);
}

#[test]
fn from_game_reflects_the_current_play_stage() {
    let game = test_game(GameStage::Play);

    let header = RecordHeader::from_game(&game, 0.1);

    assert_eq!(header.initial_stage, GameStage::Play);
}
