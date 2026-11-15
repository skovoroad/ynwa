use std::path::{Path, PathBuf};

pub const DEFAULT_TEAMS_PATH: &str = "teams";
pub const DEFAULT_PREAMBLES_PATH: &str = "ynwa-scripts/preambles";
pub const DEFAULT_TEAM_A_ID: &str = "team_a";
pub const DEFAULT_TEAM_B_ID: &str = "team_b";
pub const DEFAULT_TEMPERATURE: f32 = 0.7;
pub const DEFAULT_TIMESTEP: f32 = 1.0 / 60.0;
pub const DEFAULT_DURATION: f32 = 120.0;

const RECORDING_EXTENSION: &str = "jsonl";

pub struct Cli {
    pub teams_path: PathBuf,
    pub preambles_path: PathBuf,
    pub team_a_id: String,
    pub team_b_id: String,
    pub out: Option<PathBuf>,
    pub seed: Option<u64>,
    pub temperature: f32,
    pub timestep: f32,
    pub duration: f32,
}

pub fn parse(args: &[String]) -> Result<Cli, String> {
    let mut teams_path = None;
    let mut preambles_path = None;
    let mut team_a_id = None;
    let mut team_b_id = None;
    let mut out = None;
    let mut seed = None;
    let mut temperature = None;
    let mut timestep = None;
    let mut duration = None;

    let mut index = 0;
    while index < args.len() {
        let flag = args[index].as_str();
        match flag {
            "--teams" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut teams_path, PathBuf::from(value), flag)?;
            }
            "--preambles" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut preambles_path, PathBuf::from(value), flag)?;
            }
            "--team-a" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut team_a_id, value.to_string(), flag)?;
            }
            "--team-b" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut team_b_id, value.to_string(), flag)?;
            }
            "--out" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut out, PathBuf::from(value), flag)?;
            }
            "--seed" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut seed, parse_seed(value)?, flag)?;
            }
            "--temperature" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut temperature, parse_temperature(value)?, flag)?;
            }
            "--timestep" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut timestep, parse_positive_finite(flag, value)?, flag)?;
            }
            "--duration" => {
                index += 1;
                let value = option_value(args, index, flag)?;
                set_once(&mut duration, parse_positive_finite(flag, value)?, flag)?;
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {}", other));
            }
            other => {
                return Err(format!("unexpected positional argument: {}", other));
            }
        }
        index += 1;
    }

    Ok(Cli {
        teams_path: teams_path.unwrap_or_else(|| PathBuf::from(DEFAULT_TEAMS_PATH)),
        preambles_path: preambles_path.unwrap_or_else(|| PathBuf::from(DEFAULT_PREAMBLES_PATH)),
        team_a_id: team_a_id.unwrap_or_else(|| DEFAULT_TEAM_A_ID.to_string()),
        team_b_id: team_b_id.unwrap_or_else(|| DEFAULT_TEAM_B_ID.to_string()),
        out,
        seed,
        temperature: temperature.unwrap_or(DEFAULT_TEMPERATURE),
        timestep: timestep.unwrap_or(DEFAULT_TIMESTEP),
        duration: duration.unwrap_or(DEFAULT_DURATION),
    })
}

/// Resolves where the recording of the match is written.
///
/// A path with the `.jsonl` extension is used as the exact recording file, creating its parent
/// directories when needed. Any other path is a directory (created when needed) that receives a
/// `<team_a>_vs_<team_b>_<timestamp>.jsonl` name. Without `--out`, recordings go to
/// `<home>/.ynwa/games`.
pub fn recording_path(
    out: Option<&Path>,
    team_a_id: &str,
    team_b_id: &str,
) -> Result<PathBuf, String> {
    match out {
        Some(path) if is_recording_file(path) => {
            create_parent_dir(path)?;
            Ok(path.to_path_buf())
        }
        Some(dir) => next_recording_path(dir, team_a_id, team_b_id),
        None => next_recording_path(&default_recordings_dir()?, team_a_id, team_b_id),
    }
}

fn default_recordings_dir() -> Result<PathBuf, String> {
    let home = env_home::env_home_dir().ok_or_else(|| "home directory not found".to_string())?;
    Ok(home.join(".ynwa/games"))
}

fn is_recording_file(path: &Path) -> bool {
    path.extension()
        .is_some_and(|extension| extension == RECORDING_EXTENSION)
}

fn create_parent_dir(path: &Path) -> Result<(), String> {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())
        }
        _ => Ok(()),
    }
}

/// First free descriptive path in `dir`, appending `_2`, `_3`, ... on name clashes.
fn next_recording_path(dir: &Path, team_a_id: &str, team_b_id: &str) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir).map_err(|error| error.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let base = format!("{}_vs_{}_{}", team_a_id, team_b_id, timestamp);
    Ok(collision_free_path(dir, &base))
}

fn collision_free_path(dir: &Path, base: &str) -> PathBuf {
    let mut suffix = 1;
    loop {
        let name = match suffix {
            1 => format!("{}.jsonl", base),
            _ => format!("{}_{}.jsonl", base, suffix),
        };
        let candidate = dir.join(name);
        if !candidate.exists() {
            return candidate;
        }
        suffix += 1;
    }
}

fn parse_seed(value: &str) -> Result<u64, String> {
    value
        .parse()
        .map_err(|_| format!("--seed must be a non-negative integer, got '{}'", value))
}

fn parse_temperature(value: &str) -> Result<f32, String> {
    let temperature: f32 = value
        .parse()
        .map_err(|_| format!("--temperature must be a number, got '{}'", value))?;
    if !(0.0..=1.0).contains(&temperature) {
        return Err(format!(
            "--temperature must be in [0.0, 1.0], got '{}'",
            value
        ));
    }
    Ok(temperature)
}

fn parse_positive_finite(flag: &str, value: &str) -> Result<f32, String> {
    let number: f32 = value
        .parse()
        .map_err(|_| format!("{} must be a number, got '{}'", flag, value))?;
    if !number.is_finite() || number <= 0.0 {
        return Err(format!(
            "{} must be a positive finite number, got '{}'",
            flag, value
        ));
    }
    Ok(number)
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("{} specified more than once", flag));
    }
    *slot = Some(value);
    Ok(())
}

fn option_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    match args.get(index) {
        Some(value) if !value.starts_with("--") => Ok(value.as_str()),
        _ => Err(format!("{} requires a value", flag)),
    }
}

#[cfg(test)]
#[path = "cli_tests.rs"]
mod tests;
