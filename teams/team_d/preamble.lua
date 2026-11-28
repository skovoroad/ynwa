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
local SHOOTING_ZONE_FROM = "I32"
local SHOOTING_ZONE_TO   = "R40"

-- Returns the number of the nearest of three teammates, compared in order.
local function nearest_teammate_number(n1, n2, n3)
    local my_pos = my_position()
    local best_number = n1
    local best_distance = distance(my_pos, teammate_position(n1))
    local d2 = distance(my_pos, teammate_position(n2))
    if d2 < best_distance then
        best_number, best_distance = n2, d2
    end
    local d3 = distance(my_pos, teammate_position(n3))
    if d3 < best_distance then
        best_number = n3
    end
    return best_number
end

local function nearest_teammate_ahead_number(n1, n2, n3)
    local my_pos = my_position()
    local best_number = 0
    local best_distance = math.huge

    if is_teammate_ahead(n1) then
        local d1 = distance(my_pos, teammate_position(n1))
        if d1 < best_distance then
            best_number, best_distance = n1, d1
        end
    end
    if is_teammate_ahead(n2) then
        local d2 = distance(my_pos, teammate_position(n2))
        if d2 < best_distance then
            best_number, best_distance = n2, d2
        end
    end
    if is_teammate_ahead(n3) then
        local d3 = distance(my_pos, teammate_position(n3))
        if d3 < best_distance then
            best_number, best_distance = n3, d3
        end
    end

    return best_number
end

function defender_with_ball()
    local number = nearest_teammate_ahead_number(6, 7, 8)
    if number > 0 then
        return pass_to(number)
    end
    number = nearest_teammate_number(6, 7, 8)
    return pass_to(number)
end

function midfielder_with_ball()
    local number = nearest_teammate_ahead_number(9, 10, 11)
    if number > 0 then
        return pass_to(number)
    end
    number = nearest_teammate_number(9, 10, 11)
    return pass_to(number)
end

function midfielder_without_ball()
    if distance(my_position(), ball_position()) < 10 then
         return chase_ball("midfielder_without_ball")
    else
         return run_to_defence_position("midfielder_without_ball")
    end
end

function forward_with_ball(my_number)
    if is_in_region(SHOOTING_ZONE_FROM, SHOOTING_ZONE_TO) then
        return kick_to_opponent_goal()
    end

    if count_opponents_around(my_position(), 5) > 0 then
        if my_number ~= 9 and is_teammate_free(9) then
            return pass_to(9, "pass to 9")
        end
        if my_number ~= 10 and is_teammate_free(10) then
            return pass_to(10, "pass to 10")
        end
        if my_number ~= 11 and is_teammate_free(11) then
            return pass_to(11, "pass to 11")
        end
    end

    return run_to_region(SHOOTING_ZONE_FROM, SHOOTING_ZONE_TO)
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
