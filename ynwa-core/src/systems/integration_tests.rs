#[cfg(test)]
mod tests {
    use crate::field::zones::Velocity3D;
    use crate::field::Field;
    use crate::game::{BallDef, Decision, DecisionTarget, Game, GameConfig, PlayerDef};
    use crate::physics_util::distance;
    use crate::region::{GridCell, Region};
    use crate::systems::decision::DecisionError;
    use crate::systems::{
        ActionSystem, DecisionMaker, DecisionSystem, PhysicsSystem, PlayerReactionSystem,
    };
    use crate::team::Team;
    use crate::world::World;
    use std::collections::HashMap;
    use uom::si::length::meter;

    /// Test DecisionMaker that returns pre-scripted decisions
    struct ScriptedDecisionMaker {
        // player_index -> list of decisions
        decisions: HashMap<usize, Vec<Decision>>,
        // player_index -> current decision index
        decision_counters: HashMap<usize, usize>,
    }

    impl ScriptedDecisionMaker {
        fn new(decisions: Vec<(usize, Vec<Decision>)>) -> Self {
            let mut decisions_map = HashMap::new();
            let mut counters = HashMap::new();

            for (player_index, decision_list) in decisions {
                decisions_map.insert(player_index, decision_list);
                counters.insert(player_index, 0);
            }

            Self {
                decisions: decisions_map,
                decision_counters: counters,
            }
        }
    }

    impl DecisionMaker for ScriptedDecisionMaker {
        fn make_decision(
            &mut self,
            _game: &Game,
            player_index: usize,
        ) -> Result<Decision, DecisionError> {
            if let Some(decisions) = self.decisions.get(&player_index) {
                if let Some(counter) = self.decision_counters.get_mut(&player_index) {
                    let decision_index = *counter;
                    if decision_index < decisions.len() {
                        *counter += 1;
                        return Ok(decisions[decision_index].clone());
                    }
                }
            }
            // Return Stop if no more decisions available
            Ok(Decision::Stop)
        }
    }

    /// Creates a test game with two players with different characteristics
    fn create_test_game() -> Game {
        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        // Player 0: fast and reactive
        let start_region_0 = Region::new(
            Team::A,
            GridCell::new(1, 1).unwrap(),
            GridCell::new(1, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let player_0 = PlayerDef::new(
            Team::A,
            1,
            "Fast Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region_0,
        )
        .with_reaction_rate(100) // reaction_rate = 100 (1 second interval)
        .with_speed_rate(100); // speed_rate = 100 (full speed)

        // Player 1: slow and less reactive
        let start_region_1 = Region::new(
            Team::A,
            GridCell::new(5, 1).unwrap(),
            GridCell::new(5, 1).unwrap(),
            grid_dims,
        )
        .unwrap();

        let player_1 = PlayerDef::new(
            Team::A,
            2,
            "Slow Player".to_string(),
            "function make_decision() return {} end".to_string(),
            start_region_1,
        )
        .with_reaction_rate(50) // reaction_rate = 50 (2 second interval)
        .with_speed_rate(50); // speed_rate = 50 (half speed)

        let config = GameConfig {
            field,
            players: vec![player_0, player_1],
            ball: BallDef::default(),
            referees: vec![],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        Game::with_stage(config, crate::game::GameStage::Play)
    }

    #[test]
    fn test_full_game_simulation_pipeline() {
        // 1. Create game with two players
        let game = create_test_game();

        // Save initial positions for comparison
        let initial_pos_0 = game.state.player_states[0].position;
        let initial_pos_1 = game.state.player_states[1].position;

        println!(
            "Initial position player 0: ({:.2}, {:.2}, {:.2})",
            initial_pos_0.x.get::<meter>(),
            initial_pos_0.y.get::<meter>(),
            initial_pos_0.z.get::<meter>()
        );
        println!(
            "Initial position player 1: ({:.2}, {:.2}, {:.2})",
            initial_pos_1.x.get::<meter>(),
            initial_pos_1.y.get::<meter>(),
            initial_pos_1.z.get::<meter>()
        );

        // 2. Create scripted DecisionMaker
        // Player 0 will run to a specific cell, then stop
        // Player 1 will run to a specific region, then stop
        let grid_dims = game.config().field.grid_dimensions();

        let target_region = Region::new(
            Team::A,
            GridCell::new(15, 15).unwrap(),
            GridCell::new(20, 20).unwrap(),
            grid_dims,
        )
        .unwrap();

        let decision_maker = ScriptedDecisionMaker::new(vec![
            (
                0,
                vec![
                    Decision::Run(DecisionTarget::GridCell(GridCell::new(10, 10).unwrap())),
                    Decision::Stop,
                ],
            ),
            (
                1,
                vec![
                    Decision::Run(DecisionTarget::Region(target_region)),
                    Decision::Stop,
                ],
            ),
        ]);

        // 3. Create World with all systems
        let mut world = World::new(game);
        world.add_system(Box::new(PlayerReactionSystem::new()));
        world.add_system(Box::new(
            DecisionSystem::new().with_decision_maker(Box::new(decision_maker)),
        ));
        world.add_system(Box::new(ActionSystem::new()));
        world.add_system(Box::new(PhysicsSystem::new()));

        // 4. t=0.1 - first tick
        world.step(0.1);

        // Checks after first tick:
        // - Both players received decisions
        assert!(world.game().state.player_states[0].decision_processed);
        assert!(world.game().state.player_states[1].decision_processed);

        // - Different decision types: GridCell vs Region
        assert!(matches!(
            world.game().state.player_states[0].current_decision,
            Some(Decision::Run(DecisionTarget::GridCell(_)))
        ));
        assert!(matches!(
            world.game().state.player_states[1].current_decision,
            Some(Decision::Run(DecisionTarget::Region(_)))
        ));

        // - Both players started moving
        assert_ne!(
            world.game().state.player_states[0].velocity,
            Velocity3D::zero()
        );
        assert_ne!(
            world.game().state.player_states[1].velocity,
            Velocity3D::zero()
        );

        // - Positions changed (PhysicsSystem applied velocity with delta=0.1)
        let player_0_moved = distance(
            &world.game().state.player_states[0].position,
            &initial_pos_0,
        ) > 0.0;
        let player_1_moved = distance(
            &world.game().state.player_states[1].position,
            &initial_pos_1,
        ) > 0.0;
        assert!(player_0_moved);
        assert!(player_1_moved);

        // 5. t=0.2 - second tick (movement continues)
        world.step(0.1);

        // Positions changed
        let player_0_moved = distance(
            &world.game().state.player_states[0].position,
            &initial_pos_0,
        ) > 0.0;
        let player_1_moved = distance(
            &world.game().state.player_states[1].position,
            &initial_pos_1,
        ) > 0.0;
        assert!(player_0_moved);
        assert!(player_1_moved);

        // Save positions for later checks
        let player_0_pos_at_0_2 = world.game().state.player_states[0].position;
        let player_1_pos_at_0_2 = world.game().state.player_states[1].position;

        // 6. t=0.3 to t=0.9 - continue movement
        for _ in 3..10 {
            world.step(0.1);
        }

        // Players traveled significant distance
        let player_0_distance_traveled = distance(
            &player_0_pos_at_0_2,
            &world.game().state.player_states[0].position,
        );
        let player_1_distance_traveled = distance(
            &player_1_pos_at_0_2,
            &world.game().state.player_states[1].position,
        );

        // Both moved
        assert!(player_0_distance_traveled > 0.0);
        assert!(player_1_distance_traveled > 0.0);

        println!(
            "From t=0.2 to t=0.9: player 0 traveled {:.2} m, player 1 traveled {:.2} m",
            player_0_distance_traveled, player_1_distance_traveled
        );

        // Player 0 is faster (speed_rate=100 vs 50), but distance depends on target points
        // Main thing - both moved

        // 7. t=1.0 - player 0 receives second decision Stop (reaction_rate=100)
        world.step(0.1);

        // Player 0 received new decision and stopped
        assert_eq!(
            world.game().state.player_states[0].velocity,
            Velocity3D::zero()
        );
        assert!(matches!(
            world.game().state.player_states[0].current_decision,
            Some(Decision::Stop)
        ));

        // Player 1 still moving (reaction_rate=50, not time yet)
        assert_ne!(
            world.game().state.player_states[1].velocity,
            Velocity3D::zero()
        );

        // 8. t=1.1 to t=1.9 - player 0 stands still, player 1 moves
        let player_0_pos_at_1_0 = world.game().state.player_states[0].position;
        let player_1_pos_at_1_0 = world.game().state.player_states[1].position;

        for _ in 11..20 {
            world.step(0.1);
        }

        let player_0_pos_at_1_9 = world.game().state.player_states[0].position;
        let player_1_pos_at_1_9 = world.game().state.player_states[1].position;

        // Player 0 barely moves (standing still)
        let player_0_stopped_distance = distance(&player_0_pos_at_1_0, &player_0_pos_at_1_9);

        // Player 1 continues moving
        let player_1_moving_distance = distance(&player_1_pos_at_1_0, &player_1_pos_at_1_9);

        // Player 1 traveled much more than stopped player 0
        assert!(player_1_moving_distance > player_0_stopped_distance * 5.0);

        // 9. t=2.0 - player 1 receives second decision Stop (reaction_rate=50, 2 seconds passed)
        world.step(0.1);

        // Both players stopped
        assert_eq!(
            world.game().state.player_states[0].velocity,
            Velocity3D::zero()
        );
        assert_eq!(
            world.game().state.player_states[1].velocity,
            Velocity3D::zero()
        );

        // Both have Stop decision
        assert!(matches!(
            world.game().state.player_states[0].current_decision,
            Some(Decision::Stop)
        ));
        assert!(matches!(
            world.game().state.player_states[1].current_decision,
            Some(Decision::Stop)
        ));

        // 10. t=3.0 - final check (both standing)
        let final_pos_0 = world.game().state.player_states[0].position;
        let final_pos_1 = world.game().state.player_states[1].position;

        world.step(1.0);

        // Positions didn't change (both standing)
        assert_eq!(world.game().state.player_states[0].position, final_pos_0);
        assert_eq!(world.game().state.player_states[1].position, final_pos_1);

        // Final check: player 0 traveled more total distance
        // (was 2x faster, though stopped earlier)
        let total_distance_0 = distance(&initial_pos_0, &final_pos_0);
        let total_distance_1 = distance(&initial_pos_1, &final_pos_1);

        println!(
            "Player 0 (GridCell, speed=100, reaction=100) traveled: {:.2} m",
            total_distance_0
        );
        println!(
            "Player 1 (Region, speed=50, reaction=50) traveled: {:.2} m",
            total_distance_1
        );

        // Player 0 moved ~1 second at speed up to 10 m/s
        // Player 1 moved ~2 seconds at speed up to 5 m/s
        // Check that both traveled reasonable distance
        assert!(total_distance_0 > 2.0); // Player 0 traveled at least 2 meters
        assert!(total_distance_1 > 4.0); // Player 1 traveled at least 4 meters
    }

    #[test]
    fn test_lua_decision_maker_integration() {
        use crate::systems::decision::ScriptedDecisionMaker;

        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(13, 22).unwrap(), // Center of field
            GridCell::new(13, 22).unwrap(),
            grid_dims,
        )
        .unwrap();

        // Player with Lua script that runs to A1
        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(
                Team::A,
                1,
                "Lua Player".to_string(),
                r#"
                function make_decision()
                    -- Check context is available
                    local my_team = context.me.team
                    local my_number = context.me.number
                    
                    -- Always run to A1
                    return {
                        action = "run",
                        target_type = "cell",
                        target = "A1"
                    }
                end
                "#
                .to_string(),
                start_region,
            )
            .with_reaction_rate(100) // Fast reaction
            .with_speed_rate(100)], // Fast movement
            ball: BallDef::default(),
            referees: vec![],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::with_stage(config, crate::game::GameStage::Play);

        // Create ScriptedDecisionMaker
        let scripted_maker =
            ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

        // Build world with scripted decision maker
        let mut world = World::new(game);
        world.add_system(Box::new(PlayerReactionSystem));
        world.add_system(Box::new(
            DecisionSystem::new().with_decision_maker(Box::new(scripted_maker)),
        ));
        world.add_system(Box::new(ActionSystem::new()));
        world.add_system(Box::new(PhysicsSystem::new()));

        let initial_pos = world.game().state.player_states[0].position.clone();

        // Run simulation for 2 seconds
        for _ in 0..120 {
            world.step(1.0 / 60.0);
        }

        let final_pos = world.game().state.player_states[0].position.clone();

        // Player should have moved towards A1 (col=1, row=1)
        // A1 is in the corner, so both x and z should decrease
        assert!(final_pos.x.get::<meter>() < initial_pos.x.get::<meter>());
        assert!(final_pos.z.get::<meter>() < initial_pos.z.get::<meter>());

        // Player should have traveled some distance
        let traveled = distance(&initial_pos, &final_pos);
        println!("Lua player traveled: {:.2} m towards A1", traveled);
        assert!(traveled > 5.0); // Should move at least 5 meters in 2 seconds
    }

    #[test]
    fn test_lua_decision_maker_error_handling() {
        use crate::systems::decision::ScriptedDecisionMaker;

        let field = Field::from_meters(100.0, 60.0, 26, 44);
        let grid_dims = field.grid_dimensions();

        let start_region = Region::new(
            Team::A,
            GridCell::new(13, 22).unwrap(),
            GridCell::new(13, 22).unwrap(),
            grid_dims,
        )
        .unwrap();

        // Player with buggy Lua script
        let config = GameConfig {
            field,
            players: vec![PlayerDef::new(Team::A, 1, "Buggy Player".to_string(), r#"
                function make_decision()
                    error("Intentional error for testing")
                end
                "#
                .to_string(), start_region,
            )],
            ball: BallDef::default(),
            referees: vec![],
            scripting: crate::game::ScriptingConfig::empty(),
        };

        let game = Game::with_stage(config, crate::game::GameStage::Play);

        let scripted_maker =
            ScriptedDecisionMaker::new(&game).expect("Failed to create ScriptedDecisionMaker");

        let mut world = World::new(game);
        world.add_system(Box::new(PlayerReactionSystem));
        world.add_system(Box::new(
            DecisionSystem::new().with_decision_maker(Box::new(scripted_maker)),
        )); // Run simulation - should not crash despite error
        for _ in 0..60 {
            world.step(1.0 / 60.0);
        }

        // Player should have error message saved
        let player_state = &world.game().state.player_states[0];
        assert!(player_state.last_error.is_some());

        let error_msg = player_state.last_error.as_ref().unwrap();
        assert!(error_msg.contains("Intentional error"));

        // Player should not have a decision (error returned None)
        assert!(player_state.current_decision.is_none());
    }
}
