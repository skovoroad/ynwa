//! Tests for [`FootballWorldBuilder`](crate::FootballWorldBuilder): strict build errors and the
//! placeholder fallback used by the playable entry point.

use crate::test_utils::deterministic_rng;
use crate::{create_football_world, FootballWorldBuilder};
use ynwa_core::game::GameStage;
use ynwa_core::repository::{TeamRecord, TeamRepository};
use ynwa_repository::FsTeamRepository;

/// Delegates to a real repository, replacing every team preamble with invalid Lua.
struct BrokenPreambleRepository<'a> {
    inner: &'a FsTeamRepository,
}

impl TeamRepository for BrokenPreambleRepository<'_> {
    fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> {
        let mut record = self.inner.load_team(team_id)?;
        record.preamble = "this is not valid lua !!!".to_string();
        Ok(record)
    }
}

fn real_repository() -> Option<FsTeamRepository> {
    let teams_path = std::path::Path::new("../teams");
    teams_path
        .exists()
        .then(|| FsTeamRepository::new(teams_path))
}

fn preambles_path() -> &'static std::path::Path {
    std::path::Path::new("../ynwa-scripts/preambles")
}

#[test]
fn strict_build_fails_on_broken_preamble() {
    let Some(repo) = real_repository() else {
        return;
    };
    let broken = BrokenPreambleRepository { inner: &repo };

    let error = FootballWorldBuilder::new(&broken, preambles_path())
        .with_rng(deterministic_rng())
        .build()
        .map(|_| ())
        .unwrap_err();

    assert!(
        error.contains("ScriptedDecisionMaker"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn placeholder_fallback_builds_on_broken_preamble() {
    let Some(repo) = real_repository() else {
        return;
    };
    let broken = BrokenPreambleRepository { inner: &repo };

    let world = FootballWorldBuilder::new(&broken, preambles_path())
        .with_rng(deterministic_rng())
        .with_placeholder_fallback()
        .build()
        .expect("fallback build must succeed with a placeholder decision system");

    assert_eq!(world.game().config().players.len(), 22);
}

#[test]
fn strict_build_fails_on_missing_preamble() {
    let Some(repo) = real_repository() else {
        return;
    };
    let missing = std::path::Path::new("../ynwa-scripts/absent");

    let error = FootballWorldBuilder::new(&repo, missing)
        .with_rng(deterministic_rng())
        .build()
        .map(|_| ())
        .unwrap_err();

    assert!(error.contains("core.lua"), "unexpected error: {}", error);
}

#[test]
fn placeholder_fallback_does_not_hide_missing_preamble() {
    let Some(repo) = real_repository() else {
        return;
    };
    let missing = std::path::Path::new("../ynwa-scripts/absent");

    let error = FootballWorldBuilder::new(&repo, missing)
        .with_rng(deterministic_rng())
        .with_placeholder_fallback()
        .build()
        .map(|_| ())
        .unwrap_err();

    assert!(error.contains("core.lua"), "unexpected error: {}", error);
}

#[test]
fn build_applies_requested_stage() {
    let Some(repo) = real_repository() else {
        return;
    };

    let world = FootballWorldBuilder::new(&repo, preambles_path())
        .with_rng(deterministic_rng())
        .with_stage(GameStage::Play)
        .build()
        .expect("valid repository must build");

    assert_eq!(world.game().config().players.len(), 22);
    assert_eq!(world.game().state().stage, GameStage::Play);
}

#[test]
fn default_wrapper_builds_playable_world() {
    let Some(repo) = real_repository() else {
        return;
    };

    let world = create_football_world(&repo, preambles_path())
        .expect("valid repository must build through the default wrapper");

    assert_eq!(world.game().config().players.len(), 22);
}
