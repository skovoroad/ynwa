player_play = {
    i_have_ball       = function() return stop("secured") end,
    ball_is_free      = function() return chase_ball() end,
    team_has_ball     = function() return chase_ball() end,
    opponent_has_ball = function() return chase_ball() end,
}
