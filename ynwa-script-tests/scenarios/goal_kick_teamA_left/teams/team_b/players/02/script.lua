player_play = {
    i_have_ball       = function() return kick_to_region("M20", "N21") end,
    team_has_ball     = function() return run_to_region("K40", "K40") end,
    ball_is_free      = function() return run_to_region("K40", "K40") end,
    opponent_has_ball = function() return run_to_region("K40", "K40") end,
}
