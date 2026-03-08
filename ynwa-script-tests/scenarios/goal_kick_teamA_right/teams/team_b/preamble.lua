-- Scenario: goal_kick_teamA_right
-- Team B: два игрока.

team_play = {
    i_have_ball = function()
        return kick_to_region("M20", "N21")
    end,
    team_has_ball     = function() return stop("test behaviour") end,
    ball_is_free      = function() return stop("test behaviour") end,
    opponent_has_ball = function() return stop("test behaviour") end,
}


