use macroquad::prelude::*;
use ynwa_core::game::{Decision, DecisionTarget, GameConfig, GameState, PlayerState};
use ynwa_core::journal::{JournalEntry, JournalEvent};
use ynwa_core::team::Team;
use ynwa_football::events::FootballEvent;

/// `JournalEvent::External::kind` used by football semantic events.
const FOOTBALL_EVENT_KIND: &str = "football_event";

/// How many of the most recent events the replay panel shows at once.
pub const EVENT_PANEL_SIZE: usize = 5;

pub fn draw_control_panel(
    panel_x: f32,
    game_config: &GameConfig,
    game_state: &GameState,
    is_paused: bool,
    events: Option<&[JournalEntry]>,
    event_scroll: usize,
) {
    let panel_x = panel_x + 20.0;
    let mut y_offset = 40.0;
    let line_height = 30.0;

    draw_text(
        &format!("Time: {:.1}s", game_state.elapsed_time),
        panel_x,
        y_offset,
        24.0,
        WHITE,
    );
    y_offset += line_height;

    let status = if is_paused { "PAUSED" } else { "Running" };
    let status_color = if is_paused { YELLOW } else { GREEN };
    draw_text(
        &format!("Status: {}", status),
        panel_x,
        y_offset,
        24.0,
        status_color,
    );
    y_offset += line_height * 1.5;

    draw_score(panel_x, y_offset, game_state);
    y_offset += line_height * 2.0;

    // `Some` marks the replay panel (drawn even before the first event); `None` is play/record.
    if let Some(events) = events {
        draw_event_panel(panel_x, y_offset, events, event_scroll);
        let rows = events.len().clamp(1, EVENT_PANEL_SIZE);
        y_offset += 24.0 * (rows as f32 + 1.0) + 10.0;
        draw_text(
            "Wheel / Up / Down - scroll events",
            panel_x,
            y_offset,
            16.0,
            LIGHTGRAY,
        );
        y_offset += 20.0;
    }

    draw_text("Space - pause/resume", panel_x, y_offset, 20.0, LIGHTGRAY);
    y_offset += line_height * 2.0;

    draw_player_decisions_table(panel_x, y_offset, game_config, game_state);
}

fn draw_score(x: f32, y: f32, game_state: &GameState) {
    let score_a = game_state
        .team_stats
        .get(&Team::A)
        .map_or(0.0, |s| s.get("score")) as u32;
    let score_b = game_state
        .team_stats
        .get(&Team::B)
        .map_or(0.0, |s| s.get("score")) as u32;
    draw_text(
        &format!("Score:  A {}  :  {} B", score_a, score_b),
        x,
        y,
        24.0,
        WHITE,
    );
}

fn decision_text(decision: &Decision) -> String {
    match decision {
        Decision::Run(target) => match target {
            DecisionTarget::Region(region) => {
                format!("Run to {:?}", region.to_grid_notation())
            }
            DecisionTarget::GridCell(cell) => {
                let col_label = ynwa_core::GridCell::column_to_label(cell.col);
                format!("Run to {}{}", col_label, cell.row)
            }
            DecisionTarget::Point(_) => "Run to point".to_string(),
            DecisionTarget::Ball => "Chase ball".to_string(),
        },
        Decision::Stop => "Stop".to_string(),
        Decision::Kick(_) => "Kick".to_string(),
    }
}

fn player_decision_text(player_state: &PlayerState) -> String {
    match &player_state.current_decision {
        Some(decision) => decision_text(decision),
        None => match &player_state.last_error {
            Some(error) => format!("ERROR: {}", error),
            None => "—".to_string(),
        },
    }
}

fn draw_player_column(
    x: f32,
    start_y: f32,
    players: &[(usize, &ynwa_core::game::PlayerDef)],
    game_state: &GameState,
    label: &str,
) {
    let line_height = 26.0;
    let sub_line_height = 20.0;
    let row_gap = 4.0;
    let header_color = Color::new(0.9, 0.9, 0.9, 1.0);
    let text_color = Color::new(0.8, 0.8, 0.8, 1.0);
    let meta_color = Color::new(0.55, 0.55, 0.55, 1.0);

    let mut y = start_y;
    draw_text(label, x, y, 22.0, header_color);
    y += line_height;
    draw_text("#", x, y, 20.0, header_color);
    draw_text("Decision", x + 36.0, y, 20.0, header_color);
    y += line_height;

    for (i, player_def) in players {
        let player_state = &game_state.player_states[*i];

        draw_text(&player_def.number.to_string(), x, y, 20.0, text_color);
        draw_text(
            &player_decision_text(player_state),
            x + 36.0,
            y,
            20.0,
            text_color,
        );
        y += line_height;

        let time_str = if player_state.current_decision.is_some() {
            format!("t={:.1}s", player_state.last_decision_time)
        } else {
            "t=—".to_string()
        };
        let meta_text = if let Some(reason) = &player_state.decision_reason {
            format!("  {} | {}", time_str, reason)
        } else {
            format!("  {}", time_str)
        };
        draw_text(&meta_text, x + 36.0, y, 16.0, meta_color);
        y += sub_line_height + row_gap;

        if y > screen_height() - 20.0 {
            break;
        }
    }
}

fn draw_player_decisions_table(
    x: f32,
    start_y: f32,
    game_config: &GameConfig,
    game_state: &GameState,
) {
    let header_color = Color::new(0.9, 0.9, 0.9, 1.0);
    draw_text("Player Decisions:", x, start_y, 22.0, header_color);

    // Split players into two teams by their team field
    let team_a: Vec<(usize, &ynwa_core::game::PlayerDef)> = game_config
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.team == ynwa_core::team::Team::A)
        .collect();
    let team_b: Vec<(usize, &ynwa_core::game::PlayerDef)> = game_config
        .players
        .iter()
        .enumerate()
        .filter(|(_, p)| p.team == ynwa_core::team::Team::B)
        .collect();

    let col_width = (screen_width() - x) / 2.0 - 10.0;
    let table_y = start_y + 30.0;

    draw_player_column(x, table_y, &team_a, game_state, "Team A");
    draw_player_column(x + col_width, table_y, &team_b, game_state, "Team B");
}

fn team_label(team: Team) -> &'static str {
    match team {
        Team::A => "A",
        Team::B => "B",
    }
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
        FootballEvent::GameEnd => "Match end".to_string(),
    }
}

fn event_text(event: &JournalEvent) -> String {
    match event {
        JournalEvent::DecisionAssigned {
            player_index,
            decision,
            reason,
        } => match reason {
            Some(reason) => format!("#{} {} ({})", player_index, decision_text(decision), reason),
            None => format!("#{} {}", player_index, decision_text(decision)),
        },
        JournalEvent::PossessionChange {
            possessed_by,
            last_possessing_team,
        } => {
            let possessor = possessed_by.map_or("none".to_string(), |index| format!("#{}", index));
            let last_team = last_possessing_team.map_or("none", team_label);
            format!("Possession: {} (last team {})", possessor, last_team)
        }
        JournalEvent::KickOutcome { player_index, .. } => {
            format!("Kick by #{}", player_index)
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
fn draw_event_panel(x: f32, y: f32, events: &[JournalEntry], scroll: usize) {
    let line_height = 24.0;
    let title_color = Color::new(0.9, 0.9, 0.9, 1.0);
    let text_color = Color::new(0.8, 0.8, 0.8, 1.0);

    let mut cursor = y;
    draw_text("Events:", x, cursor, 22.0, title_color);
    cursor += line_height;

    if events.is_empty() {
        draw_text("(no events yet)", x, cursor, 20.0, text_color);
        return;
    }

    let end = events.len().saturating_sub(scroll);
    let start = end.saturating_sub(EVENT_PANEL_SIZE);
    for entry in &events[start..end] {
        let text = format!("{:.1}s  {}", entry.timestamp, event_text(&entry.event));
        draw_text(&text, x, cursor, 20.0, text_color);
        cursor += line_height;
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
