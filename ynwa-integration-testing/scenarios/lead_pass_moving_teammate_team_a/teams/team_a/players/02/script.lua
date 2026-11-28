local RUN_TARGET = {x = 34, z = 95}

player_play = {
    i_have_ball       = function() return stop("secured") end,
    ball_is_free      = function() return {action = "run", target_type = "point", target = RUN_TARGET} end,
    team_has_ball     = function() return {action = "run", target_type = "point", target = RUN_TARGET} end,
    opponent_has_ball = function() return {action = "run", target_type = "point", target = RUN_TARGET} end,
}
