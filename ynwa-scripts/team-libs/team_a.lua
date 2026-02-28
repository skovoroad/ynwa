-- Team A: dispatch tables for play and setup stages

-- Shooting zone: slightly deeper than penalty_area_b, slightly narrower.
-- I=col9, R=col18 (penalty spans cols 2-25); row 30 is ~3 rows deeper than penalty start.
local SHOOTING_ZONE_FROM = "I30"
local SHOOTING_ZONE_TO   = "R44"

local function pass_to_numbers(numbers)
    return pass_to_players_by_numbers(numbers)
end

function defender_with_ball()
    return pass_to_numbers({6, 7, 8})
end

function midfielder_with_ball()
    return pass_to_numbers({9, 10, 11})
end

function forward_with_ball()
    if is_in_region(SHOOTING_ZONE_FROM, SHOOTING_ZONE_TO) then
        return kick_to_opponent_goal()
    end
    return run_to_region(SHOOTING_ZONE_FROM, SHOOTING_ZONE_TO)
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}

team_setup = {
    start      = run_to_start_position,
    after_goal = run_to_start_position,
}