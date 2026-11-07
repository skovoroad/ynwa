player_play = {
    i_have_ball       = function() return {action="run", target_type="region", target={from="A1", to="B2"}, reason="symmetry:region"} end,
    ball_is_free      = function() return {action="run", target_type="region", target={from="A1", to="B2"}, reason="symmetry:region"} end,
    team_has_ball     = function() return {action="run", target_type="region", target={from="A1", to="B2"}, reason="symmetry:region"} end,
    opponent_has_ball = function() return {action="run", target_type="region", target={from="A1", to="B2"}, reason="symmetry:region"} end,
}
