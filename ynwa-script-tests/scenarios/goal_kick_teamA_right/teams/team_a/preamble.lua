-- Scenario: goal_kick_teamA_right
-- Team A бьёт правее ворот B (col V, row 40) → мяч за лицевой → удар от ворот для Team B.

team_play = {
    i_have_ball = function()
        return kick_to_cell("V40")
    end,
    team_has_ball     = chase_ball,
    ball_is_free      = chase_ball,
    opponent_has_ball = chase_ball,
}


