team_play = {
    i_have_ball       = function() return { action = "kick", target = { x = 34.0, z = 93.61538 }, reason = "corner_cross" } end,
    ball_is_free      = function() return stop("hold") end,
    team_has_ball     = function() return stop("hold") end,
    opponent_has_ball = function() return stop("hold") end,
}
