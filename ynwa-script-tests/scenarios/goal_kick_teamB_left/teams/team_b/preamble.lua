-- Scenario: goal_kick_teamB_left
-- Team B бьёт левее ворот A (col E, row 40 в системе B) → мяч за лицевой → удар от ворот для Team A.

team_play = {
    i_have_ball = function()
        return kick_to_cell("E40")
    end,
    team_has_ball     = chase_ball,
    ball_is_free      = chase_ball,
    opponent_has_ball = chase_ball,
}


