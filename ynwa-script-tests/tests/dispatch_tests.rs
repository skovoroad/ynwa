// Integration tests: verify the event-driven dispatch mechanism for Play and Setup stages

use ynwa_core::game::{Decision, GameStage};
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::team::Team;
use ynwa_core::System;
use ynwa_script_tests::{
    create_test_game_with_full_preambles_and_stage, load_test_script, request_decisions_for_all,
};

/// Run the spy script under given ball state configuration and return (decision, reason).
fn run_spy(
    stage: GameStage,
    possessed_by: Option<usize>,       // global player index owning the ball
    last_possessing_team: Option<Team>, // team last owning the ball
) -> (Option<Decision>, Option<String>) {
    let script = load_test_script("dispatch_spy.lua");
    let mut game = create_test_game_with_full_preambles_and_stage(&script, stage);
    game.state.ball_state.possessed_by = possessed_by;
    game.state.ball_state.last_possessing_team = last_possessing_team;
    request_decisions_for_all(&mut game);

    let decision_maker = ScriptedDecisionMaker::new(&game).expect("ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let state = &game.state().player_states[0];
    assert!(
        state.last_error.is_none(),
        "dispatch error: {:?}",
        state.last_error
    );
    (state.current_decision.clone(), state.decision_reason.clone())
}

// --- Play stage: possession state routing ---

#[test]
fn test_dispatch_i_have_ball() {
    // Player index 0 owns the ball → i_have_ball handler
    let (_, reason) = run_spy(GameStage::Play, Some(0), Some(Team::A));
    assert_eq!(reason.as_deref(), Some("spy:i_have_ball"));
}

#[test]
fn test_dispatch_ball_is_free() {
    // No owner, no last team → ball_is_free handler
    let (_, reason) = run_spy(GameStage::Play, None, None);
    assert_eq!(reason.as_deref(), Some("spy:ball_is_free"));
}

#[test]
fn test_dispatch_team_has_ball() {
    // Ball last owned by Team A, player is Team A, no current holder → team_has_ball handler
    let (_, reason) = run_spy(GameStage::Play, None, Some(Team::A));
    assert_eq!(reason.as_deref(), Some("spy:team_has_ball"));
}

#[test]
fn test_dispatch_opponent_has_ball() {
    // Ball last owned by Team B, player is Team A → opponent_has_ball handler
    let (_, reason) = run_spy(GameStage::Play, None, Some(Team::B));
    assert_eq!(reason.as_deref(), Some("spy:opponent_has_ball"));
}

// --- Setup stage: reason routing ---

#[test]
fn test_dispatch_setup_start() {
    let (decision, reason) = run_spy(GameStage::Setup("start".to_string()), None, None);
    assert_eq!(reason.as_deref(), Some("spy:setup_start"), "decision={:?}", decision);
}

#[test]
fn test_dispatch_setup_after_goal() {
    let (_, reason) = run_spy(GameStage::Setup("after_goal".to_string()), None, None);
    assert_eq!(reason.as_deref(), Some("spy:setup_after_goal"));
}

#[test]
fn test_dispatch_setup_unknown_reason_falls_back_to_default() {
    // Unknown reason → default_get_setup_position → Run (no spy tag)
    let script = load_test_script("dispatch_spy.lua");
    let mut game = create_test_game_with_full_preambles_and_stage(
        &script,
        GameStage::Setup("throw_in".to_string()),
    );
    request_decisions_for_all(&mut game);
    let decision_maker = ScriptedDecisionMaker::new(&game).expect("ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert!(
        matches!(state.current_decision, Some(Decision::Run(_))),
        "Expected Run from fallback, got: {:?}",
        state.current_decision
    );
    // reason is not "spy:*" — default handler was used
    assert!(
        state.decision_reason.as_deref().map_or(true, |r| !r.starts_with("spy:")),
        "Expected fallback (non-spy) reason, got: {:?}",
        state.decision_reason
    );
}

// --- Priority: player_play overrides team_play ---

#[test]
fn test_dispatch_player_play_overrides_team_play() {
    let player_script = format!(
        "{}\nplayer_play = {{ opponent_has_ball = function() return {{action = \"stop\", reason = \"player:override\"}} end }}",
        load_test_script("dispatch_spy.lua")
    );
    let mut game = create_test_game_with_full_preambles_and_stage(&player_script, GameStage::Play);
    // Ball owned by Team B → opponent_has_ball state
    game.state.ball_state.last_possessing_team = Some(Team::B);
    request_decisions_for_all(&mut game);
    let decision_maker = ScriptedDecisionMaker::new(&game).expect("ScriptedDecisionMaker");
    let mut decision_system = DecisionSystem::new().with_decision_maker(Box::new(decision_maker));
    decision_system.update(&mut game, 1.0);

    let state = &game.state().player_states[0];
    assert!(state.last_error.is_none(), "{:?}", state.last_error);
    assert_eq!(state.decision_reason.as_deref(), Some("player:override"));
}
