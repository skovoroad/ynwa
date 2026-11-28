-- Team A: dispatch tables for play and setup stages

-- Returns true if this player is among the 3 closest teammates to the ball
local function am_i_top3_closest_to_ball()
    local ball = ball_position()
    return count_teammates_around(ball, distance(my_position(), ball)) < 3
end

-- Action: chase ball if in top-3 closest, otherwise run to attack position
local function press_or_attack()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_attack_position()
end

-- Action: chase ball if in top-3 closest, otherwise run to defence position
local function press_or_defend()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_defence_position()
end

-- Assign via player_play in script.lua: player_play = goalkeeper_play
goalkeeper_play = {
    i_have_ball       = default_goalkeeper_with_ball,
    team_has_ball     = function() return run_to_defence_position() end,
    opponent_has_ball = function() return default_goalkeeper_cover_position() end,
}

-- Shooting zone: slightly deeper than penalty_area_b, slightly narrower.
-- I=col9, R=col18 (penalty spans cols 2-25); row 27 is ~3 rows deeper than penalty start (row 34 in 26×40).
local SHOOTING_ZONE_FROM = "I27"
local SHOOTING_ZONE_TO   = "R40"

-- Returns the number of the nearest teammate among `numbers`.
-- Intentionally written as a loop to match the future DSL design:
-- `for player in teammates <numbers> order by distance to me ascending`.
local function nearest_teammate_number(numbers)
    local my_pos = my_position()
    local ordered = {}
    for i, n in ipairs(numbers) do
        ordered[i] = {number = n, key = distance(my_pos, teammate_position(n)), idx = i}
    end
    table.sort(ordered, function(a, b)
        if a.key ~= b.key then return a.key < b.key end
        return a.idx < b.idx
    end)
    return ordered[1].number
end

function defender_with_ball()
    local number = nearest_teammate_number({6, 7, 8})
    return pass_to(number)
end

function midfielder_with_ball()
    local number = nearest_teammate_number({9, 10, 11})
    return pass_to(number)
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
