-- Scenario: goal_kick_teamB_right
-- Team B бьёт правее ворот A (col V, row 40 в системе B) → мяч за лицевой → удар от ворот для Team A.

team_play = {
    i_have_ball = function()
        return kick_to_cell("V40")
    end,
    team_has_ball     = chase_ball,
    ball_is_free      = chase_ball,
    opponent_has_ball = chase_ball,
}

team_setup = {
    start     = run_to_start_position,
    goal_kick = default_goal_kick_setup,
}
