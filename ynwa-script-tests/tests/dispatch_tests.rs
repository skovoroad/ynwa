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

// --- Setup stage ---

// Setup decisions are assigned by FootballGameManager, not by Lua scripts.
// ScriptedDecisionMaker returns an error if called during Setup (see scripted_decision_maker.rs).

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
