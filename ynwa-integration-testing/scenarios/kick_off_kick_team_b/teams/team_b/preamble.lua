team_play = {
    i_have_ball       = function() return kick_to_opponent_goal() end,
    ball_is_free      = function() return stop("hold") end,
    team_has_ball     = function() return stop("hold") end,
    opponent_has_ball = function() return stop("hold") end,
}
