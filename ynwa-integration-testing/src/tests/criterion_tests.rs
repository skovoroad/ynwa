use super::{EventMatcher, JournalMatcher, StageMatcher, StopCriterion, StopReason};
use crate::dto::{EventKindDto, JournalEntryPatternDto, StageNameDto};
use ynwa_core::field::Field;
use ynwa_core::game::{BallDef, GameConfig, GameStage, ScriptingConfig};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::team::Team;
use ynwa_core::Point3D;
use ynwa_football::events::FootballEvent;

fn test_config() -> GameConfig {
    GameConfig {
        field: Field::from_meters(68.0, 104.615_38, 26, 40),
        players: Vec::new(),
        ball: BallDef::default(),
        referees: Vec::new(),
        scripting: ScriptingConfig::empty(),
    }
}

#[test]
fn stage_matcher_covers_every_target() {
    let cases = [
        (StageMatcher::Play, GameStage::Play, true),
        (StageMatcher::Play, GameStage::GameOver, false),
        (
            StageMatcher::Play,
            GameStage::Setup("kick off".into()),
            false,
        ),
        (StageMatcher::GameOver, GameStage::GameOver, true),
        (StageMatcher::GameOver, GameStage::Play, false),
        (
            StageMatcher::Setup { reason: None },
            GameStage::Setup("throw in".into()),
            true,
        ),
        (StageMatcher::Setup { reason: None }, GameStage::Play, false),
        (
            StageMatcher::Setup {
                reason: Some("throw in".to_string()),
            },
            GameStage::Setup("throw in".into()),
            true,
        ),
        (
            StageMatcher::Setup {
                reason: Some("throw in".to_string()),
            },
            GameStage::Setup("corner".into()),
            false,
        ),
    ];

    for (matcher, actual, expected) in cases {
        assert_eq!(
            matcher.matches(&actual),
            expected,
            "matcher {matcher:?} against {actual:?}"
        );
    }
}

#[test]
fn event_matcher_matches_kind_and_optional_team() {
    let point = Point3D::from_meters(1.0, 0.0, 2.0);
    let goal_a = FootballEvent::Goal(Team::A);
    let touchline_b = FootballEvent::Touchline(point, Team::B);

    let cases = [
        (
            EventMatcher::Goal {
                team: Some(Team::A),
            },
            &goal_a,
            true,
        ),
        (
            EventMatcher::Goal {
                team: Some(Team::B),
            },
            &goal_a,
            false,
        ),
        (EventMatcher::Goal { team: None }, &goal_a, true),
        (
            EventMatcher::Touchline {
                team: Some(Team::B),
            },
            &touchline_b,
            true,
        ),
        (EventMatcher::Goal { team: None }, &touchline_b, false),
        (EventMatcher::GameEnd, &FootballEvent::GameEnd, true),
    ];

    for (matcher, event, expected) in cases {
        assert_eq!(
            matcher.matches_football(event),
            expected,
            "matcher {matcher:?} against {event:?}"
        );
    }
}

#[test]
fn goal_line_matcher_covers_team_variants() {
    let point = Point3D::from_meters(68.0, 0.0, 5.5);
    let goal_line_a = FootballEvent::GoalLine(point, Team::A);

    let cases = [
        (EventMatcher::GoalLine { team: None }, &goal_line_a, true),
        (
            EventMatcher::GoalLine {
                team: Some(Team::A),
            },
            &goal_line_a,
            true,
        ),
        (
            EventMatcher::GoalLine {
                team: Some(Team::B),
            },
            &goal_line_a,
            false,
        ),
    ];

    for (matcher, event, expected) in cases {
        assert_eq!(
            matcher.matches_football(event),
            expected,
            "matcher {matcher:?} against {event:?}"
        );
    }
}

#[test]
fn from_def_builds_matchers_and_rejects_team_on_game_end() {
    assert_eq!(
        EventMatcher::from_def(EventKindDto::Goal, Some(Team::A)).unwrap(),
        EventMatcher::Goal {
            team: Some(Team::A)
        }
    );
    assert_eq!(
        EventMatcher::from_def(EventKindDto::Touchline, None).unwrap(),
        EventMatcher::Touchline { team: None }
    );
    assert_eq!(
        EventMatcher::from_def(EventKindDto::GameEnd, None).unwrap(),
        EventMatcher::GameEnd
    );

    let error = EventMatcher::from_def(EventKindDto::GameEnd, Some(Team::A)).unwrap_err();
    assert!(error.contains("GameEnd"), "unexpected error: {error}");
}

#[test]
fn steps_and_time_criteria_fire_at_their_limits() {
    let config = test_config();
    let steps = StopCriterion::OnSteps(5);
    assert_eq!(
        steps
            .evaluate(&GameStage::Play, 0.0, 4, &[], &[], &config)
            .unwrap(),
        None
    );
    assert_eq!(
        steps
            .evaluate(&GameStage::Play, 0.0, 5, &[], &[], &config)
            .unwrap(),
        Some(StopReason::Steps(5))
    );

    let time = StopCriterion::OnTime(1.0);
    assert_eq!(
        time.evaluate(&GameStage::Play, 0.9, 0, &[], &[], &config)
            .unwrap(),
        None
    );
    assert_eq!(
        time.evaluate(&GameStage::Play, 1.0, 0, &[], &[], &config)
            .unwrap(),
        Some(StopReason::Time(1.0))
    );
}

#[test]
fn event_criterion_fires_on_a_new_matching_event() {
    let config = test_config();
    let criterion = StopCriterion::OnEvent(EventMatcher::Goal {
        team: Some(Team::B),
    });

    assert_eq!(
        criterion
            .evaluate(&GameStage::Play, 0.0, 1, &[], &[], &config)
            .unwrap(),
        None
    );
    assert_eq!(
        criterion
            .evaluate(
                &GameStage::Play,
                0.0,
                1,
                &[(0.5, FootballEvent::Goal(Team::B))],
                &[],
                &config,
            )
            .unwrap(),
        Some(StopReason::Event(EventMatcher::Goal {
            team: Some(Team::B),
        }))
    );
}

#[test]
fn journal_criterion_fires_on_a_matching_new_entry() {
    let config = test_config();
    let criterion = StopCriterion::OnJournal(JournalMatcher::new(
        JournalEntryPatternDto::StageChange {
            stage: Some(StageNameDto::Play),
            setup_reason: None,
            at: None,
        }
        .try_into()
        .unwrap(),
    ));
    let matching = JournalEntry {
        timestamp: 0.5,
        event: JournalEvent::StageChange {
            stage: GameStage::Play,
        },
    };
    let other = JournalEntry {
        timestamp: 0.5,
        event: JournalEvent::StageChange {
            stage: GameStage::GameOver,
        },
    };

    assert_eq!(
        criterion
            .evaluate(&GameStage::Play, 0.0, 1, &[], &[], &config)
            .unwrap(),
        None
    );
    assert_eq!(
        criterion
            .evaluate(&GameStage::Play, 0.0, 1, &[], &[other], &config)
            .unwrap(),
        None
    );
    assert_eq!(
        criterion
            .evaluate(
                &GameStage::Play,
                0.0,
                1,
                &[],
                std::slice::from_ref(&matching),
                &config,
            )
            .unwrap(),
        Some(StopReason::Journal(matching))
    );
}

#[test]
fn stage_match_helper_builds_every_variant() {
    assert_eq!(
        crate::criterion::stage_matcher(StageNameDto::Play, None),
        StageMatcher::Play
    );
    assert_eq!(
        crate::criterion::stage_matcher(StageNameDto::GameOver, None),
        StageMatcher::GameOver
    );
    assert_eq!(
        crate::criterion::stage_matcher(StageNameDto::Setup, Some("corner".to_string())),
        StageMatcher::Setup {
            reason: Some("corner".to_string())
        }
    );
}
