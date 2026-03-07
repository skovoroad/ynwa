-- Scenario: goal_kick_teamB_left
-- Team A: один игрок, стоит на стартовой позиции.
-- В фазе возобновления (goal_kick): Team A restarting → идёт к мячу; иначе — 25м от центра.

-- В игре: если мяч достался — бить в ворота; иначе — стоять
team_play = {
    i_have_ball = function()
        return kick_to_region("M20", "N21")
    end,
    team_has_ball     = function() return {action = "stop"} end,
    ball_is_free      = function() return {action = "stop"} end,
    opponent_has_ball = function() return {action = "stop"} end,
}

-- При возобновлении: если наша команда бьёт — идти к мячу; иначе — в 25м от центра.
local function restart_setup()
    if is_my_team_restarting() then
        local rp = get_restart_position()
        if rp then
            return {action = "run", target_type = "point", target = {x = rp.x, z = rp.z, y = 0}}
        end
    end
    return run_to_region("N10", "N10")
end

team_setup = {
    start      = function() return run_to_region("N16", "N16") end,
    after_goal = function() return run_to_region("N16", "N16") end,
    throw_in   = restart_setup,
    goal_kick  = restart_setup,
    corner     = restart_setup,
}
