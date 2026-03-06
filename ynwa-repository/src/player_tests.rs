#[cfg(test)]
mod tests {
    use std::fs;
    use crate::FsTeamRepository;
    use ynwa_core::repository::TeamRepository;

    fn repo() -> FsTeamRepository {
        // Canonical test data lives in teams/ at the workspace root.
        let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("teams");
        FsTeamRepository::new(base)
    }

    // Grid notation: one or two uppercase letters followed by digits, e.g. "N3", "AA12".
    fn is_valid_grid_notation(s: &str) -> bool {
        let split = s.find(|c: char| c.is_ascii_digit()).unwrap_or(s.len());
        let (letters, digits) = s.split_at(split);
        !letters.is_empty()
            && letters.chars().all(|c| c.is_ascii_alphabetic())
            && !digits.is_empty()
            && digits.chars().all(|c| c.is_ascii_digit())
    }

    #[test]
    fn loads_both_teams() {
        let repo = repo();
        for team_id in ["team_a", "team_b"] {
            let team = repo.load_team(team_id).expect(team_id);
            assert!(!team.players.is_empty(), "{team_id}: no players");
            assert!(!team.preamble.is_empty(), "{team_id}: empty preamble");
        }
    }

    #[test]
    fn player_fields_are_valid() {
        let repo = repo();
        for team_id in ["team_a", "team_b"] {
            let team = repo.load_team(team_id).expect(team_id);
            for p in &team.players {
                let s = &p.static_data;
                let t = &p.tactical;
                assert!(!s.name.is_empty(), "{team_id}: empty name");
                for (label, val) in [
                    ("reaction_rate", s.reaction_rate),
                    ("speed_rate",    s.speed_rate),
                    ("tackle_rate",   s.tackle_rate),
                    ("shot_power",    s.shot_power),
                    ("shot_accuracy", s.shot_accuracy),
                ] {
                    assert!((10..=100).contains(&val), "{team_id} #{}: {label}={val} out of 10-100", t.number);
                }
                assert!((1..=99).contains(&t.number), "{team_id}: number {} out of range", t.number);
                for (label, pos) in [
                    ("start_position",   &t.start_position),
                    ("attack_position",  &t.attack_position),
                    ("defence_position", &t.defence_position),
                ] {
                    assert!(is_valid_grid_notation(pos), "{team_id} #{}: {label}={pos:?} is not valid grid notation", t.number);
                }
            }
        }
    }

    #[test]
    fn player_numbers_form_contiguous_sequence() {
        let repo = repo();
        for team_id in ["team_a", "team_b"] {
            let team = repo.load_team(team_id).expect(team_id);
            let mut numbers: Vec<u32> = team.players.iter().map(|p| p.tactical.number).collect();
            numbers.sort();
            let expected: Vec<u32> = (1..=numbers.len() as u32).collect();
            assert_eq!(numbers, expected, "{team_id}: numbers must be 1..N with no gaps");
        }
    }

    #[test]
    fn nonexistent_team_returns_error() {
        assert!(repo().load_team("team_x").is_err());
    }

    #[test]
    fn missing_players_dir_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let team_dir = tmp.path().join("t");
        fs::create_dir_all(&team_dir).unwrap();
        fs::write(team_dir.join("preamble.lua"), "-- x").unwrap();
        // no players/ directory

        assert!(FsTeamRepository::new(tmp.path()).load_team("t").is_err());
    }

    #[test]
    fn empty_players_dir_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let team_dir = tmp.path().join("t");
        fs::create_dir_all(team_dir.join("players")).unwrap();
        fs::write(team_dir.join("preamble.lua"), "-- empty").unwrap();

        assert!(FsTeamRepository::new(tmp.path()).load_team("t").is_err());
    }

    #[test]
    fn broken_static_toml_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), "not valid toml ][").unwrap();
        fs::write(player_dir.join("tactical.toml"), tactical_toml_content()).unwrap();

        assert!(FsTeamRepository::new(tmp.path()).load_team("t").is_err());
    }

    #[test]
    fn missing_tactical_toml_returns_error() {
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
        // no tactical.toml

        assert!(FsTeamRepository::new(tmp.path()).load_team("t").is_err());
    }

    #[test]
    fn missing_script_lua_yields_none() {
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
        fs::write(player_dir.join("tactical.toml"), tactical_toml_content()).unwrap();

        let team = FsTeamRepository::new(tmp.path()).load_team("t").unwrap();
        assert!(team.players[0].script.is_none());
    }

    #[test]
    fn present_script_lua_yields_some() {
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
        fs::write(player_dir.join("tactical.toml"), tactical_toml_content()).unwrap();
        fs::write(player_dir.join("script.lua"), "player_play = {}").unwrap();

        let team = FsTeamRepository::new(tmp.path()).load_team("t").unwrap();
        assert_eq!(team.players[0].script.as_deref(), Some("player_play = {}"));
    }

    // Player order is deterministic (sorted by directory name) because global player
    // indices in ynwa-core depend on position in the players array.
    #[test]
    fn players_are_sorted_by_directory_name() {
        let tmp = tempfile::tempdir().unwrap();
        // Directories created in reverse order; loaded order must follow directory name sort.
        for (dir, number) in [("03", 3u32), ("01", 1), ("02", 2)] {
            let player_dir = tmp.path().join(format!("t/players/{dir}"));
            fs::create_dir_all(&player_dir).unwrap();
            let tactical = format!(
                "number = {number}\nstart_position = \"A1\"\nattack_position = \"A1\"\ndefence_position = \"A1\""
            );
            fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
            fs::write(player_dir.join("tactical.toml"), tactical).unwrap();
        }
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();

        let team = FsTeamRepository::new(tmp.path()).load_team("t").unwrap();
        let numbers: Vec<u32> = team.players.iter().map(|p| p.tactical.number).collect();
        assert_eq!(numbers, vec![1, 2, 3]);
    }

    fn static_toml_content() -> &'static str {
        "name = \"Test Player\"\nreaction_rate = 50\nspeed_rate = 50\ntackle_rate = 50\nshot_power = 50\nshot_accuracy = 50"
    }

    fn tactical_toml_content() -> &'static str {
        "number = 1\nstart_position = \"A1\"\nattack_position = \"A1\"\ndefence_position = \"A1\""
    }

    #[test]
    fn optional_tactical_fields_default_to_none() {
        // tactical.toml without new fields must parse without error; all Option fields are None.
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
        fs::write(player_dir.join("tactical.toml"), tactical_toml_content()).unwrap();

        let team = FsTeamRepository::new(tmp.path()).load_team("t").unwrap();
        let t = &team.players[0].tactical;
        assert!(t.goal_kick_own_position.is_none());
        assert!(t.goal_kick_opp_position.is_none());
        assert!(t.corner_own_left.is_none());
        assert!(t.corner_own_right.is_none());
        assert!(t.corner_opp_left.is_none());
        assert!(t.corner_opp_right.is_none());
    }

    #[test]
    fn optional_tactical_fields_parsed_when_present() {
        let tmp = tempfile::tempdir().unwrap();
        let player_dir = tmp.path().join("t/players/01");
        fs::create_dir_all(&player_dir).unwrap();
        fs::write(tmp.path().join("t/preamble.lua"), "-- x").unwrap();
        fs::write(player_dir.join("static.toml"), static_toml_content()).unwrap();
        let tactical = r#"
number = 1
start_position = "A1"
attack_position = "A1"
defence_position = "A1"
goal_kick_own_position = "B2"
goal_kick_opp_position = "C3"
corner_own_left = "D4"
corner_own_right = "E5"
corner_opp_left = "F6"
corner_opp_right = "G7"
"#;
        fs::write(player_dir.join("tactical.toml"), tactical).unwrap();

        let team = FsTeamRepository::new(tmp.path()).load_team("t").unwrap();
        let t = &team.players[0].tactical;
        assert_eq!(t.goal_kick_own_position.as_deref(), Some("B2"));
        assert_eq!(t.goal_kick_opp_position.as_deref(), Some("C3"));
        assert_eq!(t.corner_own_left.as_deref(),        Some("D4"));
        assert_eq!(t.corner_own_right.as_deref(),       Some("E5"));
        assert_eq!(t.corner_opp_left.as_deref(),        Some("F6"));
        assert_eq!(t.corner_opp_right.as_deref(),       Some("G7"));
    }
}
