-- Scenario: goal_kick_teamA_left
-- Team A бьёт левее ворот B (col E, row 40) → мяч за лицевой → удар от ворот для Team B.

team_play = {
    i_have_ball = function()
        -- Center of cell E40 on the 68 × 104.6 m football field.
        return kick_to_position({x = 11.77, z = 103.31})
    end,
    team_has_ball     = chase_ball,
    ball_is_free      = chase_ball,
    opponent_has_ball = chase_ball,
}


