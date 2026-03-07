-- Scenario: goal_kick_teamA_left
-- Team A бьёт левее ворот B (col E, row 40) → мяч за лицевой → удар от ворот для Team B.

team_play = {
    i_have_ball = function()
        return kick_to_cell("E40")
    end,
    team_has_ball     = chase_ball,
    ball_is_free      = chase_ball,
    opponent_has_ball = chase_ball,
}

team_setup = {
    start     = run_to_start_position,
    goal_kick = default_goal_kick_setup,
}
