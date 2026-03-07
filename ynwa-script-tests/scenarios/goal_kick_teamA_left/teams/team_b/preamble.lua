-- Scenario: goal_kick_teamA_left
-- Team B: два игрока.
-- Игрок 1 (исполнитель): player_setup → идёт к мячу.
-- Игрок 2 (поддержка): player_setup → default_goal_kick_setup → goal_kick_own_position.

team_play = {
    i_have_ball = function()
        return kick_to_region("M20", "N21")
    end,
    team_has_ball     = function() return {action = "stop"} end,
    ball_is_free      = function() return {action = "stop"} end,
    opponent_has_ball = function() return {action = "stop"} end,
}

team_setup = {
    start = run_to_start_position,
}
