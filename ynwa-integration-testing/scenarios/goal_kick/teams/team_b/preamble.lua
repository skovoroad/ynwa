-- Players hold position; out-of-play scenarios drive the ball with an initial velocity.
team_play = {
    i_have_ball       = function() return stop("hold") end,
    ball_is_free      = function() return stop("hold") end,
    team_has_ball     = function() return stop("hold") end,
    opponent_has_ball = function() return stop("hold") end,
}
