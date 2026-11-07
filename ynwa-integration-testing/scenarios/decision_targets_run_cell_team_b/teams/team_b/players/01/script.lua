player_play = {
    i_have_ball       = function() return {action="run", target_type="cell", target="A1", reason="target:cell"} end,
    ball_is_free      = function() return {action="run", target_type="cell", target="A1", reason="target:cell"} end,
    team_has_ball     = function() return {action="run", target_type="cell", target="A1", reason="target:cell"} end,
    opponent_has_ball = function() return {action="run", target_type="cell", target="A1", reason="target:cell"} end,
}
