-- Scenario: goal_kick_teamB_right
-- Team B бьёт правее ворот A (col V, row 40 в системе B) → мяч за лицевой → удар от ворот для Team A.

team_play = {
    i_have_ball = function()
        return kick_to_cell("V40")
    end,
    team_has_ball     = function() return chase_ball() end,
    ball_is_free      = function() return chase_ball() end,
    opponent_has_ball = function() return chase_ball() end,
}

team_setup = {
    start      = function() return run_to_region("N16", "N16") end,
    after_goal = function() return run_to_region("N16", "N16") end,
    throw_in   = function() return run_to_region("N10", "N10") end,
    goal_kick  = function() return run_to_region("N10", "N10") end,
    corner     = function() return run_to_region("N10", "N10") end,
}
