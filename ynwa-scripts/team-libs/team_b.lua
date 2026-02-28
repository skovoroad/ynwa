-- Team B: dispatch tables for play and setup stages

local FORWARDS = {10, 11}

-- Assign via player_play in config: player_play = goalkeeper_play
goalkeeper_play = {
    i_have_ball       = function() return pass_to_nearest_teammate() end,
    team_has_ball     = function() return run_to_defence_position() end,
    opponent_has_ball = function() return goalkeeper_cover_position() end,
}

function nonforward_with_ball()
    local pos = my_regions()["attack position"]
    if pos and is_in_region_obj(pos) then
        return pass_to_players_by_numbers(FORWARDS)
    end
    return run_to_attack_position()
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
