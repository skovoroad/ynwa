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
use ynwa_core::game::{BallDef, Game, GameConfig, GameStage, PlayerDef, RefereeDef, ScriptingConfig};
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

            let start = parse(&p.tactical.start_position)?;
            let attack = parse(&p.tactical.attack_position)?;
            let defence = parse(&p.tactical.defence_position)?;

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
            regions.insert("start position".to_string(), flip(start)?);
            regions.insert("attack position".to_string(), flip(attack)?);
            regions.insert("defence position".to_string(), flip(defence)?);

            let optional_regions: &[(&str, &Option<String>)] = &[
                ("goal kick own position", &p.tactical.goal_kick_own_position),
                ("goal kick opp position", &p.tactical.goal_kick_opp_position),
                ("corner own left",        &p.tactical.corner_own_left),
                ("corner own right",       &p.tactical.corner_own_right),
                ("corner opp left",        &p.tactical.corner_opp_left),
                ("corner opp right",       &p.tactical.corner_opp_right),
            ];
            for (key, maybe_pos) in optional_regions {
                if let Some(pos) = maybe_pos {
                    regions.insert(key.to_string(), flip(parse(pos)?)?);
                }
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
            .with_shot_accuracy(p.static_data.shot_accuracy);

            Ok(def)
        })
        .collect()
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

    let mut players = build_player_defs(Team::A, &team_a_record, grid_dims)?;
    players.extend(build_player_defs(Team::B, &team_b_record, grid_dims)?);

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
                std::collections::HashMap::from([("start position".to_string(), start_region)]),
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
                std::collections::HashMap::from([("start position".to_string(), start_region)]),
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
            start_position:         "A1".to_string(),
            attack_position:        "A1".to_string(),
            defence_position:       "A1".to_string(),
            goal_kick_own_position: Some("B2".to_string()),
            goal_kick_opp_position: None,
            corner_own_left:        Some("C3".to_string()),
            corner_own_right:       None,
            corner_opp_left:        None,
            corner_opp_right:       None,
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
        assert!(defs[0].regions.contains_key("goal kick own position"), "goal kick own position must be set");
        assert!(!defs[0].regions.contains_key("goal kick opp position"), "goal kick opp position must be absent");
        assert!(defs[0].regions.contains_key("corner own left"), "corner own left must be set");
        assert!(!defs[0].regions.contains_key("corner own right"), "corner own right must be absent");
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
            start_position:         "A1".to_string(),
            attack_position:        "A1".to_string(),
            defence_position:       "A1".to_string(),
            goal_kick_own_position: Some("C5".to_string()),
            goal_kick_opp_position: None,
            corner_own_left: None, corner_own_right: None,
            corner_opp_left: None, corner_opp_right: None,
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

        let ra = &defs_a[0].regions["goal kick own position"];
        let rb = &defs_b[0].regions["goal kick own position"];

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

        assert!((b_min_x - (field_width  - a_max_x)).abs() < 0.01, "b_min_x={b_min_x} expected {}", field_width  - a_max_x);
        assert!((b_max_x - (field_width  - a_min_x)).abs() < 0.01, "b_max_x={b_max_x} expected {}", field_width  - a_min_x);
        assert!((b_min_z - (field_length - a_max_z)).abs() < 0.01, "b_min_z={b_min_z} expected {}", field_length - a_max_z);
        assert!((b_max_z - (field_length - a_min_z)).abs() < 0.01, "b_max_z={b_max_z} expected {}", field_length - a_min_z);
    }
}
