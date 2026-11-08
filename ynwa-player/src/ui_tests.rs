use super::event_text;
use serde_json::json;
use ynwa_core::field::zones::{Point3D, Velocity3D};
use ynwa_core::game::{Decision, GameStage};
use ynwa_core::journal::JournalEvent;
use ynwa_core::team::Team;
use ynwa_football::events::FootballEvent;

fn external(kind: &str, data: serde_json::Value) -> JournalEvent {
    JournalEvent::External {
        kind: kind.to_string(),
        data,
    }
}

fn football_external(event: FootballEvent) -> JournalEvent {
    external(
        "football_event",
        serde_json::to_value(event).expect("serialize football event"),
    )
}

#[test]
fn decision_assigned_renders_player_and_decision() {
    let cases = [
        (Some("press".to_string()), "#3 Stop (press)"),
        (None, "#3 Stop"),
    ];

    for (reason, expected) in cases {
        let event = JournalEvent::DecisionAssigned {
            player_index: 3,
            decision: Decision::Stop,
            reason,
        };
        assert_eq!(event_text(&event), expected);
    }
}

#[test]
fn possession_change_renders_possessor_and_last_team() {
    let cases = [
        (
            JournalEvent::PossessionChange {
                possessed_by: Some(4),
                last_possessing_team: Some(Team::B),
            },
            "Possession: #4 (last team B)",
        ),
        (
            JournalEvent::PossessionChange {
                possessed_by: None,
                last_possessing_team: None,
            },
            "Possession: none (last team none)",
        ),
    ];

    for (event, expected) in cases {
        assert_eq!(event_text(&event), expected);
    }
}

#[test]
fn kick_outcome_renders_kicker() {
    let event = JournalEvent::KickOutcome {
        player_index: 2,
        ball_velocity: Velocity3D::default(),
    };

    assert_eq!(event_text(&event), "Kick by #2");
}

#[test]
fn stage_change_renders_stage() {
    let cases = [
        (GameStage::Play, "Stage: Play"),
        (
            GameStage::Setup("corner".to_string()),
            "Stage: Setup(\"corner\")",
        ),
        (GameStage::GameOver, "Stage: GameOver"),
    ];

    for (stage, expected) in cases {
        assert_eq!(event_text(&JournalEvent::StageChange { stage }), expected);
    }
}

#[test]
fn restart_set_renders_team_or_cleared() {
    let cases = [
        (
            JournalEvent::RestartSet {
                restart_position: Some(Point3D::default()),
                restart_team: Some(Team::A),
            },
            "Restart: team A",
        ),
        (
            JournalEvent::RestartSet {
                restart_position: Some(Point3D::default()),
                restart_team: None,
            },
            "Restart: cleared",
        ),
        (
            JournalEvent::RestartSet {
                restart_position: None,
                restart_team: None,
            },
            "Restart: cleared",
        ),
    ];

    for (event, expected) in cases {
        assert_eq!(event_text(&event), expected);
    }
}

#[test]
fn decisions_reset_renders_label() {
    assert_eq!(event_text(&JournalEvent::DecisionsReset), "Decisions reset");
}

#[test]
fn stat_update_renders_team_key_and_signed_delta() {
    let cases = [
        (
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0,
            },
            "Stat: team A score +1",
        ),
        (
            JournalEvent::StatUpdate {
                team: Team::B,
                key: "fouls".to_string(),
                delta: -1.0,
            },
            "Stat: team B fouls -1",
        ),
    ];

    for (event, expected) in cases {
        assert_eq!(event_text(&event), expected);
    }
}

#[test]
fn football_external_events_keep_their_labels() {
    let point = Point3D::default();
    let cases = [
        (FootballEvent::Goal(Team::A), "Goal: team B"),
        (
            FootballEvent::Touchline(point, Team::B),
            "Throw-in; last touch: B",
        ),
        (
            FootballEvent::GoalLine(point, Team::A),
            "Goal line; last touch: A",
        ),
        (FootballEvent::GameEnd, "Match end"),
    ];

    for (event, expected) in cases {
        assert_eq!(event_text(&football_external(event)), expected);
    }
}

#[test]
fn non_football_external_renders_kind_and_data() {
    let event = external("weather", json!({ "wind": 3 }));

    assert_eq!(event_text(&event), "External(weather): {\"wind\":3}");
}

#[test]
fn undecodable_football_external_falls_back_to_generic() {
    let event = external("football_event", json!("not an event"));

    assert_eq!(
        event_text(&event),
        "External(football_event): \"not an event\""
    );
}
