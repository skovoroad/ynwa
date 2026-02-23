use crate::field::zones::Point3D;
use crate::game::Game;
use crate::team::Team;
use uom::si::length::meter;

/// Football game events
#[derive(Debug, Clone, PartialEq)]
pub enum FootballEvent {
    /// Goal scored by a team
    Goal(Team),
    /// Ball went out of bounds on sideline (position where it crossed)
    Touchline(Point3D),
    /// Ball went out of bounds on goal line (position where it crossed)
    GoalLine(Point3D),
    /// Game ended
    GameEnd,
}

const BALL_RADIUS: f32 = 0.11; // meters (standard football radius ~11cm)
pub const GAME_DURATION: f32 = 60.0; // seconds (1 minute for testing)

/// Check if a goal was scored
/// Ball must be completely inside the goal zone
pub fn check_goal(game: &Game) -> Option<FootballEvent> {
    let ball_pos = &game.state.ball_state.position;
    let field = &game.config().field;

    // Check both goals
    for ((name, team), zone) in field.zones() {
        if name == "goal" && team.is_some() && is_ball_completely_in_zone(ball_pos, zone, BALL_RADIUS) {
            return Some(FootballEvent::Goal(team.unwrap()));
        }
    }

    None
}

/// Check if ball went out on sideline (width boundaries, X axis)
pub fn check_touchline(game: &Game) -> Option<FootballEvent> {
    let ball_pos = &game.state.ball_state.position;
    let field_width = game.config().field.width().get::<meter>();

    // Ball completely over left or right sideline (X axis = width)
    if ball_pos.x.get::<meter>() - BALL_RADIUS < 0.0
        || ball_pos.x.get::<meter>() + BALL_RADIUS > field_width
    {
        return Some(FootballEvent::Touchline(*ball_pos));
    }

    None
}

/// Check if ball went out on goal line (length boundaries, Z axis)
pub fn check_goal_line(game: &Game) -> Option<FootballEvent> {
    let ball_pos = &game.state.ball_state.position;
    let field_length = game.config().field.length().get::<meter>();

    // Ball completely over near or far goal line (Z axis = length)
    if ball_pos.z.get::<meter>() - BALL_RADIUS < 0.0
        || ball_pos.z.get::<meter>() + BALL_RADIUS > field_length
    {
        return Some(FootballEvent::GoalLine(*ball_pos));
    }

    None
}

/// Check if game time has ended
pub fn check_game_end(game: &Game) -> Option<FootballEvent> {
    if game.state.elapsed_time >= GAME_DURATION {
        return Some(FootballEvent::GameEnd);
    }

    None
}

/// Check all football events and return the first one detected
/// Priority: Goal > Game End > Touchline > Goal Line
pub fn check_events(game: &Game) -> Option<FootballEvent> {
    // Check goal first (highest priority)
    if let Some(event) = check_goal(game) {
        return Some(event);
    }

    // Check game end
    if let Some(event) = check_game_end(game) {
        return Some(event);
    }

    // Check out of bounds
    if let Some(event) = check_touchline(game) {
        return Some(event);
    }

    if let Some(event) = check_goal_line(game) {
        return Some(event);
    }

    None
}

/// Helper: check if ball (with radius) is completely inside a zone
fn is_ball_completely_in_zone(
    ball_pos: &Point3D,
    zone: &crate::field::Zone,
    ball_radius: f32,
) -> bool {
    use crate::field::zones::ZoneGeometry;

    match &zone.geometry {
        ZoneGeometry::Rectangle(rect) => {
            let x = ball_pos.x.get::<meter>();
            let z = ball_pos.z.get::<meter>();

            x - ball_radius >= rect.min.x.get::<meter>()
                && x + ball_radius <= rect.max.x.get::<meter>()
                && z - ball_radius >= rect.min.z.get::<meter>()
                && z + ball_radius <= rect.max.z.get::<meter>()
        }
        _ => false, // Goals should be rectangles
    }
}

#[cfg(test)]
#[path = "../tests/events_tests.rs"]
mod tests;
