-- Minimal team preamble for script tests.
-- Provides simple dispatch tables without real game tactics.

team_play = {
    i_have_ball       = function() return {action = "stop"} end,
    ball_is_free      = function() return run_to_start_position() end,
    team_has_ball     = function() return run_to_attack_position() end,
    opponent_has_ball = function() return run_to_defence_position() end,
}
