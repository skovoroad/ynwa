//! YNWA Football - Football rules and world factory.
//!
//! This crate implements football-specific rules on top of `ynwa-core`:
//! - `field_builder` - standard football field with FIFA regulation zones
//! - `game_manager` - `FootballGameManager` system: stage transitions, player readiness
//! - `events` - goal detection, out-of-bounds, game end
//!
//! Entry point: `create_football_world` creates a ready-to-use `World` from the team repository.

pub mod events;
pub mod field_builder;
pub mod game_manager;

use field_builder::create_football_field;
use game_manager::FootballGameManager;
use ynwa_core::field::zones::ZoneGeometry;
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef, ScriptingConfig, REGION_START_POSITION};
use ynwa_core::region::Region;
use ynwa_core::repository::{TeamRecord, TeamRepository};
use ynwa_core::systems::decision::ScriptedDecisionMaker;
use ynwa_core::systems::{
    ActionSystem, BallPossessionSystem, DecisionSystem, PhysicsSystem, PlayerReactionSystem,
};
use ynwa_core::team::Team;
use ynwa_core::world::World;

fn get_ball_initial_position(field: &ynwa_core::field::Field) -> ynwa_core::field::zones::Point3D {
    let center_spot_zone = field
        .get_zone("center_spot", None)
        .expect("Football field must have center_spot zone");

    match &center_spot_zone.geometry {
        ZoneGeometry::Point(point) => point.position,
        _ => panic!("center_spot must be a Point zone"),
    }
}

fn build_player_defs(
    team: Team,
    record: &TeamRecord,
    grid_dims: ynwa_core::region::GridDimensions,
) -> Result<Vec<PlayerDef>, String> {
    record
        .players
        .iter()
        .map(|p| {
            let parse = |s: &str| {
                Region::from_grid_notation(s, grid_dims)
                    .map_err(|e| format!("Invalid position '{}': {}", s, e))
            };

            // Team B positions are in own orientation; flip to absolute field coordinates.
            let flip = |r: Region| {
                if team == Team::B {
                    r.flip_orientation(grid_dims)
                        .map_err(|e| format!("Flip error: {}", e))
                } else {
                    Ok(r)
                }
            };

            let script = p.script.clone().unwrap_or_default();

            let mut regions = std::collections::HashMap::new();
            let mut set_piece_roles = std::collections::HashSet::new();

            for (key, pos) in &p.tactical.play_positions {
                regions.insert(key.clone(), flip(parse(pos)?)?);
            }

            for (key, pos) in &p.tactical.set_piece_positions {
                if pos == "on_ball" {
                    set_piece_roles.insert(key.clone());
                    continue;
                }
                let region = flip(parse(pos)?)?;
                if key == "kick off" {
                    // "kick off" doubles as the start position used by core for initial placement.
                    regions.insert(REGION_START_POSITION.to_string(), region.clone());
                }
                regions.insert(key.clone(), region);
            }

            let def = PlayerDef::new(
                team,
                p.tactical.number,
                p.static_data.name.clone(),
                script,
                regions,
            )
            .with_reaction_rate(p.static_data.reaction_rate)
            .with_speed_rate(p.static_data.speed_rate)
            .with_tackle_rate(p.static_data.tackle_rate)
            .with_shot_power(p.static_data.shot_power)
            .with_shot_accuracy(p.static_data.shot_accuracy)
            .with_set_piece_roles(set_piece_roles);

            Ok(def)
        })
        .collect()
}

/// Validates that for every set-piece key present in any player's `set_piece_positions`,
/// exactly one player per team has the value `"on_ball"` for that key.
fn validate_on_ball(team_record: &TeamRecord, team_label: &str) -> Result<(), String> {
    let mut counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for player in &team_record.players {
        for (key, val) in &player.tactical.set_piece_positions {
            if val == "on_ball" {
                *counts.entry(key.as_str()).or_insert(0) += 1;
            }
        }
    }
    // Collect all distinct keys across all players to detect missing on_ball.
    let all_keys: std::collections::HashSet<&str> = team_record.players.iter()
        .flat_map(|p| p.tactical.set_piece_positions.keys().map(|k| k.as_str()))
        .collect();
    for key in all_keys {
        let count = counts.get(key).copied().unwrap_or(0);
        if count != 1 {
            return Err(format!(
                "team {}: set-piece '{}' has {} on_ball players, expected exactly 1",
                team_label, key, count
            ));
        }
    }
    Ok(())
}

fn add_football_systems(world: &mut World) {
    world.add_system(Box::new(FootballGameManager::new()));
    world.add_system(Box::new(PlayerReactionSystem));
    world.add_system(Box::new(BallPossessionSystem::new()));

    let decision_system = match ScriptedDecisionMaker::new(world.game()) {
        Ok(scripted_maker) => {
            println!(
                "Successfully initialized ScriptedDecisionMaker for {} players",
                world.game().config().players.len()
            );
            DecisionSystem::new().with_decision_maker(Box::new(scripted_maker))
        }
        Err(e) => {
            eprintln!(
                "Warning: Failed to create ScriptedDecisionMaker: {}. Using placeholder.",
                e
            );
            DecisionSystem::new()
        }
    };

    world.add_system(Box::new(decision_system));
    world.add_system(Box::new(ActionSystem::new()));
    world.add_system(Box::new(PhysicsSystem::new()));
}

/// Creates a football world from team repository.
///
/// `repo` supplies both teams (`"team_a"` and `"team_b"`) and their preambles.
/// `preambles_path` - directory containing `core.lua` and `stdlib.lua`.
pub fn create_football_world(
    repo: &dyn TeamRepository,
    preambles_path: &std::path::Path,
) -> Result<World, String> {
    let field = create_football_field();
    let grid_dims = field.grid_dimensions();

    let team_a_record = repo.load_team("team_a")?;
    let team_b_record = repo.load_team("team_b")?;

    validate_on_ball(&team_a_record, "team_a")?;
    validate_on_ball(&team_b_record, "team_b")?;

    let team_a_players = build_player_defs(Team::A, &team_a_record, grid_dims)?;
    let team_b_players = build_player_defs(Team::B, &team_b_record, grid_dims)?;

    let mut players = team_a_players;
    players.extend(team_b_players);

    let load_preamble = |name: &str| {
        let path = preambles_path.join(name);
        std::fs::read_to_string(&path)
            .map_err(|e| format!("Failed to read preamble '{}': {}", path.display(), e))
    };

    let game_config = GameConfig {
        field: field.clone(),
        players,
        ball: BallDef {
            initial_position: get_ball_initial_position(&field),
        },
        referees: vec![RefereeDef::default()],
        scripting: ScriptingConfig {
            core_preamble: load_preamble("core.lua")?,
            stdlib_preamble: load_preamble("stdlib.lua")?,
            team_a_preamble: team_a_record.preamble,
            team_b_preamble: team_b_record.preamble,
        },
    };

    let game = Game::with_stage(game_config, GameStage::Setup("start".to_string()));
    let mut world = World::new(game);
    add_football_systems(&mut world);
    Ok(world)
}

#[cfg(test)]
    #[path = "tests/field_builder_tests.rs"]
    mod field_builder_tests;#[cfg(test)]
mod tests {
    use super::*;
    use uom::si::length::meter;
    use ynwa_core::region::*;
    use ynwa_core::team::Team;

    fn create_football_game_config() -> GameConfig {
        let field = create_football_field();
        let grid_dims = field.grid_dimensions();
        let ball_initial_position = get_ball_initial_position(&field);

        let mut players = Vec::new();
        for i in 0..11 {
            let row = i + 1;
            let start_region = grid_dims
                .create_region(
                    GridCell::new(1, row).unwrap(),
                    GridCell::new(2, row).unwrap(),
                )
                .unwrap();

            players.push(PlayerDef::new(
                Team::A,
                i + 1,
                format!("Player A{}", i + 1),
                "function make_decision() return {} end".to_string(),
                std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            ));
        }
        for i in 0..11 {
            let row = i + 1;
            let start_region = grid_dims
                .create_region(
                    GridCell::new(25, row).unwrap(),
                    GridCell::new(26, row).unwrap(),
                )
                .unwrap();

            players.push(PlayerDef::new(
                Team::B,
                i + 1,
                format!("Player B{}", i + 1),
                "function make_decision() return {} end".to_string(),
                std::collections::HashMap::from([(REGION_START_POSITION.to_string(), start_region)]),
            ));
        }

        GameConfig {
            field,
            players,
            ball: BallDef {
                initial_position: ball_initial_position,
            },
            referees: vec![RefereeDef::default()],
            scripting: ynwa_core::game::ScriptingConfig::empty(),
        }
    }

    pub fn create_test_world() -> World {
        let game_config = create_football_game_config();
        let game = Game::with_stage(game_config, GameStage::Setup("Prepare".to_string()));
        let mut world = World::new(game);
        add_football_systems(&mut world);
        world
    }

    #[test]
    fn test_create_football_world() {
        let world = create_test_world();

        assert_eq!(world.game().config().players.len(), 22);
        assert_eq!(world.game().state().elapsed_time, 0.0);
    }

    #[test]
    fn test_ball_initial_position_at_center_spot() {
        let world = create_test_world();
        let game = world.game();

        let center_spot = game
            .config()
            .field
            .get_zone("center_spot", None)
            .expect("Football field must have center_spot");

        let expected_position = match &center_spot.geometry {
            ynwa_core::field::zones::ZoneGeometry::Point(point) => &point.position,
            _ => panic!("center_spot must be a Point zone"),
        };

        let ball_position = &game.state().ball_state.position;
        assert_eq!(ball_position.x.get::<meter>(), expected_position.x.get::<meter>());
        assert_eq!(ball_position.y.get::<meter>(), expected_position.y.get::<meter>());
        assert_eq!(ball_position.z.get::<meter>(), expected_position.z.get::<meter>());
    }

    #[test]
    fn test_create_football_world_from_repository() {
        let teams_path = std::path::Path::new("../teams");
        let preambles_path = std::path::Path::new("../ynwa-scripts/preambles");
        if !teams_path.exists() {
            println!("Skipping test - teams directory not found");
            return;
        }

        let repo = ynwa_repository::FsTeamRepository::new(teams_path);
        let world = create_football_world(&repo, preambles_path)
            .expect("Failed to create world from repository");

        assert_eq!(world.game().config().players.len(), 22);
    }

    #[test]
    fn build_player_defs_optional_regions_team_a() {
        use ynwa_core::repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord};

        let field = create_football_field();
        let grid_dims = field.grid_dimensions();

        let tactical = PlayerTactical {
            number: 1,
            play_positions: [
                ("attack".to_string(),  "A1".to_string()),
                ("defence".to_string(), "A1".to_string()),
            ].into(),
            set_piece_positions: [
                ("kick off".to_string(),        "A1".to_string()),
                ("goal kick own".to_string(),   "B2".to_string()),
                ("corner own left".to_string(), "C3".to_string()),
            ].into(),
        };
        let record = TeamRecord {
            preamble: String::new(),
            players: vec![PlayerRecord {
                static_data: PlayerStatic { name: "P".to_string(), reaction_rate: 50,
                    speed_rate: 50, tackle_rate: 50, shot_power: 50, shot_accuracy: 50 },
                tactical,
                script: None,
            }],
        };

        let defs = build_player_defs(Team::A, &record, grid_dims).unwrap();

        // play_positions: keys inserted as-is ("attack", "defence")
        assert!(defs[0].regions.contains_key("attack"));
        assert!(defs[0].regions.contains_key("defence"));

        // set_piece_positions: present keys are registered as-is
        assert!(defs[0].regions.contains_key("kick off"));
        assert!(defs[0].regions.contains_key("goal kick own"));
        assert!(defs[0].regions.contains_key("corner own left"));

        // "kick off" also registers REGION_START_POSITION for core
        assert!(defs[0].regions.contains_key(REGION_START_POSITION));

        // absent keys produce no region
        assert!(!defs[0].regions.contains_key("goal kick opp"));
        assert!(!defs[0].regions.contains_key("corner own right"));

        // set_piece_roles is empty when no "on_ball" values present
        assert!(defs[0].set_piece_roles.is_empty());
    }

    #[test]
    fn build_player_defs_on_ball_goes_to_roles_not_regions() {
        use ynwa_core::repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord};

        let field = create_football_field();
        let grid_dims = field.grid_dimensions();

        let tactical = PlayerTactical {
            number: 9,
            play_positions: [("attack".to_string(), "A1".to_string())].into(),
            set_piece_positions: [
                ("kick off".to_string(),      "A1".to_string()),
                ("goal kick own".to_string(), "on_ball".to_string()),
                ("corner own left".to_string(), "B2".to_string()),
            ].into(),
        };
        let record = TeamRecord {
            preamble: String::new(),
            players: vec![PlayerRecord {
                static_data: PlayerStatic { name: "P".to_string(), reaction_rate: 50,
                    speed_rate: 50, tackle_rate: 50, shot_power: 50, shot_accuracy: 50 },
                tactical,
                script: None,
            }],
        };

        let defs = build_player_defs(Team::A, &record, grid_dims).unwrap();

        // "on_ball" value → key in set_piece_roles, not in regions
        assert!(defs[0].set_piece_roles.contains("goal kick own"));
        assert!(!defs[0].regions.contains_key("goal kick own"));

        // other set_piece entries still go to regions
        assert!(defs[0].regions.contains_key("corner own left"));
        assert!(defs[0].set_piece_roles.is_empty() || !defs[0].set_piece_roles.contains("corner own left"));
    }

    #[test]
    fn build_player_defs_optional_regions_flipped_for_team_b() {
        use ynwa_core::repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord};
        use uom::si::length::meter;

        let field = create_football_field();
        let grid_dims = field.grid_dimensions();
        let field_width  = field.width().get::<meter>();
        let field_length = field.length().get::<meter>();

        let tactical = PlayerTactical {
            number: 1,
            play_positions: [
                ("attack".to_string(),  "A1".to_string()),
                ("defence".to_string(), "A1".to_string()),
            ].into(),
            set_piece_positions: [
                ("kick off".to_string(),        "A1".to_string()),
                ("goal kick own".to_string(),   "C5".to_string()),
            ].into(),
        };
        let record = TeamRecord {
            preamble: String::new(),
            players: vec![PlayerRecord {
                static_data: PlayerStatic { name: "P".to_string(), reaction_rate: 50,
                    speed_rate: 50, tackle_rate: 50, shot_power: 50, shot_accuracy: 50 },
                tactical,
                script: None,
            }],
        };

        let defs_a = build_player_defs(Team::A, &record, grid_dims).unwrap();
        let defs_b = build_player_defs(Team::B, &record, grid_dims).unwrap();

        // play_positions are present in both
        assert!(defs_a[0].regions.contains_key("attack"));
        assert!(defs_a[0].regions.contains_key("defence"));
        assert!(defs_b[0].regions.contains_key("attack"));
        assert!(defs_b[0].regions.contains_key("defence"));

        let ra = &defs_a[0].regions["goal kick own"];
        let rb = &defs_b[0].regions["goal kick own"];

        // flip_orientation swaps: min_x ↔ (field_width - max_x), min_z ↔ (field_length - max_z)
        let cell_size = field_width / grid_dims.columns as f32;
        let a_min_x = (ra.top_left.col     - 1) as f32 * cell_size;
        let a_max_x =  ra.bottom_right.col      as f32 * cell_size;
        let a_min_z = (ra.top_left.row     - 1) as f32 * cell_size;
        let a_max_z =  ra.bottom_right.row      as f32 * cell_size;
        let b_min_x = (rb.top_left.col     - 1) as f32 * cell_size;
        let b_max_x =  rb.bottom_right.col      as f32 * cell_size;
        let b_min_z = (rb.top_left.row     - 1) as f32 * cell_size;
        let b_max_z =  rb.bottom_right.row      as f32 * cell_size;

        assert!((b_min_x - (field_width  - a_max_x)).abs() < 0.01);
        assert!((b_max_x - (field_width  - a_min_x)).abs() < 0.01);
        assert!((b_min_z - (field_length - a_max_z)).abs() < 0.01);
        assert!((b_max_z - (field_length - a_min_z)).abs() < 0.01);
    }

    fn make_team_record(players_positions: &[&[(&str, &str)]]) -> TeamRecord {
        use ynwa_core::repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord};
        TeamRecord {
            preamble: String::new(),
            players: players_positions.iter().enumerate().map(|(i, positions)| {
                PlayerRecord {
                    static_data: PlayerStatic { name: "P".to_string(), reaction_rate: 50,
                        speed_rate: 50, tackle_rate: 50, shot_power: 50, shot_accuracy: 50 },
                    tactical: PlayerTactical {
                        number: i as u32 + 1,
                        play_positions: Default::default(),
                        set_piece_positions: positions.iter()
                            .map(|(k, v)| (k.to_string(), v.to_string()))
                            .collect(),
                    },
                    script: None,
                }
            }).collect(),
        }
    }

    #[test]
    fn validate_on_ball_ok() {
        let record = make_team_record(&[
            &[("kick off", "on_ball"), ("goal kick own", "A1")],
            &[("kick off", "A1"),     ("goal kick own", "on_ball")],
        ]);
        assert!(validate_on_ball(&record, "team_a").is_ok());
    }

    #[test]
    fn validate_on_ball_duplicate() {
        let record = make_team_record(&[
            &[("kick off", "on_ball")],
            &[("kick off", "on_ball")],
        ]);
        let err = validate_on_ball(&record, "team_a").unwrap_err();
        assert!(err.contains("kick off"), "error should mention the key: {}", err);
        assert!(err.contains("team_a"), "error should mention the team: {}", err);
    }

    #[test]
    fn validate_on_ball_missing() {
        let record = make_team_record(&[
            &[("kick off", "A1")],
            &[("kick off", "A1")],
        ]);
        let err = validate_on_ball(&record, "team_a").unwrap_err();
        assert!(err.contains("kick off"), "error should mention the key: {}", err);
    }
}
