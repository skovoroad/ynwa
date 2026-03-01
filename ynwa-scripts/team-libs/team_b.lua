-- Team B: dispatch tables for play and setup stages

local FORWARDS = {10, 11}

-- Assign via player_play in config: player_play = goalkeeper_play
goalkeeper_play = {
    i_have_ball       = function() return pass_to_nearest_teammate() end,
    team_has_ball     = function() return run_to_defence_position() end,
    opponent_has_ball = function() return goalkeeper_cover_position() end,
}

function nonforward_with_ball()
    local fwd10 = get_teammate_by_number(10)
    local fwd11 = get_teammate_by_number(11)
    local my_z  = my_position().z

    -- If a forward is ahead of me, pass to them
    if fwd10 and fwd10.position.z > my_z then return pass_to_teammate(fwd10) end
    if fwd11 and fwd11.position.z > my_z then return pass_to_teammate(fwd11) end

    -- I'm ahead: push into penalty area or shoot
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function forward_with_ball()
    local pos = my_regions()["attack position"]
    if pos and is_in_region_obj(pos) then
        return kick_to_opponent_goal()
    end
    return run_to_attack_position()
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
