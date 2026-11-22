-- Scenario: goal_kick_teamA_left
-- Team B: два игрока.

team_play = {
    i_have_ball = function()
        -- Center of region M20:N21 on the 68 × 104.6 m football field.
        return kick_to_position({x = 34.0, z = 52.31})
    end,
    team_has_ball     = function() return stop("test behaviour") end,
    ball_is_free      = function() return stop("test behaviour") end,
    opponent_has_ball = function() return stop("test behaviour") end,
}


