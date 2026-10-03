//! Partial snapshot of `GameState` used by integration scenarios.
//!
//! Every field is optional: a missing field means "do not touch" when applied to a `Game`
//! (initial state) and "do not verify" when compared against the final state. Players are
//! addressed by their global index, matching the layout of `GameState::player_states`.

use crate::field::zones::{Point3D, Velocity3D};
use crate::game::GameStage;
use crate::team::Team;
use serde::{Deserialize, Serialize};

/// Team score, stored under the `"score"` stat key of each team.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Score {
    #[serde(rename = "A")]
    pub a: u32,
    #[serde(rename = "B")]
    pub b: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotBall {
    pub position: Option<Point3D>,
    pub velocity: Option<Velocity3D>,
    /// Global player index, or `None` to leave possession unchanged.
    pub possessed_by: Option<usize>,
    pub last_possessing_team: Option<Team>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SnapshotPlayer {
    /// Global index into `GameState::player_states`.
    pub index: usize,
    pub position: Point3D,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SnapshotSetup {
    pub restart_position: Option<Point3D>,
    pub restart_team: Option<Team>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub stage: Option<GameStage>,
    pub ball: Option<SnapshotBall>,
    #[serde(default)]
    pub players: Vec<SnapshotPlayer>,
    pub setup: Option<SnapshotSetup>,
    pub score: Option<Score>,
}
