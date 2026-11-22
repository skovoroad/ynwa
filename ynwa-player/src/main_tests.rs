use super::{setup_speed, SETUP_SPEEDUP};
use ynwa_core::GameStage;

#[test]
fn setup_speed_depends_on_stage() {
    let cases = [
        (GameStage::Play, 1.0),
        (GameStage::Setup("kick off".to_string()), SETUP_SPEEDUP),
        (GameStage::Setup("corner".to_string()), SETUP_SPEEDUP),
    ];

    for (stage, expected) in cases {
        assert_eq!(setup_speed(&stage), expected, "stage {stage:?}");
    }
}
