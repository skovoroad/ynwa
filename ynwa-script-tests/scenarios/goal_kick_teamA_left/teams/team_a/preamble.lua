-- Scenario: goal_kick_teamA_left
-- Team A бьёт левее ворот B (col E, row 40) → мяч за лицевой → удар от ворот для Team B.

team_play = {
    i_have_ball = function()
        return kick_to_cell("E40")
    end,
    team_has_ball     = function() return chase_ball() end,
    ball_is_free      = function() return chase_ball() end,
    opponent_has_ball = function() return chase_ball() end,
}

team_setup = {
    start     = function() return run_to_region("N16", "N16") end,
    goal_kick = default_goal_kick_setup,
}
