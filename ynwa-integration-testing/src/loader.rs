//! Scenario loading: turn a scenario directory into a ready-to-run [`World`].
//!
//! A directory holds `initial_state.toml`, `scenario.toml`, an optional `final_state.toml` and a
//! `teams/` tree. The loader parses the state, builds the world with a deterministic RNG, resolves
//! every `{team, number}` reference to a global player index and applies the initial snapshot.

use std::fmt;
use std::path::{Path, PathBuf};

use ynwa_core::game::{Game, GameConfig, GameStage};
use ynwa_core::rng::{DefaultRngManager, RngConfig, RngManager};
use ynwa_core::snapshot::{Snapshot, SnapshotBall, SnapshotPlayer, SnapshotSetup};
use ynwa_core::team::Team;
use ynwa_core::world::World;
use ynwa_football::FootballWorldBuilder;
use ynwa_repository::FsTeamRepository;

use crate::dto::{
    BallOwnerDto, BallStateDto, ExpectDto, FinalStateDto, InitialStateDto, RunPlanDto, ScenarioDto,
    StageNameDto, TeamOrNoneDto,
};

/// Fixed RNG state: scenarios must be fully reproducible, so temperature is zero and the seed is
/// hardcoded. Changing either value invalidates every recorded scenario outcome.
const SCENARIO_RNG_TEMPERATURE: f32 = 0.0;
const SCENARIO_RNG_SEED: u64 = 0;

/// Preamble directory relative to this crate, used when no explicit path is supplied.
const DEFAULT_PREAMBLES_PATH: &str = "../ynwa-scripts/preambles";

/// Failure while reading, parsing or building a scenario.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScenarioError {
    message: String,
}

impl ScenarioError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for ScenarioError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ScenarioError {}

impl From<String> for ScenarioError {
    fn from(message: String) -> Self {
        Self::new(message)
    }
}

/// A scenario directory turned into a world with the initial snapshot already applied, together
/// with the parsed run plan and expectations for later stages.
pub struct LoadedScenario {
    pub name: String,
    pub world: World,
    pub run: RunPlanDto,
    pub expect: ExpectDto,
    pub final_state: Option<FinalStateDto>,
}

/// Loads scenario directories with an overridable preamble path.
pub struct ScenarioLoader {
    preambles_path: PathBuf,
}

impl Default for ScenarioLoader {
    fn default() -> Self {
        Self {
            preambles_path: default_preambles_path(),
        }
    }
}

impl ScenarioLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_preambles_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.preambles_path = path.into();
        self
    }

    /// Loads and fully assembles the scenario rooted at `dir`.
    pub fn load(&self, dir: &Path) -> Result<LoadedScenario, ScenarioError> {
        let initial = InitialStateDto::parse(&read_file(&dir.join("initial_state.toml"))?)?;
        let scenario = ScenarioDto::parse(&read_file(&dir.join("scenario.toml"))?)?;
        let final_state = read_optional_file(&dir.join("final_state.toml"))?
            .map(|source| FinalStateDto::parse(&source))
            .transpose()?;

        let mut world = build_world(dir, &initial, &self.preambles_path)?;
        let snapshot = build_snapshot(&initial, world.game())?;
        world.game_mut().apply_snapshot(&snapshot)?;

        Ok(LoadedScenario {
            name: scenario_name(dir),
            world,
            run: scenario.run,
            expect: scenario.expect,
            final_state,
        })
    }
}

/// Loads a scenario with the default preamble path.
pub fn load_scenario(dir: &Path) -> Result<LoadedScenario, ScenarioError> {
    ScenarioLoader::new().load(dir)
}

fn default_preambles_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(DEFAULT_PREAMBLES_PATH)
}

fn scenario_rng() -> Box<dyn RngManager> {
    Box::new(DefaultRngManager::new(RngConfig::new(
        SCENARIO_RNG_TEMPERATURE,
        Some(SCENARIO_RNG_SEED),
    )))
}

fn build_world(
    dir: &Path,
    initial: &InitialStateDto,
    preambles_path: &Path,
) -> Result<World, ScenarioError> {
    let repo = FsTeamRepository::new(dir.join("teams"));
    FootballWorldBuilder::new(&repo, preambles_path)
        .with_rng(scenario_rng())
        .with_stage(resolve_stage(initial))
        .build()
        .map_err(ScenarioError::from)
}

/// Builds the core partial snapshot from the initial state, resolving players to global indices.
fn build_snapshot(initial: &InitialStateDto, game: &Game) -> Result<Snapshot, ScenarioError> {
    let config = game.config();
    let ball = initial
        .ball
        .as_ref()
        .map(|ball| build_ball_snapshot(ball, config))
        .transpose()?;
    let players = initial
        .players
        .iter()
        .map(|placement| {
            Ok(SnapshotPlayer {
                index: resolve_player_index(config, placement.team, placement.number)?,
                position: placement.position,
            })
        })
        .collect::<Result<Vec<_>, ScenarioError>>()?;
    let setup = initial.setup.as_ref().map(|setup| SnapshotSetup {
        restart_position: setup.restart_position,
        restart_team: team_option(setup.restart_team),
    });

    Ok(Snapshot {
        stage: Some(resolve_stage(initial)),
        ball,
        players,
        setup,
        score: None,
    })
}

fn build_ball_snapshot(
    ball: &BallStateDto,
    config: &GameConfig,
) -> Result<SnapshotBall, ScenarioError> {
    let possessed_by = match ball.possessed_by {
        Some(BallOwnerDto::Player(reference)) => Some(resolve_player_index(
            config,
            reference.team,
            reference.number,
        )?),
        Some(BallOwnerDto::Free(_)) | None => None,
    };

    Ok(SnapshotBall {
        position: ball.position,
        velocity: ball.velocity,
        possessed_by,
        last_possessing_team: team_option(ball.last_possessing_team),
    })
}

/// Resolves `{team, number}` to the global player index; a missing or duplicated player is an error.
fn resolve_player_index(
    config: &GameConfig,
    team: Team,
    number: u32,
) -> Result<usize, ScenarioError> {
    let mut matches = config
        .players
        .iter()
        .enumerate()
        .filter(|(_, def)| def.team == team && def.number == number)
        .map(|(index, _)| index);

    let index = matches.next().ok_or_else(|| {
        ScenarioError::new(format!(
            "player {team:?} #{number} is not declared in the teams"
        ))
    })?;
    if matches.next().is_some() {
        return Err(ScenarioError::new(format!(
            "player {team:?} #{number} is declared more than once"
        )));
    }
    Ok(index)
}

fn resolve_stage(initial: &InitialStateDto) -> GameStage {
    match initial.stage {
        Some(StageNameDto::Play) => GameStage::Play,
        Some(StageNameDto::GameOver) => GameStage::GameOver,
        // An omitted stage also means Setup, so an explicit `setup_reason` is honoured in both cases.
        None | Some(StageNameDto::Setup) => GameStage::Setup(
            initial
                .setup_reason
                .clone()
                .unwrap_or_else(|| "kick off".to_string()),
        ),
    }
}

fn team_option(value: Option<TeamOrNoneDto>) -> Option<Team> {
    match value {
        Some(TeamOrNoneDto::Team(team)) => Some(team),
        Some(TeamOrNoneDto::None(_)) | None => None,
    }
}

fn read_file(path: &Path) -> Result<String, ScenarioError> {
    std::fs::read_to_string(path)
        .map_err(|error| ScenarioError::new(format!("cannot read '{}': {error}", path.display())))
}

fn read_optional_file(path: &Path) -> Result<Option<String>, ScenarioError> {
    match std::fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(ScenarioError::new(format!(
            "cannot read '{}': {error}",
            path.display()
        ))),
    }
}

fn scenario_name(dir: &Path) -> String {
    dir.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_string()
}

#[cfg(test)]
#[path = "tests/loader_tests.rs"]
mod tests;
