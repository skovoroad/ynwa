use std::path::{Path, PathBuf};

use serde::Deserialize;
use ynwa_core::repository::{PlayerRecord, PlayerStatic, PlayerTactical, TeamRecord, TeamRepository};

#[derive(Debug, Deserialize)]
struct StaticToml {
    name: String,
    reaction_rate: u32,
    speed_rate: u32,
    tackle_rate: u32,
    shot_power: u32,
    shot_accuracy: u32,
}

#[derive(Debug, Deserialize)]
struct TacticalToml {
    number: u32,
    start_position: String,
    attack_position: String,
    defence_position: String,
    #[serde(default)]
    set_piece_positions: std::collections::HashMap<String, String>,
}

fn read_toml<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("Cannot read '{}': {}", path.display(), e))?;
    toml::from_str(&content)
        .map_err(|e| format!("Cannot parse '{}': {}", path.display(), e))
}

fn load_player(dir: &Path) -> Result<PlayerRecord, String> {
    let s: StaticToml = read_toml(&dir.join("static.toml"))?;
    let t: TacticalToml = read_toml(&dir.join("tactical.toml"))?;

    let script_path = dir.join("script.lua");
    let script = if script_path.exists() {
        Some(
            std::fs::read_to_string(&script_path)
                .map_err(|e| format!("Cannot read '{}': {}", script_path.display(), e))?,
        )
    } else {
        None
    };

    Ok(PlayerRecord {
        static_data: PlayerStatic {
            name: s.name,
            reaction_rate: s.reaction_rate,
            speed_rate: s.speed_rate,
            tackle_rate: s.tackle_rate,
            shot_power: s.shot_power,
            shot_accuracy: s.shot_accuracy,
        },
        tactical: PlayerTactical {
            number: t.number,
            start_position: t.start_position,
            attack_position: t.attack_position,
            defence_position: t.defence_position,
            set_piece_positions: t.set_piece_positions,
        },
        script,
    })
}

/// Loads team data from a directory tree rooted at a base path.
/// `load_team(team_id)` resolves to `<base>/<team_id>/`.
pub struct FsTeamRepository {
    base: PathBuf,
}

impl FsTeamRepository {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }
}

impl TeamRepository for FsTeamRepository {
    fn load_team(&self, team_id: &str) -> Result<TeamRecord, String> {
        let team_dir = self.base.join(team_id);

        let preamble_path = team_dir.join("preamble.lua");
        let preamble = std::fs::read_to_string(&preamble_path)
            .map_err(|e| format!("Cannot read '{}': {}", preamble_path.display(), e))?;

        let players_dir = team_dir.join("players");
        let mut entries: Vec<_> = std::fs::read_dir(&players_dir)
            .map_err(|e| format!("Cannot read '{}': {}", players_dir.display(), e))?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        // Deterministic order — sort by directory name.
        entries.sort_by_key(|e| e.file_name());

        let players = entries
            .iter()
            .map(|e| load_player(&e.path()))
            .collect::<Result<Vec<_>, _>>()?;

        if players.is_empty() {
            return Err(format!("No players found in '{}'", players_dir.display()));
        }

        Ok(TeamRecord { players, preamble })
    }
}
