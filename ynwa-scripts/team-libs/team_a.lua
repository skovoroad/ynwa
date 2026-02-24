-- Team A: dispatch tables for play and setup stages

-- Shooting zone: slightly deeper than penalty_area_b, slightly narrower.
-- I=col9, R=col18 (penalty spans cols 2-25); row 30 is ~3 rows deeper than penalty start.
local SHOOTING_ZONE_FROM = "I30"
local SHOOTING_ZONE_TO   = "R44"

local function pass_to_numbers(numbers)
    local my_pos = my_position()
    local best, best_dist = nil, math.huge
    for _, tm in ipairs(get_teammates()) do
        for _, n in ipairs(numbers) do
            if tm.number == n then
                local d = distance(my_pos, tm.position)
                if d < best_dist then best_dist = d; best = tm end
            end
        end
    end
    if best then
        return {action = "kick", target = {x = best.position.x, z = best.position.z}, reason = "pass_to_#" .. best.number}
    end
    return kick_to_opponent_goal()
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