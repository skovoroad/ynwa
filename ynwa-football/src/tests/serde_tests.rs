//! Round-trip tests for the real football configuration.
//!
//! The regulation field contains `Arc` zones whose angles come from `atan2`, so it is the sharpest
//! check of the serialization contract: those angles only survive a round trip because the record
//! keeps base SI units instead of converting them to degrees.

use crate::field_builder::create_football_field;
use std::collections::{HashMap, HashSet};
use ynwa_core::field::Field;
use ynwa_core::game::{
    BallDef, GameConfig, PlayerDef, RefereeDef, ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::region::GridCell;
use ynwa_core::team::Team;

fn football_game_config() -> GameConfig {
    let field = create_football_field();
    let ball_position = crate::get_ball_initial_position(&field);
    let start_region = field
        .grid_dimensions()
        .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
        .unwrap();
    let player = PlayerDef::new(
        Team::A,
        1,
        "Player A1".to_string(),
        "function make_decision() return { action = 'stop' } end".to_string(),
        HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
    )
    .with_set_piece_roles(HashSet::from([
        "corner own".to_string(),
        "goal kick own".to_string(),
    ]));

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

#[test]
fn football_field_round_trip_is_exact_and_byte_stable() {
    let field = create_football_field();

    let json = serde_json::to_string(&field).expect("serialize football field");
    let restored: Field = serde_json::from_str(&json).expect("deserialize football field");

    assert_eq!(restored, field);
    assert_eq!(
        serde_json::to_string(&restored).expect("re-serialize football field"),
        json
    );
}

#[test]
fn football_game_config_round_trip_is_exact_and_byte_stable() {
    let config = football_game_config();

    let json = serde_json::to_string(&config).expect("serialize football config");
    let restored: GameConfig = serde_json::from_str(&json).expect("deserialize football config");

    assert_eq!(restored, config);
    assert_eq!(
        serde_json::to_string(&restored).expect("re-serialize football config"),
        json
    );
}
