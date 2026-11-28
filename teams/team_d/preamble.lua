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

function defender_with_ball()
    for player in order_by_distance_asc({6, 7, 8}) do
        if is_teammate_ahead(player) then
            return pass_to(player)
        end
    end
    for player in order_by_distance_asc({6, 7, 8}) do
        return pass_to(player)
    end
end

function midfielder_with_ball()
    for player in order_by_distance_asc({9, 10, 11}) do
        if is_teammate_ahead(player) then
            return pass_to(player)
        end
    end
    for player in order_by_distance_asc({9, 10, 11}) do
        return pass_to(player)
    end
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
        -- Pass to the first free teammate among the forwards except me.
        -- Intentionally written as a loop to match the future DSL design:
        -- `for player in teammates 9, 10, 11 [ if player is not me and player free [ decision: pass to player ] ]`.
        for player in each({9, 10, 11}) do
            if my_number ~= player and is_teammate_free(player) then
                return pass_to(player, "pass to " .. player)
            end
        end
    end

    return run_to_region(SHOOTING_ZONE_FROM, SHOOTING_ZONE_TO)
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
