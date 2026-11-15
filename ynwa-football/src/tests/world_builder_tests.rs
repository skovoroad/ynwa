//! Tests for [`FootballWorldBuilder`](crate::FootballWorldBuilder): strict build errors and the
//! placeholder fallback used by the playable entry point.

use crate::test_utils::deterministic_rng;
use crate::FootballWorldBuilder;
use std::cell::RefCell;
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

/// Serves the real `team_a` data for any requested id, deriving a display name from that id and
/// recording every request so tests can assert which identifiers were loaded.
struct RecordingRepository<'a> {
    inner: &'a FsTeamRepository,
    requested: RefCell<Vec<String>>,
    strip_set_piece_positions: bool,
}

impl TeamRepository for RecordingRepository<'_> {
    fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> {
        self.requested.borrow_mut().push(team_id.to_string());
        let mut record = self.inner.load_team("team_a")?;
        record.display_name = format!("Display {}", team_id);
        if self.strip_set_piece_positions {
            for player in &mut record.players {
                player.tactical.set_piece_positions.clear();
            }
        }
        Ok(record)
    }
}

fn recording_repository(
    inner: &FsTeamRepository,
    strip_set_piece_positions: bool,
) -> RecordingRepository<'_> {
    RecordingRepository {
        inner,
        requested: RefCell::new(Vec::new()),
        strip_set_piece_positions,
    }
}

/// Fixture repository committed with the project. Fails loudly when absent so the tests can never
/// silently pass without exercising the builder.
fn real_repository() -> FsTeamRepository {
    let teams_path = std::path::Path::new("../teams");
    assert!(
        teams_path.exists(),
        "fixture repository not found at {}",
        teams_path.display()
    );
    FsTeamRepository::new(teams_path)
}

fn preambles_path() -> &'static std::path::Path {
    std::path::Path::new("../ynwa-scripts/preambles")
}

#[test]
fn strict_build_fails_on_broken_preamble() {
    let repo = real_repository();
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
    let repo = real_repository();
    let broken = BrokenPreambleRepository { inner: &repo };

    // The same broken repository must fail the strict build, proving that the successful build
    // below goes through the placeholder fallback branch rather than the scripted engine.
    assert!(
        FootballWorldBuilder::new(&broken, preambles_path())
            .with_rng(deterministic_rng())
            .build()
            .is_err(),
        "strict build must fail on a broken preamble"
    );

    let world = FootballWorldBuilder::new(&broken, preambles_path())
        .with_rng(deterministic_rng())
        .with_placeholder_fallback()
        .build()
        .expect("fallback build must succeed with a placeholder decision system");

    assert_eq!(world.game().config().players.len(), 22);
}

#[test]
fn strict_build_fails_on_missing_preamble() {
    let repo = real_repository();
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
    let repo = real_repository();
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
fn custom_team_ids_are_loaded_and_stored_in_config() {
    let repo = real_repository();
    let recording = recording_repository(&repo, false);

    let world = FootballWorldBuilder::new(&recording, preambles_path())
        .with_rng(deterministic_rng())
        .with_teams("pressing", "counter")
        .build()
        .expect("custom team ids must build");

    assert_eq!(
        *recording.requested.borrow(),
        vec!["pressing".to_string(), "counter".to_string()]
    );
    let config = world.game().config();
    assert_eq!(config.team_a_id, "pressing");
    assert_eq!(config.team_b_id, "counter");
    assert_eq!(config.team_a_display_name, "Display pressing");
    assert_eq!(config.team_b_display_name, "Display counter");
    assert_eq!(config.players.len(), 22);
}

#[test]
fn a_team_can_play_against_itself() {
    let repo = real_repository();
    let recording = recording_repository(&repo, false);

    let world = FootballWorldBuilder::new(&recording, preambles_path())
        .with_rng(deterministic_rng())
        .with_teams("pressing", "pressing")
        .build()
        .expect("a team must be able to play against itself");

    assert_eq!(
        *recording.requested.borrow(),
        vec!["pressing".to_string(), "pressing".to_string()]
    );
    let config = world.game().config();
    assert_eq!(config.team_a_id, "pressing");
    assert_eq!(config.team_b_id, "pressing");
    assert_eq!(config.team_a_display_name, "Display pressing");
    assert_eq!(config.team_b_display_name, "Display pressing");
    assert_eq!(config.players.len(), 22);
}

#[test]
fn validation_errors_reference_custom_team_ids() {
    let repo = real_repository();
    let broken = recording_repository(&repo, true);

    let error = FootballWorldBuilder::new(&broken, preambles_path())
        .with_rng(deterministic_rng())
        .with_teams("broken_a", "broken_b")
        .build()
        .map(|_| ())
        .unwrap_err();

    assert!(
        error.contains("team broken_a"),
        "unexpected error: {}",
        error
    );
}

#[test]
fn build_applies_requested_stage() {
    let repo = real_repository();

    let world = FootballWorldBuilder::new(&repo, preambles_path())
        .with_rng(deterministic_rng())
        .with_stage(GameStage::Play)
        .build()
        .expect("valid repository must build");

    assert_eq!(world.game().config().players.len(), 22);
    assert_eq!(world.game().state().stage, GameStage::Play);
}
