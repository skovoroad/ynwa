//! Scenario model and its TOML parsing.
//!
//! A scenario directory holds a partial initial `GameState` (`initial_state.toml`), an optional
//! partial expected final state (`final_state.toml`) and a run plan with expectations
//! (`scenario.toml`). Every field of a partial state is optional: a missing field means "do not
//! set" for the initial state and "do not check" for the final state. Players are addressed by
//! `{team, number}`; resolution to global indices happens when the scenario is loaded.

use serde::de::{self, Deserializer};
use serde::Deserialize;
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Score, Velocity3D};

/// Stage name as written in scenario files; the Setup reason is stored separately.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum StageName {
    Play,
    Setup,
    GameOver,
}

/// Football event kind used by stop criteria and journal expectations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub enum EventKind {
    Goal,
    Touchline,
    GoalLine,
    GameEnd,
}

/// Reference to a player by team and number (`tactical.toml` number).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
pub struct PlayerRef {
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
pub enum BallOwner {
    Player(PlayerRef),
    Free(NoneToken),
}

/// A team or the absent-team marker `"none"`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
pub enum TeamOrNone {
    Team(Team),
    None(NoneToken),
}

/// Ball fields of a partial state. `velocity` is only meaningful for the initial state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct BallStateDef {
    #[serde(default)]
    pub position: Option<Point3D>,
    #[serde(default)]
    pub velocity: Option<Velocity3D>,
    #[serde(default)]
    pub possessed_by: Option<BallOwner>,
    #[serde(default)]
    pub last_possessing_team: Option<TeamOrNone>,
}

/// Setup (`restart_position`/`restart_team`) fields of a partial state.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct SetupDef {
    #[serde(default)]
    pub restart_position: Option<Point3D>,
    #[serde(default)]
    pub restart_team: Option<TeamOrNone>,
}

/// Explicit player position in meters, overriding the tactic-defined start position.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
pub struct PlayerPlacement {
    pub team: Team,
    pub number: u32,
    pub position: Point3D,
}

/// Partial initial state (`initial_state.toml`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct InitialState {
    #[serde(default)]
    pub stage: Option<StageName>,
    #[serde(default)]
    pub setup_reason: Option<String>,
    #[serde(default)]
    pub ball: Option<BallStateDef>,
    #[serde(default)]
    pub setup: Option<SetupDef>,
    #[serde(default)]
    pub players: Vec<PlayerPlacement>,
}

/// Partial expected final state (`final_state.toml`).
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct FinalState {
    #[serde(default)]
    pub stage: Option<StageName>,
    #[serde(default)]
    pub setup_reason: Option<String>,
    #[serde(default)]
    pub ball: Option<BallStateDef>,
    #[serde(default)]
    pub setup: Option<SetupDef>,
    #[serde(default)]
    pub players: Vec<PlayerPlacement>,
    #[serde(default)]
    pub score: Option<Score>,
}

impl InitialState {
    /// Parses and validates `initial_state.toml`.
    pub fn parse(source: &str) -> Result<Self, String> {
        let state: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        state.validate()?;
        Ok(state)
    }

    fn validate(&self) -> Result<(), String> {
        // The first Setup tick unconditionally rewrites the ball, so the ball must not be set for
        // a Setup stage; an omitted stage also resolves to Setup("kick off").
        let starts_in_setup = matches!(self.stage, None | Some(StageName::Setup));
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

impl FinalState {
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

/// Kind of a stop criterion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StopWhen {
    Stage,
    Event,
    Steps,
    Time,
}

/// A stop criterion from `[[run.stop]]`, also describing the expected stop in `[expect.stop]`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct StopCriterionDef {
    pub when: StopWhen,
    #[serde(default)]
    pub stage: Option<StageName>,
    #[serde(default)]
    pub setup_reason: Option<String>,
    #[serde(default)]
    pub event: Option<EventKind>,
    #[serde(default)]
    pub team: Option<Team>,
    #[serde(default)]
    pub steps: Option<u64>,
    #[serde(default)]
    pub time: Option<f32>,
}

/// Football event refined by an optional team.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EventMatcher {
    pub event: EventKind,
    pub team: Option<Team>,
}

impl StopCriterionDef {
    /// The event matcher carried by an event criterion, or `None` for other criteria.
    pub fn event_matcher(&self) -> Option<EventMatcher> {
        if self.when != StopWhen::Event {
            return None;
        }
        self.event.map(|event| EventMatcher {
            event,
            team: self.team,
        })
    }

    /// Checks that the field selected by `when` is present, so the criterion can actually fire.
    fn validate(&self) -> Result<(), String> {
        let missing = match self.when {
            StopWhen::Stage => self.stage.is_none().then_some("stage"),
            StopWhen::Event => self.event.is_none().then_some("event"),
            StopWhen::Steps => self.steps.is_none().then_some("steps"),
            StopWhen::Time => self.time.is_none().then_some("time"),
        };
        match missing {
            Some(field) => Err(format!(
                "stop criterion {:?} requires the '{field}' field",
                self.when
            )),
            None => Ok(()),
        }
    }
}

/// Run plan from `[run]` in `scenario.toml`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct RunPlan {
    pub dt: f32,
    #[serde(default)]
    pub stop: Vec<StopCriterionDef>,
}

impl RunPlan {
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
        let has_safety = self
            .stop
            .iter()
            .any(|criterion| matches!(criterion.when, StopWhen::Steps | StopWhen::Time));
        if !has_safety {
            return Err("run.stop must contain a 'steps' or 'time' safety criterion".to_string());
        }
        Ok(())
    }
}

/// How the expected journal list is compared with the actual one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JournalMatch {
    #[default]
    Exact,
    Subsequence,
}

/// One expected journal record (`[[expect.journal]]`).
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExpectedEventDef {
    DecisionAssigned {
        #[serde(default)]
        player: Option<PlayerRef>,
        #[serde(default)]
        decision: Option<String>,
        #[serde(default)]
        reason: Option<String>,
        #[serde(default)]
        at: Option<f32>,
    },
    PossessionChange {
        #[serde(default)]
        possessed_by: Option<BallOwner>,
        #[serde(default)]
        last_possessing_team: Option<TeamOrNone>,
        #[serde(default)]
        at: Option<f32>,
    },
    KickOutcome {
        #[serde(default)]
        player: Option<PlayerRef>,
        #[serde(default)]
        ball_velocity: Option<Velocity3D>,
        #[serde(default)]
        at: Option<f32>,
    },
    StageChange {
        #[serde(default)]
        stage: Option<StageName>,
        #[serde(default)]
        setup_reason: Option<String>,
        #[serde(default)]
        at: Option<f32>,
    },
    RestartSet {
        #[serde(default)]
        position: Option<Point3D>,
        #[serde(default)]
        team: Option<TeamOrNone>,
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
        #[serde(default)]
        event: Option<EventKind>,
        #[serde(default)]
        team: Option<Team>,
        #[serde(default)]
        at: Option<f32>,
    },
}

/// Expectations from `[expect]` in `scenario.toml`.
#[derive(Debug, Clone, Default, PartialEq, Deserialize)]
pub struct Expect {
    #[serde(default)]
    pub journal_match: JournalMatch,
    #[serde(default)]
    pub stop: Option<StopCriterionDef>,
    #[serde(default)]
    pub journal: Vec<ExpectedEventDef>,
}

/// Parsed `scenario.toml`: run plan plus expectations.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct ScenarioDef {
    pub run: RunPlan,
    #[serde(default)]
    pub expect: Expect,
}

impl ScenarioDef {
    /// Parses and validates `scenario.toml`.
    pub fn parse(source: &str) -> Result<Self, String> {
        let scenario: Self = toml::from_str(source).map_err(|error| error.to_string())?;
        scenario.run.validate()?;
        if let Some(stop) = &scenario.expect.stop {
            stop.validate()?;
        }
        Ok(scenario)
    }
}

fn validate_players(players: &[PlayerPlacement]) -> Result<(), String> {
    for player in players {
        validate_player_number(player.team, player.number)?;
    }
    Ok(())
}

fn validate_ball_owner(ball: &BallStateDef) -> Result<(), String> {
    if let Some(BallOwner::Player(player)) = ball.possessed_by {
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
