use macroquad::prelude::*;
use ynwa_core::game::{Decision, DecisionTarget, GameConfig, GameState};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::team::Team;
use ynwa_football::events::FootballEvent;

/// `JournalEvent::External::kind` used by football semantic events.
const FOOTBALL_EVENT_KIND: &str = "football_event";

/// Vertical offset of the first telemetry line in the right-hand panel.
const PANEL_TOP: f32 = 40.0;
/// Line step of the telemetry block.
const LINE_HEIGHT: f32 = 30.0;
/// Vertical offset of the `Events:` title, below the time, status, and score lines.
const EVENT_PANEL_TOP: f32 = PANEL_TOP + LINE_HEIGHT * 4.5;
/// Height of the `Events:` title row.
const EVENT_TITLE_HEIGHT: f32 = 24.0;
/// Height of one journal row.
const EVENT_LINE_HEIGHT: f32 = 24.0;
/// Space kept below the events list for the scroll and pause hints.
const PANEL_HINT_HEIGHT: f32 = 64.0;
/// Margin between the panel content and the window bottom.
const PANEL_BOTTOM_MARGIN: f32 = 20.0;

/// Number of journal rows that fit between the `Events:` title and the panel hints.
pub fn event_panel_visible_rows() -> usize {
    let list_top = EVENT_PANEL_TOP + EVENT_TITLE_HEIGHT;
    let list_bottom = screen_height() - PANEL_BOTTOM_MARGIN - PANEL_HINT_HEIGHT;
    let available = (list_bottom - list_top).max(0.0);
    (available / EVENT_LINE_HEIGHT).floor() as usize
}

pub fn draw_control_panel(
    panel_x: f32,
    game_config: &GameConfig,
    game_state: &GameState,
    is_paused: bool,
    events: &[JournalEntry],
    event_scroll: usize,
) {
    let panel_x = panel_x + 20.0;

    draw_text(
        &format!("Time: {:.1}s", game_state.elapsed_time),
        panel_x,
        PANEL_TOP,
        24.0,
        WHITE,
    );

    let status = if is_paused { "PAUSED" } else { "Running" };
    let status_color = if is_paused { YELLOW } else { GREEN };
    draw_text(
        &format!("Status: {}", status),
        panel_x,
        PANEL_TOP + LINE_HEIGHT,
        24.0,
        status_color,
    );

    draw_score(
        panel_x,
        PANEL_TOP + LINE_HEIGHT * 2.5,
        game_config,
        game_state,
    );

    let visible_rows = event_panel_visible_rows();
    draw_event_panel(panel_x, EVENT_PANEL_TOP, events, event_scroll, visible_rows);

    let hint_y =
        EVENT_PANEL_TOP + EVENT_TITLE_HEIGHT + visible_rows as f32 * EVENT_LINE_HEIGHT + 10.0;
    draw_text(
        "Wheel / Up / Down / PgUp / PgDown - scroll events",
        panel_x,
        hint_y,
        16.0,
        LIGHTGRAY,
    );
    draw_text(
        "Space - pause/resume",
        panel_x,
        hint_y + 20.0,
        20.0,
        LIGHTGRAY,
    );
}

fn score_text(
    team_a_display_name: &str,
    score_a: u32,
    score_b: u32,
    team_b_display_name: &str,
) -> String {
    format!(
        "Score: {} {}  :  {} {}",
        team_a_display_name, score_a, score_b, team_b_display_name
    )
}

fn draw_score(x: f32, y: f32, game_config: &GameConfig, game_state: &GameState) {
    let score_a = game_state
        .team_stats
        .get(&Team::A)
        .map_or(0.0, |s| s.get("score")) as u32;
    let score_b = game_state
        .team_stats
        .get(&Team::B)
        .map_or(0.0, |s| s.get("score")) as u32;
    draw_text(
        &score_text(
            &game_config.team_a_display_name,
            score_a,
            score_b,
            &game_config.team_b_display_name,
        ),
        x,
        y,
        24.0,
        WHITE,
    );
}

fn team_label(team: Team) -> &'static str {
    match team {
        Team::A => "A",
        Team::B => "B",
    }
}

fn player_label(team: Team, number: u32) -> String {
    format!("{}#{}", team_label(team), number)
}

fn football_event_text(event: &FootballEvent) -> String {
    match event {
        // In a goal event the team is the one whose net was hit; the scorer is the opposite.
        FootballEvent::Goal(conceding_team) => {
            format!("Goal: team {}", team_label(conceding_team.opposite()))
        }
        FootballEvent::Touchline(_, last_team) => {
            format!("Throw-in; last touch: {}", team_label(*last_team))
        }
        FootballEvent::GoalLine(_, last_team) => {
            format!("Goal line; last touch: {}", team_label(*last_team))
        }
    }
}

fn event_text(event: &JournalEvent) -> String {
    match event {
        JournalEvent::DecisionAssigned {
            team,
            number,
            decision,
            reason,
            ..
        } => {
            let player = player_label(*team, *number);
            let decision = match decision {
                Decision::Run(target) => match target {
                    DecisionTarget::Region(region) => {
                        format!("Run to {:?}", region.to_grid_notation())
                    }
                    DecisionTarget::GridCell(cell) => {
                        let col_label = ynwa_core::GridCell::column_to_label(cell.col);
                        format!("Run to {}{}", col_label, cell.row)
                    }
                    DecisionTarget::Point(_) => "Run to point".to_string(),
                },
                Decision::Stop => "Stop".to_string(),
                Decision::Kick(_) => "Kick".to_string(),
            };
            match reason {
                Some(reason) => format!("{} {} ({})", player, decision, reason),
                None => format!("{} {}", player, decision),
            }
        }
        JournalEvent::PossessionChange {
            possessed_by_team,
            possessed_by_number,
            last_possessing_team,
            ..
        } => {
            let possessor = match (possessed_by_team, possessed_by_number) {
                (Some(team), Some(number)) => player_label(*team, *number),
                _ => "none".to_string(),
            };
            let last_team = last_possessing_team.map_or("none", team_label);
            format!("Possession: {} (last team {})", possessor, last_team)
        }
        JournalEvent::KickOutcome { team, number, .. } => {
            format!("Kick by {}", player_label(*team, *number))
        }
        JournalEvent::StageChange { stage } => format!("Stage: {:?}", stage),
        JournalEvent::RestartSet {
            restart_position,
            restart_team,
        } => match (restart_position, restart_team) {
            (Some(_), Some(team)) => format!("Restart: team {}", team_label(*team)),
            _ => "Restart: cleared".to_string(),
        },
        JournalEvent::DecisionsReset => "Decisions reset".to_string(),
        JournalEvent::StatUpdate { team, key, delta } => {
            format!("Stat: team {} {} {:+}", team_label(*team), key, delta)
        }
        JournalEvent::External { kind, data } => {
            match serde_json::from_value::<FootballEvent>(data.clone()) {
                Ok(event) if kind == FOOTBALL_EVENT_KIND => football_event_text(&event),
                _ => format!("External({}): {}", kind, data),
            }
        }
    }
}

/// Draws the visible window of journal entries, newest at the bottom when `scroll` is `0`.
fn draw_event_panel(x: f32, y: f32, events: &[JournalEntry], scroll: usize, visible_rows: usize) {
    let title_color = Color::new(0.9, 0.9, 0.9, 1.0);
    let text_color = Color::new(0.8, 0.8, 0.8, 1.0);

    let mut cursor = y;
    draw_text("Events:", x, cursor, 22.0, title_color);
    cursor += EVENT_TITLE_HEIGHT;

    if events.is_empty() {
        draw_text("(no events yet)", x, cursor, 20.0, text_color);
        return;
    }

    let end = events.len().saturating_sub(scroll);
    let start = end.saturating_sub(visible_rows);
    for entry in &events[start..end] {
        let text = format!("{:.1}s  {}", entry.timestamp, event_text(&entry.event));
        draw_text(&text, x, cursor, 20.0, text_color);
        cursor += EVENT_LINE_HEIGHT;
    }
}

pub fn draw_separator(x: f32, screen_height: f32) {
    draw_line(
        x,
        0.0,
        x,
        screen_height,
        2.0,
        Color::new(0.2, 0.2, 0.2, 1.0),
    );
}

#[cfg(test)]
#[path = "ui_tests.rs"]
mod tests;
