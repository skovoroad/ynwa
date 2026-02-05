// Integration test: verify that Lua scripts produce decisions in the decision system

use ynwa_core::game::Decision;
use ynwa_core::systems::decision::{DecisionSystem, ScriptedDecisionMaker};
use ynwa_core::systems::player_reaction::PlayerReactionSystem;
use ynwa_core::System;
use ynwa_script_tests::{create_test_game_with_script, load_script};

#[test]
fn test_simple_stop_script_produces_stop_decision() {
    // Load script from ynwa-scripts library
    let script = load_script("test-scripts/simple_stop.lua");
    
    // Create game with this script
    let mut game = create_test_game_with_script(&script);
    
    // Use PlayerReactionSystem to set needs_decision flag
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0); // 1 second should trigger reaction
    
    // Create decision system with JSON decision maker
    let decision_maker = ScriptedDecisionMaker::new(&game)
        .expect("Failed to create ScriptedDecisionMaker");
    
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(decision_maker));
    
    // Execute decision system
    decision_system.update(&mut game, 1.0);
    
    // Verify that player has stop decision
    let player_state = &game.state().player_states[0];
    assert!(player_state.current_decision.is_some(), "Player should have a decision");
    
    match player_state.current_decision.as_ref().unwrap() {
        Decision::Stop => {
            // Expected - test passed
        }
        _ => panic!("Expected Stop decision, got {:?}", player_state.current_decision),
    }
}

#[test]
fn test_run_to_cell_script_produces_run_decision() {
    // Load script from ynwa-scripts library
    let script = load_script("test-scripts/run_to_cell.lua");
    
    // Create game with this script
    let mut game = create_test_game_with_script(&script);
    
    // Use PlayerReactionSystem to set needs_decision flag
    let mut reaction_system = PlayerReactionSystem::new();
    reaction_system.update(&mut game, 1.0); // 1 second should trigger reaction
    
    // Create decision system with JSON decision maker
    let decision_maker = ScriptedDecisionMaker::new(&game)
        .expect("Failed to create ScriptedDecisionMaker");
    
    let mut decision_system = DecisionSystem::new()
        .with_decision_maker(Box::new(decision_maker));
    
    // Execute decision system
    decision_system.update(&mut game, 1.0);
    
    // Verify that player has run decision
    let player_state = &game.state().player_states[0];
    assert!(player_state.current_decision.is_some(), "Player should have a decision");
    
    match player_state.current_decision.as_ref().unwrap() {
        Decision::Run(_) => {
            // Expected - we got a Run decision, test passed
        }
        Decision::Stop => panic!("Expected Run decision, got Stop"),
    }
}
