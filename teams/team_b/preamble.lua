-- Team B: dispatch tables for play and setup stages

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

function nonforward_with_ball()
    -- If a forward is ahead of me, pass to them
    if is_teammate_ahead(10) then
        return kick_to_position(teammate_position(10), "pass_to_#10")
    end
    if is_teammate_ahead(11) then
        return kick_to_position(teammate_position(11), "pass_to_#11")
    end

    -- I'm ahead: push into penalty area or shoot
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function forward_with_ball()
    -- Shoot in the opponent penalty area; otherwise carry the ball into it.
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}
