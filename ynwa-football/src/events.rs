use ynwa_core::field::zones::Point3D;
use ynwa_core::game::Game;
use ynwa_core::team::Team;
use uom::si::length::meter;

/// Football game events
#[derive(Debug, Clone, PartialEq)]
pub enum FootballEvent {
    /// Goal scored by a team
    Goal(Team),
    /// Ball went out of bounds on sideline: crossing position + last team to touch
    Touchline(Point3D, Team),
    /// Ball went out of bounds on goal line: crossing position + last team to touch
    GoalLine(Point3D, Team),
    /// Game ended
    GameEnd,
}

pub(crate) const BALL_RADIUS: f32 = 0.11; // meters (standard football radius ~11cm)
pub const GAME_DURATION: f32 = 120.0; // seconds (1 minute for testing)

/// Check if a goal was scored.
/// Goal condition (FIFА): ball has completely crossed the goal line between the goalposts.
/// - Z axis (goal line): ball fully past the line, i.e. ball radius is accounted for
/// - X axis (goalposts): ball center is within the goal width, no radius — touching the
///   post from inside counts as a goal; touching from outside does not
pub fn check_goal(game: &Game) -> Option<FootballEvent> {
    let ball_pos = &game.state.ball_state.position;
    let field = &game.config().field;

    // Check both goals
    for ((name, team), zone) in field.zones() {
        if name == "goal" && team.is_some() && is_ball_in_goal(ball_pos, zone, BALL_RADIUS) {
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
        let last_team = game.state.ball_state.last_possessing_team.unwrap_or(Team::A);
        return Some(FootballEvent::Touchline(*ball_pos, last_team));
    }

    None
}

/// Check if ball went out on goal line (length boundaries, Z axis).
/// Does NOT fire if the ball is between the goalposts — that is handled by check_goal.
pub fn check_goal_line(game: &Game) -> Option<FootballEvent> {
    let ball_pos = &game.state.ball_state.position;
    let field_length = game.config().field.length().get::<meter>();
    let ball_x = ball_pos.x.get::<meter>();
    let ball_z = ball_pos.z.get::<meter>();

    let crossed_near = ball_z - BALL_RADIUS < 0.0;
    let crossed_far = ball_z + BALL_RADIUS > field_length;

    if !crossed_near && !crossed_far {
        return None;
    }

    // If ball is between the goalposts, let check_goal handle it
    if is_ball_between_goalposts(game, ball_x) {
        return None;
    }

    let last_team = game.state.ball_state.last_possessing_team.unwrap_or(Team::A);
    Some(FootballEvent::GoalLine(*ball_pos, last_team))
}

/// Returns true if the ball's X position is within any goal's width on this field.
fn is_ball_between_goalposts(game: &Game, ball_x: f32) -> bool {
    use ynwa_core::field::zones::ZoneGeometry;

    for ((name, _team), zone) in game.config().field.zones() {
        if name != "goal" {
            continue;
        }
        if let ZoneGeometry::Rectangle(rect) = &zone.geometry {
            let x_min = rect.min.x.get::<meter>();
            let x_max = rect.max.x.get::<meter>();
            if ball_x >= x_min && ball_x <= x_max {
                return true;
            }
        }
    }
    false
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

fn is_ball_in_goal(
    ball_pos: &Point3D,
    zone: &ynwa_core::field::Zone,
    ball_radius: f32,
) -> bool {
    use ynwa_core::field::zones::ZoneGeometry;

    match &zone.geometry {
        ZoneGeometry::Rectangle(rect) => {
            let x = ball_pos.x.get::<meter>();
            let z = ball_pos.z.get::<meter>();
            let x_min = rect.min.x.get::<meter>();
            let x_max = rect.max.x.get::<meter>();
            let z_min = rect.min.z.get::<meter>();
            let z_max = rect.max.z.get::<meter>();

            // Ball fully crossed the goal line (Z axis): radius counts
            let crossed_line = z - ball_radius >= z_min && z + ball_radius <= z_max;
            // Ball center between the goalposts (X axis): no radius
            let between_posts = x >= x_min && x <= x_max;

            crossed_line && between_posts
        }
        _ => false,
    }
}

#[cfg(test)]
#[path = "tests/events_tests.rs"]
mod tests;
