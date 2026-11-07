player_play = {
    i_have_ball       = function() return {action="run", target_type="point", target={x=10.0, z=20.0}, reason="symmetry:point"} end,
    ball_is_free      = function() return {action="run", target_type="point", target={x=10.0, z=20.0}, reason="symmetry:point"} end,
    team_has_ball     = function() return {action="run", target_type="point", target={x=10.0, z=20.0}, reason="symmetry:point"} end,
    opponent_has_ball = function() return {action="run", target_type="point", target={x=10.0, z=20.0}, reason="symmetry:point"} end,
}
