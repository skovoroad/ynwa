//! Input DTOs of the scenario model and their TOML parsing.
//!
//! These types mirror the scenario files (`initial_state.toml`, `final_state.toml`,
//! `scenario.toml`); they are the on-disk representation only, hence the `Dto` suffix. Runtime
//! counterparts (e.g. [`StopCriterion`](crate::criterion::StopCriterion)) are built from them by
//! the loader and the runner. Every field of a partial state is optional: a missing field means
//! "do not set" for the initial state and "do not check" for the final state. Players are
//! addressed by `{team, number}`; resolution to global indices happens when the scenario is loaded.

use crate::criterion::EventMatcher;
use serde::de::{self, Deserializer};
use serde::Deserialize;
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Score, Velocity3D};

/// Stage name as written in scenario files; the Setup reason is stored separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StageNameDto {
    Play,
    Setup,
    GameOver,
}

/// Football event kind used by stop criteria and journal expectations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum EventKindDto {
    Goal,
    Touchline,
    GoalLine,
    GameEnd,
}

/// Reference to a player by team and number (`tactical.toml` number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerRefDto {
    pub team: Team,
    pub number: u32,
}

/// Marker deserialized only from the string `"none"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoneToken;

impl<'de> Deserialize<'de> for NoneToken {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        if value == "none" {
            Ok(NoneToken)
        } else {
            Err(de::Error::custom(format!(
                "expected \"none\", got \"{value}\""
            )))
        }
    }
}

/// Ball owner: a player reference or the free-ball marker `"none"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum BallOwnerDto {
    Player(PlayerRefDto),
    Free(NoneToken),
}

/// A team or the absent-team marker `"none"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum TeamOrNoneDto {
    Team(Team),
    None(NoneToken),
}

/// Ball fields of a partial state. `velocity` is only meaningful for the initial state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BallStateDto {
    #[serde(default)]
    pub position: Option<Point3D>,
    #[serde(default)]
    pub velocity: Option<Velocity3D>,
    #[serde(default)]
    pub possessed_by: Option<BallOwnerDto>,
    #[serde(default)]
    pub last_possessing_team: Option<TeamOrNoneDto>,
}

/// Setup (`restart_position`/`restart_team`) fields of a partial state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SetupDto {
    #[serde(default)]
    pub restart_position: Option<Point3D>,
    #[serde(default)]
    pub restart_team: Option<TeamOrNoneDto>,
}

/// Explicit player position in meters, overriding the tactic-defined start position.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlayerPlacementDto {
    pub team: Team,
    pub number: u32,
    pub position: Point3D,
}

/// Partial initial state (`initial_state.toml`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InitialStateDto {
    #[serde(default)]
    pub stage: Option<StageNameDto>,
    #[serde(default)]
    pub setup_reason: Option<String>,
    #[serde(default)]
    pub ball: Option<BallStateDto>,
    #[serde(default)]
    pub setup: Option<SetupDto>,
    #[serde(default)]
    pub players: Vec<PlayerPlacementDto>,
}

/// Partial expected final state (`final_state.toml`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalStateDto {
    #[serde(default)]
    pub stage: Option<StageNameDto>,
    #[serde(default)]
    pub setup_reason: Option<String>,
    #[serde(default)]
    pub ball: Option<BallStateDto>,
    #[serde(default)]
    pub setup: Option<SetupDto>,
    #[serde(default)]
    pub players: Vec<PlayerPlacementDto>,
    #[serde(default)]
    pub score: Option<Score>,
}

impl InitialStateDto {
    /// Parses and validates `initial_state.toml`.
    pub fn parse(source: &str) -> Result<Self, String> {
        let state: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), String> {
        // The first Setup tick unconditionally rewrites the ball, so the ball must not be set for
        // a Setup stage; an omitted stage also resolves to Setup("kick off").
        let starts_in_setup = matches!(self.stage, None | Some(StageNameDto::Setup));
        if starts_in_setup && self.ball.is_some() {
            return Err(
                "ball state cannot be set for a Setup stage; use [setup] instead".to_string(),
            );
        }
        validate_players(&self.players)?;
        if let Some(ball) = &self.ball {
            validate_ball_owner(ball)?;
        }
        Ok(())
    }
}

impl FinalStateDto {
    /// Parses and validates `final_state.toml`.
    pub fn parse(source: &str) -> Result<Self, String> {
        let state: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), String> {
        validate_players(&self.players)?;
        if let Some(ball) = &self.ball {
            validate_ball_owner(ball)?;
        }
        Ok(())
    }
}

/// A stop criterion from `[[run.stop]]`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "when", rename_all = "snake_case")]
pub enum StopCriterionDto {
    Stage {
        stage: StageNameDto,
        #[serde(default)]
        setup_reason: Option<String>,
    },
    Event {
        event: EventKindDto,
        #[serde(default)]
        team: Option<Team>,
    },
    Steps {
        steps: u64,
    },
    Time {
        time: f32,
    },
}

impl StopCriterionDto {
    /// Rejects combinations the runtime matchers cannot represent (a team on `GameEnd`).
    fn validate(&self) -> Result<(), String> {
        if let StopCriterionDto::Event { event, team } = self {
            EventMatcher::from_def(*event, *team)?;
        }
        Ok(())
    }
}

/// Run plan from `[run]` in `scenario.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunPlanDto {
    pub dt: f32,
    #[serde(default)]
    pub stop: Vec<StopCriterionDto>,
}

impl RunPlanDto {
    /// Parses and validates a run plan; a `steps` or `time` safety criterion is required.
    pub fn parse(source: &str) -> Result<Self, String> {
        let plan: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        plan.validate()?;
        Ok(plan)
    }

    fn validate(&self) -> Result<(), String> {
        if self.dt <= 0.0 || !self.dt.is_finite() {
            return Err(format!("run.dt must be a positive number, got {}", self.dt));
        }
        for criterion in &self.stop {
            criterion.validate()?;
        }
        let has_safety = self.stop.iter().any(|criterion| {
            matches!(
                criterion,
                StopCriterionDto::Steps { .. } | StopCriterionDto::Time { .. }
            )
        });
        if !has_safety {
            return Err("run.stop must contain a 'steps' or 'time' safety criterion".to_string());
        }
        Ok(())
    }
}

/// How the expected journal list is compared with the actual one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalMatchDto {
    #[default]
    Exact,
    Subsequence,
}

/// One expected journal record (`[[expect.journal]]`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpectedEventDto {
    DecisionAssigned {
        #[serde(default)]
        player: Option<PlayerRefDto>,
        #[serde(default)]
        decision: Option<String>,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        at: Option<f32>,
    },
    PossessionChange {
        #[serde(default)]
        possessed_by: Option<BallOwnerDto>,
        #[serde(default)]
        last_possessing_team: Option<TeamOrNoneDto>,
        #[serde(default)]
        at: Option<f32>,
    },
    KickOutcome {
        #[serde(default)]
        player: Option<PlayerRefDto>,
        #[serde(default)]
        ball_velocity: Option<Velocity3D>,
        #[serde(default)]
        at: Option<f32>,
    },
    StageChange {
        #[serde(default)]
        stage: Option<StageNameDto>,
        #[serde(default)]
        setup_reason: Option<String>,
        #[serde(default)]
        at: Option<f32>,
    },
    RestartSet {
        #[serde(default)]
        position: Option<Point3D>,
        #[serde(default)]
        team: Option<TeamOrNoneDto>,
        #[serde(default)]
        at: Option<f32>,
    },
    DecisionsReset {
        #[serde(default)]
        at: Option<f32>,
    },
    StatUpdate {
        #[serde(default)]
        team: Option<Team>,
        #[serde(default)]
        key: Option<String>,
        #[serde(default)]
        delta: Option<f64>,
        #[serde(default)]
        at: Option<f32>,
    },
    FootballEvent {
        event: EventKindDto,
        #[serde(default)]
        team: Option<Team>,
        #[serde(default)]
        at: Option<f32>,
    },
}

impl ExpectedEventDto {
    /// Rejects combinations the runtime matchers cannot represent (a team on `GameEnd`).
    fn validate(&self) -> Result<(), String> {
        if let ExpectedEventDto::FootballEvent { event, team, .. } = self {
            EventMatcher::from_def(*event, *team)?;
        }
        Ok(())
    }
}

/// Expected stop in `[expect.stop]`: the criterion that ended the run, plus an optional exact
/// number of steps taken. For `when = "steps"` the limit and the step count are the same field.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "when", rename_all = "snake_case")]
pub enum StopExpectationDto {
    Stage {
        #[serde(default)]
        setup_reason: Option<String>,
        #[serde(default)]
        steps: Option<u64>,
    },
    Event {
        event: EventKindDto,
        #[serde(default)]
        team: Option<Team>,
        #[serde(default)]
        steps: Option<u64>,
    },
    Steps {
        steps: u64,
    },
    Time {
        time: f32,
        #[serde(default)]
        steps: Option<u64>,
    },
}

impl StopExpectationDto {
    /// Rejects combinations the runtime matchers cannot represent (a team on `GameEnd`).
    fn validate(&self) -> Result<(), String> {
        if let StopExpectationDto::Event { event, team, .. } = self {
            EventMatcher::from_def(*event, *team)?;
        }
        Ok(())
    }
}

/// Expectations from `[expect]` in `scenario.toml`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectDto {
    #[serde(default)]
    pub journal_match: JournalMatchDto,
    #[serde(default)]
    pub stop: Option<StopExpectationDto>,
    #[serde(default)]
    pub journal: Vec<ExpectedEventDto>,
}

/// Parsed `scenario.toml`: run plan plus expectations.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioDto {
    pub run: RunPlanDto,
    #[serde(default)]
    pub expect: ExpectDto,
}

impl ScenarioDto {
    /// Parses and validates `scenario.toml`.
    pub fn parse(source: &str) -> Result<Self, String> {
        let scenario: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        scenario.run.validate()?;
        if let Some(stop) = &scenario.expect.stop {
            stop.validate()?;
        }
        for entry in &scenario.expect.journal {
            entry.validate()?;
        }
        Ok(scenario)
    }
}

fn validate_players(players: &[PlayerPlacementDto]) -> Result<(), String> {
    for player in players {
        validate_player_number(player.team, player.number)?;
    }
    Ok(())
}

fn validate_ball_owner(ball: &BallStateDto) -> Result<(), String> {
    if let Some(BallOwnerDto::Player(player)) = ball.possessed_by {
        validate_player_number(player.team, player.number)?;
    }
    Ok(())
}

fn validate_player_number(team: Team, number: u32) -> Result<(), String> {
    if number == 0 {
        return Err(format!("player number must be positive for team {team:?}"));
    }
    Ok(())
}

#[cfg(test)]
#[path = "tests/dto_tests.rs"]
mod tests;
