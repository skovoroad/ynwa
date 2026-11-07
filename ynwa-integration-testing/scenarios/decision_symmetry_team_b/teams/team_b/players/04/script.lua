player_play = {
    i_have_ball       = function() return {action="kick", target={x=10.0, z=20.0}, reason="symmetry:kick"} end,
    ball_is_free      = function() return {action="kick", target={x=10.0, z=20.0}, reason="symmetry:kick"} end,
    team_has_ball     = function() return {action="kick", target={x=10.0, z=20.0}, reason="symmetry:kick"} end,
    opponent_has_ball = function() return {action="kick", target={x=10.0, z=20.0}, reason="symmetry:kick"} end,
}
