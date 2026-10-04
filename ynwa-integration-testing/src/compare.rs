//! Expectation matching: compare a finished run with the scenario's declared expectations.
//!
//! Every expectation is partial: an absent field is not checked. Coordinates, velocities and
//! stat deltas are compared with [`TOLERANCE`]; timestamps with `max(dt, TOLERANCE)`. Mismatches
//! are collected as human-readable `expected`/`actual` lines instead of failing on the first one;
//! only a structural problem (an unresolvable player reference) is a hard error.
//!
//! Expectations are compared against the concrete journal records directly: a thin dispatch picks
//! the variant, while small field matchers (`field_eq`, `owner_matches`, `team_matches`, ...)
//! carry the partiality, tolerances and `{team, number}` resolution.

use uom::si::length::meter;
use uom::si::velocity::meter_per_second;

use ynwa_core::game::{Decision, GameConfig, GameStage, GameState};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::team::Team;
use ynwa_core::{Point3D, Velocity3D};
use ynwa_football::decode_football_events;
use ynwa_football::events::FootballEvent;

use crate::criterion::{EventMatcher, StageMatcher, StopReason};
use crate::dto::{
    BallOwnerDto, BallStateDto, ExpectDto, ExpectedEventDto, ExpectedFinalStateDto,
    JournalMatchDto, PlayerRefDto, SetupDto, StageNameDto, StopExpectationDto, TeamOrNoneDto,
};
use crate::loader::{resolve_player_index, ScenarioError};
use crate::runner::RunOutcome;

/// Absolute tolerance for coordinates (m), velocities (m/s) and stat deltas.
pub const TOLERANCE: f32 = 1e-4;

/// Compares a finished run with the scenario expectations, returning the list of mismatches
/// (empty when everything matched).
pub fn compare_outcome(
    outcome: &RunOutcome,
    expect: &ExpectDto,
    final_state: Option<&ExpectedFinalStateDto>,
    config: &GameConfig,
    dt: f32,
) -> Result<Vec<String>, ScenarioError> {
    let mut diffs = if expect.journal.is_empty() {
        Vec::new()
    } else {
        compare_journal(
            &expect.journal,
            &outcome.journal,
            expect.journal_match,
            config,
            dt,
        )?
    };
    if let Some(stop) = &expect.stop {
        diffs.extend(compare_stop(stop, &outcome.stop_reason, outcome.steps));
    }
    if let Some(final_state) = final_state {
        diffs.extend(compare_final_state(
            final_state,
            &outcome.final_state,
            config,
        )?);
    }
    Ok(diffs)
}

/// Compares the expected journal list with the actual entries in `exact` or `subsequence` mode.
fn compare_journal(
    expected: &[ExpectedEventDto],
    actual: &[JournalEntry],
    mode: JournalMatchDto,
    config: &GameConfig,
    dt: f32,
) -> Result<Vec<String>, ScenarioError> {
    let mut diffs = Vec::new();
    match mode {
        JournalMatchDto::Exact => {
            if expected.len() != actual.len() {
                diffs.push(format!(
                    "journal length: expected {}, got {}",
                    expected.len(),
                    actual.len()
                ));
            }
            for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
                if !entry_matches(expected, actual, config, dt)? {
                    diffs.push(format!(
                        "journal[{index}]: expected {expected:?}, got {:?}",
                        actual.event
                    ));
                }
            }
        }
        JournalMatchDto::Subsequence => {
            let mut cursor = 0;
            for expected in expected {
                let mut matched_at = None;
                for (offset, actual) in actual[cursor..].iter().enumerate() {
                    if entry_matches(expected, actual, config, dt)? {
                        matched_at = Some(cursor + offset);
                        break;
                    }
                }
                match matched_at {
                    Some(index) => cursor = index + 1,
                    None => diffs.push(format!(
                        "journal: expected event {expected:?} not found in order"
                    )),
                }
            }
        }
    }
    Ok(diffs)
}

/// Whether one expected journal record matches a concrete entry: the variant must be the same and
/// every specified field must match.
fn entry_matches(
    expected: &ExpectedEventDto,
    actual: &JournalEntry,
    config: &GameConfig,
    dt: f32,
) -> Result<bool, ScenarioError> {
    let matched = match (expected, &actual.event) {
        (
            ExpectedEventDto::DecisionAssigned {
                player,
                decision,
                reason,
                ..
            },
            JournalEvent::DecisionAssigned {
                player_index,
                decision: actual_decision,
                reason: actual_reason,
            },
        ) => {
            player_matches(player, *player_index, config)?
                && decision_field_matches(decision, actual_decision)
                && optional_field_eq(reason, actual_reason)
        }
        (
            ExpectedEventDto::PossessionChange {
                possessed_by,
                last_possessing_team,
                ..
            },
            JournalEvent::PossessionChange {
                possessed_by: actual_owner,
                last_possessing_team: actual_team,
            },
        ) => {
            owner_matches(possessed_by, *actual_owner, config)?
                && team_matches(last_possessing_team, *actual_team)
        }
        (
            ExpectedEventDto::KickOutcome {
                player,
                ball_velocity,
                ..
            },
            JournalEvent::KickOutcome {
                player_index,
                ball_velocity: actual_velocity,
            },
        ) => {
            player_matches(player, *player_index, config)?
                && velocity_field_matches(ball_velocity, actual_velocity)
        }
        (
            ExpectedEventDto::StageChange {
                stage,
                setup_reason,
                ..
            },
            JournalEvent::StageChange {
                stage: actual_stage,
            },
        ) => stage_matches(stage, setup_reason, actual_stage),
        (
            ExpectedEventDto::RestartSet { position, team, .. },
            JournalEvent::RestartSet {
                restart_position: actual_position,
                restart_team: actual_team,
            },
        ) => point_field_matches(position, actual_position) && team_matches(team, *actual_team),
        (ExpectedEventDto::DecisionsReset { .. }, JournalEvent::DecisionsReset) => true,
        (
            ExpectedEventDto::StatUpdate {
                team, key, delta, ..
            },
            JournalEvent::StatUpdate {
                team: actual_team,
                key: actual_key,
                delta: actual_delta,
            },
        ) => {
            field_eq(team, actual_team)
                && field_eq(key, actual_key)
                && delta_field_matches(delta, *actual_delta)
        }
        (ExpectedEventDto::FootballEvent { event, team, .. }, JournalEvent::External { .. }) => {
            let matcher = EventMatcher::from_def(*event, *team).map_err(ScenarioError::new)?;
            decode_single(actual)
                .is_some_and(|football_event| matcher.matches_football(&football_event))
        }
        _ => false,
    };

    Ok(matched && entry_timestamp_matches(expected, actual.timestamp, dt))
}

fn compare_stop(expected: &StopExpectationDto, actual: &StopReason, steps: u64) -> Vec<String> {
    let mut diffs = Vec::new();

    let reason_matches = match expected {
        StopExpectationDto::Stage {
            stage,
            setup_reason,
            ..
        } => match actual {
            StopReason::Stage(matcher) => stop_stage_matches(stage, setup_reason, matcher),
            _ => false,
        },
        StopExpectationDto::Event { event, team, .. } => {
            match EventMatcher::from_def(*event, *team) {
                Ok(matcher) => actual == &StopReason::Event(matcher),
                Err(_) => false,
            }
        }
        StopExpectationDto::Steps { steps } => actual == &StopReason::Steps(*steps),
        StopExpectationDto::Time { time, .. } => match actual {
            StopReason::Time(actual_time) => near(*time, *actual_time),
            _ => false,
        },
    };
    record(&mut diffs, reason_matches, || {
        format!("stop reason: expected {expected:?}, got {actual:?}")
    });

    if let Some(expected_steps) = expected_steps(expected) {
        record(&mut diffs, steps == expected_steps, || {
            format!("stop steps: expected {expected_steps}, got {steps}")
        });
    }

    diffs
}

fn compare_final_state(
    expected: &ExpectedFinalStateDto,
    actual: &GameState,
    config: &GameConfig,
) -> Result<Vec<String>, ScenarioError> {
    let mut diffs = Vec::new();

    compare_final_stage(expected, actual, &mut diffs);
    compare_final_score(expected, actual, &mut diffs);
    if let Some(ball) = &expected.ball {
        diffs.extend(compare_final_ball(ball, actual, config)?);
    }
    if let Some(setup) = &expected.setup {
        diffs.extend(compare_final_setup(setup, actual));
    }
    for placement in &expected.players {
        let index = resolve_player_index(config, placement.team, placement.number)?;
        let actual_position = actual.player_states[index].position;
        record(
            &mut diffs,
            point_close(placement.position, actual_position),
            || {
                format!(
                    "final_state.player {:?}#{}: expected {:?}, got {actual_position:?}",
                    placement.team, placement.number, placement.position
                )
            },
        );
    }

    Ok(diffs)
}

fn compare_final_stage(
    expected: &ExpectedFinalStateDto,
    actual: &GameState,
    diffs: &mut Vec<String>,
) {
    if let Some(stage) = &expected.stage {
        record(
            diffs,
            stage_matches(&expected.stage, &expected.setup_reason, &actual.stage),
            || {
                format!(
                    "final_state.stage: expected {stage:?}, got {:?}",
                    actual.stage
                )
            },
        );
    } else if let Some(reason) = &expected.setup_reason {
        record(
            diffs,
            matches!(&actual.stage, GameStage::Setup(actual_reason) if actual_reason == reason),
            || {
                format!(
                    "final_state.setup_reason: expected {reason:?}, got {:?}",
                    actual.stage
                )
            },
        );
    }
}

fn compare_final_score(
    expected: &ExpectedFinalStateDto,
    actual: &GameState,
    diffs: &mut Vec<String>,
) {
    let Some(score) = &expected.score else {
        return;
    };
    for (team, expected_score) in [(Team::A, score.a), (Team::B, score.b)] {
        let actual_score = actual
            .team_stats
            .get(&team)
            .map_or(0.0, |stats| stats.get("score"));
        record(
            diffs,
            (actual_score - expected_score as f64).abs() <= TOLERANCE as f64,
            || {
                format!(
                    "final_state.score[{team:?}]: expected {expected_score}, got {actual_score}"
                )
            },
        );
    }
}

fn compare_final_ball(
    ball: &BallStateDto,
    actual: &GameState,
    config: &GameConfig,
) -> Result<Vec<String>, ScenarioError> {
    let mut diffs = Vec::new();

    if let Some(position) = ball.position {
        record(
            &mut diffs,
            point_close(position, actual.ball_state.position),
            || {
                format!(
                    "final_state.ball.position: expected {position:?}, got {:?}",
                    actual.ball_state.position
                )
            },
        );
    }
    if let Some(velocity) = ball.velocity {
        record(
            &mut diffs,
            velocity_close(velocity, actual.ball_state.velocity),
            || {
                format!(
                    "final_state.ball.velocity: expected {velocity:?}, got {:?}",
                    actual.ball_state.velocity
                )
            },
        );
    }
    record(
        &mut diffs,
        owner_matches(&ball.possessed_by, actual.ball_state.possessed_by, config)?,
        || {
            format!(
                "final_state.ball.possessed_by: expected {:?}, got {:?}",
                ball.possessed_by, actual.ball_state.possessed_by
            )
        },
    );
    record(
        &mut diffs,
        team_matches(
            &ball.last_possessing_team,
            actual.ball_state.last_possessing_team,
        ),
        || {
            format!(
                "final_state.ball.last_possessing_team: expected {:?}, got {:?}",
                ball.last_possessing_team, actual.ball_state.last_possessing_team
            )
        },
    );

    Ok(diffs)
}

fn compare_final_setup(setup: &SetupDto, actual: &GameState) -> Vec<String> {
    let mut diffs = Vec::new();

    if let Some(position) = setup.restart_position {
        let close = actual
            .restart_position
            .is_some_and(|actual_position| point_close(position, actual_position));
        record(&mut diffs, close, || {
            format!(
                "final_state.setup.restart_position: expected {position:?}, got {:?}",
                actual.restart_position
            )
        });
    }
    record(
        &mut diffs,
        team_matches(&setup.restart_team, actual.restart_team),
        || {
            format!(
                "final_state.setup.restart_team: expected {:?}, got {:?}",
                setup.restart_team, actual.restart_team
            )
        },
    );

    diffs
}

/// Pushes `message` when the check failed; keeps every check site to a single line.
fn record(diffs: &mut Vec<String>, ok: bool, message: impl FnOnce() -> String) {
    if !ok {
        diffs.push(message());
    }
}

/// `None` expected means "not checked"; otherwise exact equality with the actual field.
fn field_eq<T: PartialEq>(expected: &Option<T>, actual: &T) -> bool {
    expected.as_ref().is_none_or(|expected| expected == actual)
}

/// Like [`field_eq`] for an actual field that is itself optional.
fn optional_field_eq<T: PartialEq>(expected: &Option<T>, actual: &Option<T>) -> bool {
    expected.is_none() || expected == actual
}

fn delta_field_matches(expected: &Option<f64>, actual: f64) -> bool {
    expected.is_none_or(|expected| (expected - actual).abs() <= TOLERANCE as f64)
}

fn velocity_field_matches(expected: &Option<Velocity3D>, actual: &Velocity3D) -> bool {
    expected.is_none_or(|expected| velocity_close(expected, *actual))
}

fn point_field_matches(expected: &Option<Point3D>, actual: &Option<Point3D>) -> bool {
    expected.is_none_or(|expected| actual.is_some_and(|actual| point_close(expected, actual)))
}

fn decision_field_matches(expected: &Option<String>, actual: &Decision) -> bool {
    expected
        .as_ref()
        .is_none_or(|expected| decision_matches(expected, actual))
}

/// `None` expected means "not checked"; otherwise the reference must resolve to the actual index.
fn player_matches(
    expected: &Option<PlayerRefDto>,
    actual: usize,
    config: &GameConfig,
) -> Result<bool, ScenarioError> {
    match expected {
        None => Ok(true),
        Some(reference) => {
            Ok(resolve_player_index(config, reference.team, reference.number)? == actual)
        }
    }
}

/// `None` expected means "not checked"; `"none"` means the ball must be free; a player reference
/// must resolve to the actual owner index.
fn owner_matches(
    expected: &Option<BallOwnerDto>,
    actual: Option<usize>,
    config: &GameConfig,
) -> Result<bool, ScenarioError> {
    let expected = match expected {
        None => return Ok(true),
        Some(BallOwnerDto::Free(_)) => None,
        Some(BallOwnerDto::Player(reference)) => Some(resolve_player_index(
            config,
            reference.team,
            reference.number,
        )?),
    };
    Ok(expected == actual)
}

/// `None` expected means "not checked"; `"none"` means the actual team must be absent.
fn team_matches(expected: &Option<TeamOrNoneDto>, actual: Option<Team>) -> bool {
    match expected {
        None => true,
        Some(TeamOrNoneDto::None(_)) => actual.is_none(),
        Some(TeamOrNoneDto::Team(team)) => actual == Some(*team),
    }
}

/// Matches a stage expectation against an actual stage. An omitted `stage` does not restrict the
/// target; a bare `setup_reason` still pins a `Setup` transition to that reason.
fn stage_matches(
    stage: &Option<StageNameDto>,
    setup_reason: &Option<String>,
    actual: &GameStage,
) -> bool {
    match stage {
        Some(StageNameDto::Play) => matches!(actual, GameStage::Play),
        Some(StageNameDto::GameOver) => matches!(actual, GameStage::GameOver),
        Some(StageNameDto::Setup) => setup_stage_matches(setup_reason, actual),
        None => match setup_reason {
            None => true,
            Some(_) => setup_stage_matches(setup_reason, actual),
        },
    }
}

fn setup_stage_matches(setup_reason: &Option<String>, actual: &GameStage) -> bool {
    match actual {
        GameStage::Setup(reason) => setup_reason
            .as_ref()
            .is_none_or(|expected| expected == reason),
        _ => false,
    }
}

/// Matches a `[expect.stop] when = "stage"` expectation against the stage matcher that ended the
/// run. A bare `setup_reason` still requires `Setup`.
fn stop_stage_matches(
    stage: &StageNameDto,
    setup_reason: &Option<String>,
    actual: &StageMatcher,
) -> bool {
    let stage_ok = matches!(
        (stage, actual),
        (StageNameDto::Play, StageMatcher::Play)
            | (StageNameDto::GameOver, StageMatcher::GameOver)
            | (StageNameDto::Setup, StageMatcher::Setup { .. })
    );

    stage_ok
        && setup_reason.as_ref().is_none_or(|expected| {
            matches!(actual, StageMatcher::Setup { reason: Some(reason) } if reason == expected)
        })
}

fn entry_timestamp_matches(expected: &ExpectedEventDto, actual: f32, dt: f32) -> bool {
    expected_at(expected).is_none_or(|expected| timestamps_match(expected, actual, dt))
}

/// Scenario files name a decision by its kind (`Stop` / `Run` / `Kick`); the target is not part
/// of the expected string.
fn decision_matches(expected: &str, actual: &Decision) -> bool {
    let actual = match actual {
        Decision::Run(_) => "Run",
        Decision::Stop => "Stop",
        Decision::Kick(_) => "Kick",
    };
    expected == actual
}

fn expected_at(expected: &ExpectedEventDto) -> Option<f32> {
    match expected {
        ExpectedEventDto::DecisionAssigned { at, .. }
        | ExpectedEventDto::PossessionChange { at, .. }
        | ExpectedEventDto::KickOutcome { at, .. }
        | ExpectedEventDto::StageChange { at, .. }
        | ExpectedEventDto::RestartSet { at, .. }
        | ExpectedEventDto::DecisionsReset { at }
        | ExpectedEventDto::StatUpdate { at, .. }
        | ExpectedEventDto::FootballEvent { at, .. } => *at,
    }
}

fn expected_steps(expected: &StopExpectationDto) -> Option<u64> {
    match expected {
        StopExpectationDto::Stage { steps, .. }
        | StopExpectationDto::Event { steps, .. }
        | StopExpectationDto::Time { steps, .. } => *steps,
        StopExpectationDto::Steps { steps } => Some(*steps),
    }
}

fn decode_single(entry: &JournalEntry) -> Option<FootballEvent> {
    decode_football_events(std::slice::from_ref(entry))
        .into_iter()
        .next()
        .map(|(_, event)| event)
}

fn point_close(a: Point3D, b: Point3D) -> bool {
    near(a.x.get::<meter>(), b.x.get::<meter>())
        && near(a.y.get::<meter>(), b.y.get::<meter>())
        && near(a.z.get::<meter>(), b.z.get::<meter>())
}

fn velocity_close(a: Velocity3D, b: Velocity3D) -> bool {
    near(a.x.get::<meter_per_second>(), b.x.get::<meter_per_second>())
        && near(a.y.get::<meter_per_second>(), b.y.get::<meter_per_second>())
        && near(a.z.get::<meter_per_second>(), b.z.get::<meter_per_second>())
}

fn timestamps_match(expected: f32, actual: f32, dt: f32) -> bool {
    (expected - actual).abs() <= dt.max(TOLERANCE)
}

fn near(a: f32, b: f32) -> bool {
    (a - b).abs() <= TOLERANCE
}

#[cfg(test)]
#[path = "tests/compare_tests.rs"]
mod tests;
