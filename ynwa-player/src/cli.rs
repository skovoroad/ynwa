use std::path::{Path, PathBuf};

pub const DEFAULT_TEAMS_PATH: &str = "teams";
pub const DEFAULT_PREAMBLES_PATH: &str = "ynwa-scripts/preambles";

pub enum Mode {
    Play,
    Record,
    Replay(PathBuf),
}

pub struct Cli {
    pub mode: Mode,
    pub teams_path: PathBuf,
    pub preambles_path: PathBuf,
    pub out_dir: Option<PathBuf>,
}

pub fn parse(args: &[String]) -> Result<Cli, String> {
    let mut record = false;
    let mut replay: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut positionals: Vec<String> = Vec::new();

    let mut index = 0;
    while index < args.len() {
        let arg = args[index].as_str();
        match arg {
            "--record" => {
                if record {
                    return Err("--record specified more than once".to_string());
                }
                record = true;
            }
            "--replay" => {
                if replay.is_some() {
                    return Err("--replay specified more than once".to_string());
                }
                index += 1;
                replay = Some(PathBuf::from(option_value(args, index, "--replay")?));
            }
            "--out" => {
                if out_dir.is_some() {
                    return Err("--out specified more than once".to_string());
                }
                index += 1;
                out_dir = Some(PathBuf::from(option_value(args, index, "--out")?));
            }
            other if other.starts_with("--") => {
                return Err(format!("unknown flag: {}", other));
            }
            other => positionals.push(other.to_string()),
        }
        index += 1;
    }

    if record && replay.is_some() {
        return Err("--record and --replay cannot be used together".to_string());
    }
    if out_dir.is_some() && !record {
        return Err("--out requires --record".to_string());
    }
    if positionals.len() > 2 {
        return Err("at most two positional arguments are allowed".to_string());
    }

    let mode = if record {
        Mode::Record
    } else if let Some(path) = replay {
        Mode::Replay(path)
    } else {
        Mode::Play
    };

    Ok(Cli {
        mode,
        teams_path: PathBuf::from(
            positionals
                .first()
                .map(String::as_str)
                .unwrap_or(DEFAULT_TEAMS_PATH),
        ),
        preambles_path: PathBuf::from(
            positionals
                .get(1)
                .map(String::as_str)
                .unwrap_or(DEFAULT_PREAMBLES_PATH),
        ),
        out_dir,
    })
}

fn option_value<'a>(args: &'a [String], index: usize, flag: &str) -> Result<&'a str, String> {
    match args.get(index) {
        Some(value) if !value.starts_with("--") => Ok(value.as_str()),
        _ => Err(format!("{} requires a value", flag)),
    }
}

/// Directory recordings are written to: `out` when given, otherwise `<home>/.ynwa/games`.
/// Created if missing.
pub fn games_dir(out: Option<&Path>) -> Result<PathBuf, String> {
    let dir = match out {
        Some(path) => path.to_path_buf(),
        None => env_home::env_home_dir()
            .ok_or_else(|| "home directory not found".to_string())?
            .join(".ynwa/games"),
    };
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir)
}

/// First free `game_<local timestamp>.jsonl` path in `dir`, adding `_2`, `_3`, … on name clashes.
pub fn next_record_path(dir: &Path) -> Result<PathBuf, String> {
    let timestamp = chrono::Local::now().format("%Y%m%d-%H%M%S");
    let candidate = dir.join(format!("game_{}.jsonl", timestamp));
    if !candidate.exists() {
        return Ok(candidate);
    }

    let mut suffix = 2;
    loop {
        let candidate = dir.join(format!("game_{}_{}.jsonl", timestamp, suffix));
        if !candidate.exists() {
            return Ok(candidate);
        }
        suffix += 1;
    }
}
