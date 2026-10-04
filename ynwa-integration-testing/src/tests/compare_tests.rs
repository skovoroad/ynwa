use std::collections::HashMap;

use super::*;
use crate::dto::{
    BallStateDto, EventKindDto, NoneToken, PlayerPlacementDto, PlayerRefDto, SetupDto,
};
use ynwa_core::field::Field;
use ynwa_core::game::{
    BallDef, DecisionTarget, PlayerDef, RefereeDef, ScriptingConfig, REGION_START_POSITION,
};
use ynwa_core::region::GridCell;
use ynwa_core::{DefaultRngManager, Game, RngConfig, Score};

fn test_config(players: &[(Team, u32)]) -> GameConfig {
    let field = Field::from_meters(68.0, 104.615_38, 26, 40);
    let grid_dims = field.grid_dimensions();
    let defs = players
        .iter()
        .map(|&(team, number)| {
            let region = grid_dims
                .create_region(GridCell::new(1, 1).unwrap(), GridCell::new(2, 2).unwrap())
                .unwrap();
            PlayerDef::new(
                team,
                number,
                format!("Player {team:?}{number}"),
                String::new(),
                HashMap::from([(REGION_START_POSITION.to_string(), region)]),
            )
        })
        .collect();
    GameConfig {
        field,
        players: defs,
        ball: BallDef::default(),
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig::empty(),
    }
}

fn test_state(players: &[(Team, u32)]) -> (GameConfig, GameState) {
    let config = test_config(players);
    let game = Game::with_stage(
        config.clone(),
        GameStage::Play,
        Box::new(DefaultRngManager::new(RngConfig::new(0.0, Some(0)))),
    );
    (config, game.state.clone())
}

fn entry(timestamp: f32, event: JournalEvent) -> JournalEntry {
    JournalEntry { timestamp, event }
}

fn football_entry(timestamp: f32, event: FootballEvent) -> JournalEntry {
    JournalEntry {
        timestamp,
        event: JournalEvent::External {
            kind: "football_event".to_string(),
            data: serde_json::to_value(event).unwrap(),
        },
    }
}

fn decision_assigned(player_index: usize, decision: Decision) -> JournalEvent {
    JournalEvent::DecisionAssigned {
        player_index,
        decision,
        reason: None,
    }
}

fn journal_diffs(
    expected: &[ExpectedEventDto],
    actual: &[JournalEntry],
    mode: JournalMatchDto,
    config: &GameConfig,
) -> Vec<String> {
    compare_journal(expected, actual, mode, config, 0.1).unwrap()
}

#[test]
fn exact_journal_match_passes() {
    let config = test_config(&[(Team::A, 9)]);
    let expected = vec![ExpectedEventDto::DecisionAssigned {
        player: Some(PlayerRefDto {
            team: Team::A,
            number: 9,
        }),
        decision: Some("Stop".to_string()),
        reason: Some("hold".to_string()),
        at: None,
    }];
    let actual = vec![entry(
        0.1,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            decision: Decision::Stop,
            reason: Some("hold".to_string()),
        },
    )];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn exact_journal_reports_length_and_field_mismatch() {
    let config = test_config(&[(Team::A, 9)]);
    let expected = vec![ExpectedEventDto::DecisionAssigned {
        player: Some(PlayerRefDto {
            team: Team::A,
            number: 9,
        }),
        decision: Some("Run".to_string()),
        reason: None,
        at: None,
    }];
    let actual = vec![
        entry(0.1, decision_assigned(0, Decision::Stop)),
        entry(0.2, JournalEvent::DecisionsReset),
    ];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);

    assert_eq!(diffs.len(), 2, "{diffs:?}");
    assert!(diffs[0].contains("journal length"), "{diffs:?}");
    assert!(diffs[1].contains("journal[0]"), "{diffs:?}");
}

#[test]
fn subsequence_skips_unlisted_events() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::StageChange {
        stage: Some(StageNameDto::Setup),
        setup_reason: Some("throw in".to_string()),
        at: None,
    }];
    let actual = vec![
        entry(0.1, JournalEvent::DecisionsReset),
        entry(
            0.2,
            JournalEvent::StageChange {
                stage: GameStage::Setup("throw in".to_string()),
            },
        ),
    ];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Subsequence, &config);

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn subsequence_reports_missing_event() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::DecisionsReset { at: None }];
    let actual = vec![entry(
        0.1,
        JournalEvent::StageChange {
            stage: GameStage::Play,
        },
    )];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Subsequence, &config);

    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(diffs[0].contains("not found"), "{diffs:?}");
}

#[test]
fn timestamp_is_checked_with_step_tolerance() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::DecisionsReset { at: Some(1.0) }];

    let close = vec![entry(1.05, JournalEvent::DecisionsReset)];
    assert!(journal_diffs(&expected, &close, JournalMatchDto::Exact, &config).is_empty());

    let far = vec![entry(1.2, JournalEvent::DecisionsReset)];
    assert_eq!(
        journal_diffs(&expected, &far, JournalMatchDto::Exact, &config).len(),
        1
    );
}

#[test]
fn football_event_matches_decoded_external_entry() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::FootballEvent {
        event: EventKindDto::Goal,
        team: Some(Team::B),
        at: None,
    }];

    let matching = vec![football_entry(0.3, FootballEvent::Goal(Team::B))];
    assert!(journal_diffs(&expected, &matching, JournalMatchDto::Exact, &config).is_empty());

    let mismatching = vec![football_entry(0.3, FootballEvent::Goal(Team::A))];
    assert_eq!(
        journal_diffs(&expected, &mismatching, JournalMatchDto::Exact, &config).len(),
        1
    );
}

#[test]
fn possession_change_resolves_player_reference() {
    let config = test_config(&[(Team::A, 1), (Team::B, 2)]);
    let expected = vec![ExpectedEventDto::PossessionChange {
        possessed_by: Some(BallOwnerDto::Player(PlayerRefDto {
            team: Team::B,
            number: 2,
        })),
        last_possessing_team: Some(TeamOrNoneDto::None(NoneToken)),
        at: None,
    }];
    let actual = vec![entry(
        0.1,
        JournalEvent::PossessionChange {
            possessed_by: Some(1),
            last_possessing_team: None,
        },
    )];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn kick_outcome_checks_velocity_with_tolerance() {
    let config = test_config(&[(Team::A, 9)]);
    let expected = vec![ExpectedEventDto::KickOutcome {
        player: Some(PlayerRefDto {
            team: Team::A,
            number: 9,
        }),
        ball_velocity: Some(Velocity3D::from_meters_per_second(1.0, 0.0, 2.0)),
        at: None,
    }];
    let actual = vec![entry(
        0.1,
        JournalEvent::KickOutcome {
            player_index: 0,
            ball_velocity: Velocity3D::from_meters_per_second(1.0 + TOLERANCE / 2.0, 0.0, 2.0),
        },
    )];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn stage_change_matches_play_and_game_over() {
    let config = test_config(&[(Team::A, 1)]);
    let cases = [
        (StageNameDto::Play, GameStage::Play, true),
        (StageNameDto::Play, GameStage::GameOver, false),
        (StageNameDto::GameOver, GameStage::GameOver, true),
    ];

    for (dto, stage, expected_match) in cases {
        let expected = vec![ExpectedEventDto::StageChange {
            stage: Some(dto),
            setup_reason: None,
            at: None,
        }];
        let actual = vec![entry(
            0.1,
            JournalEvent::StageChange {
                stage: stage.clone(),
            },
        )];

        let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);
        assert_eq!(diffs.is_empty(), expected_match, "{dto:?} vs {stage:?}");
    }
}

#[test]
fn unknown_player_reference_is_a_hard_error() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::DecisionAssigned {
        player: Some(PlayerRefDto {
            team: Team::B,
            number: 5,
        }),
        decision: None,
        reason: None,
        at: None,
    }];
    let actual = vec![entry(0.1, decision_assigned(0, Decision::Stop))];

    let result = compare_journal(&expected, &actual, JournalMatchDto::Exact, &config, 0.1);

    assert!(result.is_err());
}

#[test]
fn stop_matches_reason_and_step_count() {
    let expected = StopExpectationDto::Steps { steps: 7 };

    let diffs = compare_stop(&expected, &StopReason::Steps(7), 7);
    assert!(diffs.is_empty(), "{diffs:?}");

    let diffs = compare_stop(&expected, &StopReason::Steps(7), 8);
    assert_eq!(diffs.len(), 1, "{diffs:?}");
    assert!(diffs[0].contains("steps"), "{diffs:?}");
}

#[test]
fn stop_event_matches_with_optional_steps() {
    let expected = StopExpectationDto::Event {
        event: EventKindDto::Goal,
        team: Some(Team::B),
        steps: Some(15),
    };

    let diffs = compare_stop(
        &expected,
        &StopReason::Event(EventMatcher::Goal {
            team: Some(Team::B),
        }),
        15,
    );
    assert!(diffs.is_empty(), "{diffs:?}");

    let diffs = compare_stop(
        &expected,
        &StopReason::Event(EventMatcher::Goal {
            team: Some(Team::A),
        }),
        15,
    );
    assert_eq!(diffs.len(), 1, "{diffs:?}");
}

#[test]
fn stop_stage_and_time_are_compared() {
    let stage = StopExpectationDto::Stage {
        stage: Some(StageNameDto::Setup),
        setup_reason: Some("throw in".to_string()),
        steps: None,
    };
    let diffs = compare_stop(
        &stage,
        &StopReason::Stage(StageMatcher::Setup {
            reason: Some("throw in".to_string()),
        }),
        3,
    );
    assert!(diffs.is_empty(), "{diffs:?}");

    let time = StopExpectationDto::Time {
        time: 2.5,
        steps: None,
    };
    let diffs = compare_stop(&time, &StopReason::Time(2.5), 25);
    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn final_state_checks_stage_score_ball_and_players() {
    let (config, mut state) = test_state(&[(Team::A, 9), (Team::B, 4)]);
    state.stage = GameStage::Play;
    state
        .team_stats
        .get_mut(&Team::A)
        .unwrap()
        .set("score", 2.0);
    state.ball_state.position = Point3D::from_meters(34.0, 0.0, 10.0);
    state.ball_state.possessed_by = Some(0);
    state.ball_state.last_possessing_team = Some(Team::A);
    state.player_states[0].position = Point3D::from_meters(1.0, 0.0, 2.0);

    let expected = ExpectedFinalStateDto {
        stage: Some(StageNameDto::Play),
        setup_reason: None,
        ball: Some(BallStateDto {
            position: Some(Point3D::from_meters(34.0, 0.0, 10.0)),
            velocity: None,
            possessed_by: Some(BallOwnerDto::Player(PlayerRefDto {
                team: Team::A,
                number: 9,
            })),
            last_possessing_team: Some(TeamOrNoneDto::Team(Team::A)),
        }),
        setup: None,
        players: vec![PlayerPlacementDto {
            team: Team::A,
            number: 9,
            position: Point3D::from_meters(1.0, 0.0, 2.0),
        }],
        score: Some(Score { a: 2, b: 0 }),
    };

    let diffs = compare_final_state(&expected, &state, &config).unwrap();

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn final_state_reports_mismatches() {
    let (config, mut state) = test_state(&[(Team::A, 9)]);
    state.stage = GameStage::GameOver;
    state.ball_state.position = Point3D::from_meters(0.0, 0.0, 0.0);
    state.ball_state.possessed_by = Some(0);

    let expected = ExpectedFinalStateDto {
        stage: Some(StageNameDto::Play),
        setup_reason: None,
        ball: Some(BallStateDto {
            position: Some(Point3D::from_meters(5.0, 0.0, 5.0)),
            velocity: None,
            possessed_by: Some(BallOwnerDto::Free(NoneToken)),
            last_possessing_team: None,
        }),
        setup: None,
        players: vec![],
        score: Some(Score { a: 1, b: 0 }),
    };

    let diffs = compare_final_state(&expected, &state, &config).unwrap();

    assert!(diffs.iter().any(|diff| diff.contains("stage")), "{diffs:?}");
    assert!(diffs.iter().any(|diff| diff.contains("score")), "{diffs:?}");
    assert!(
        diffs.iter().any(|diff| diff.contains("ball.position")),
        "{diffs:?}"
    );
    assert!(
        diffs.iter().any(|diff| diff.contains("possessed_by")),
        "{diffs:?}"
    );
}

#[test]
fn final_state_checks_setup_restart() {
    let (config, mut state) = test_state(&[(Team::A, 9)]);
    state.restart_position = Some(Point3D::from_meters(1.0, 0.0, 2.0));
    state.restart_team = Some(Team::B);

    let expected = ExpectedFinalStateDto {
        stage: None,
        setup_reason: None,
        ball: None,
        setup: Some(SetupDto {
            restart_position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
            restart_team: Some(TeamOrNoneDto::Team(Team::B)),
        }),
        players: vec![],
        score: None,
    };

    let diffs = compare_final_state(&expected, &state, &config).unwrap();

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn compare_outcome_without_expectations_passes() {
    let (config, state) = test_state(&[(Team::A, 9)]);
    let outcome = RunOutcome {
        journal: vec![],
        football_events: vec![],
        stop_reason: StopReason::Steps(10),
        steps: 10,
        final_state: state,
    };

    let diffs = compare_outcome(&outcome, &ExpectDto::default(), None, &config, 0.1).unwrap();

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn empty_journal_expectation_is_not_checked() {
    let (config, state) = test_state(&[(Team::A, 9)]);
    let outcome = RunOutcome {
        journal: vec![entry(0.1, JournalEvent::DecisionsReset)],
        football_events: vec![],
        stop_reason: StopReason::Steps(10),
        steps: 10,
        final_state: state,
    };

    let diffs = compare_outcome(&outcome, &ExpectDto::default(), None, &config, 0.1).unwrap();

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn journal_restart_set_and_stat_update_match() {
    let config = test_config(&[(Team::A, 9)]);
    let expected = vec![
        ExpectedEventDto::RestartSet {
            position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
            team: Some(TeamOrNoneDto::Team(Team::A)),
            at: None,
        },
        ExpectedEventDto::StatUpdate {
            team: Some(Team::A),
            key: Some("score".to_string()),
            delta: Some(1.0),
            at: None,
        },
    ];
    let actual = vec![
        entry(
            0.1,
            JournalEvent::RestartSet {
                restart_position: Some(Point3D::from_meters(1.0, 0.0, 2.0)),
                restart_team: Some(Team::A),
            },
        ),
        entry(
            0.2,
            JournalEvent::StatUpdate {
                team: Team::A,
                key: "score".to_string(),
                delta: 1.0 + TOLERANCE as f64 / 2.0,
            },
        ),
    ];

    let diffs = journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config);

    assert!(diffs.is_empty(), "{diffs:?}");
}

#[test]
fn journal_matches_every_decision_kind() {
    let config = test_config(&[(Team::A, 1)]);
    let cases = [
        (
            "Run",
            Decision::Run(DecisionTarget::Point(Point3D::from_meters(1.0, 0.0, 1.0))),
        ),
        ("Stop", Decision::Stop),
        ("Kick", Decision::Kick(Point3D::from_meters(2.0, 0.0, 2.0))),
    ];

    for (kind, decision) in cases {
        let expected = vec![ExpectedEventDto::DecisionAssigned {
            player: None,
            decision: Some(kind.to_string()),
            reason: None,
            at: None,
        }];
        let actual = vec![entry(0.1, decision_assigned(0, decision))];

        assert!(
            journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config).is_empty(),
            "{kind} must match"
        );
    }
}

#[test]
fn football_event_game_end_matches() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::FootballEvent {
        event: EventKindDto::GameEnd,
        team: None,
        at: None,
    }];
    let actual = vec![football_entry(0.3, FootballEvent::GameEnd)];

    assert!(journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config).is_empty());
}

#[test]
fn stage_change_without_stage_matches_any_target() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::StageChange {
        stage: None,
        setup_reason: None,
        at: None,
    }];

    for stage in [
        GameStage::Play,
        GameStage::GameOver,
        GameStage::Setup("corner".to_string()),
    ] {
        let actual = vec![entry(0.1, JournalEvent::StageChange { stage })];

        assert!(
            journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config).is_empty(),
            "any stage must match"
        );
    }
}

#[test]
fn stage_change_without_stage_but_with_reason_requires_setup() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::StageChange {
        stage: None,
        setup_reason: Some("throw in".to_string()),
        at: None,
    }];

    let matching = vec![entry(
        0.1,
        JournalEvent::StageChange {
            stage: GameStage::Setup("throw in".to_string()),
        },
    )];
    assert!(journal_diffs(&expected, &matching, JournalMatchDto::Exact, &config).is_empty());

    let other_reason = vec![entry(
        0.1,
        JournalEvent::StageChange {
            stage: GameStage::Setup("corner".to_string()),
        },
    )];
    assert_eq!(
        journal_diffs(&expected, &other_reason, JournalMatchDto::Exact, &config).len(),
        1
    );
}

#[test]
fn journal_reports_reason_mismatch() {
    let config = test_config(&[(Team::A, 1)]);
    let expected = vec![ExpectedEventDto::DecisionAssigned {
        player: None,
        decision: None,
        reason: Some("hold".to_string()),
        at: None,
    }];
    let actual = vec![entry(
        0.1,
        JournalEvent::DecisionAssigned {
            player_index: 0,
            decision: Decision::Stop,
            reason: Some("run".to_string()),
        },
    )];

    assert_eq!(
        journal_diffs(&expected, &actual, JournalMatchDto::Exact, &config).len(),
        1
    );
}

#[test]
fn stop_stage_matches_target_stage() {
    let any = StopExpectationDto::Stage {
        stage: None,
        setup_reason: None,
        steps: None,
    };
    assert!(compare_stop(&any, &StopReason::Stage(StageMatcher::Play), 1).is_empty());

    let play = StopExpectationDto::Stage {
        stage: Some(StageNameDto::Play),
        setup_reason: None,
        steps: None,
    };
    assert!(compare_stop(&play, &StopReason::Stage(StageMatcher::Play), 1).is_empty());
    assert_eq!(
        compare_stop(&play, &StopReason::Stage(StageMatcher::GameOver), 1).len(),
        1
    );

    let setup = StopExpectationDto::Stage {
        stage: Some(StageNameDto::Setup),
        setup_reason: Some("throw in".to_string()),
        steps: Some(3),
    };
    assert!(compare_stop(
        &setup,
        &StopReason::Stage(StageMatcher::Setup {
            reason: Some("throw in".to_string()),
        }),
        3,
    )
    .is_empty());
    assert_eq!(
        compare_stop(
            &setup,
            &StopReason::Stage(StageMatcher::Setup {
                reason: Some("corner".to_string()),
            }),
            3,
        )
        .len(),
        1
    );
}

#[test]
fn final_state_checks_ball_velocity() {
    let (config, mut state) = test_state(&[(Team::A, 9)]);
    state.ball_state.velocity = Velocity3D::from_meters_per_second(1.0, 0.0, 2.0);

    let expected = ExpectedFinalStateDto {
        ball: Some(BallStateDto {
            velocity: Some(Velocity3D::from_meters_per_second(1.0, 0.0, 2.0)),
            ..BallStateDto::default()
        }),
        ..ExpectedFinalStateDto::default()
    };

    assert!(compare_final_state(&expected, &state, &config)
        .unwrap()
        .is_empty());
}

#[test]
fn final_state_reports_setup_restart_mismatch() {
    let (config, mut state) = test_state(&[(Team::A, 9)]);
    state.restart_position = Some(Point3D::from_meters(1.0, 0.0, 2.0));
    state.restart_team = None;

    let expected = ExpectedFinalStateDto {
        setup: Some(SetupDto {
            restart_position: Some(Point3D::from_meters(9.0, 0.0, 9.0)),
            restart_team: Some(TeamOrNoneDto::Team(Team::B)),
        }),
        ..ExpectedFinalStateDto::default()
    };

    let diffs = compare_final_state(&expected, &state, &config).unwrap();

    assert!(
        diffs.iter().any(|diff| diff.contains("restart_position")),
        "{diffs:?}"
    );
    assert!(
        diffs.iter().any(|diff| diff.contains("restart_team")),
        "{diffs:?}"
    );
}

#[test]
fn final_state_checks_setup_reason_without_stage() {
    let (config, mut state) = test_state(&[(Team::A, 9)]);
    state.stage = GameStage::Setup("throw in".to_string());

    let matching = ExpectedFinalStateDto {
        setup_reason: Some("throw in".to_string()),
        ..ExpectedFinalStateDto::default()
    };
    assert!(compare_final_state(&matching, &state, &config)
        .unwrap()
        .is_empty());

    let wrong = ExpectedFinalStateDto {
        setup_reason: Some("corner".to_string()),
        ..ExpectedFinalStateDto::default()
    };
    assert_eq!(
        compare_final_state(&wrong, &state, &config).unwrap().len(),
        1
    );
}
