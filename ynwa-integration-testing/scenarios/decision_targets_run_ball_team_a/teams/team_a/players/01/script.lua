player_play = {
    i_have_ball       = function() return {action="run", target_type="ball", reason="target:ball"} end,
    ball_is_free      = function() return {action="run", target_type="ball", reason="target:ball"} end,
    team_has_ball     = function() return {action="run", target_type="ball", reason="target:ball"} end,
    opponent_has_ball = function() return {action="run", target_type="ball", reason="target:ball"} end,
}
