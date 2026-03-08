-- Spy script for testing the dispatch mechanism.
-- Overrides all team_play and team_setup handlers to return tagged Stop/Run decisions.
-- Tests inspect the `reason` field to verify which handler was invoked.

team_play = {
    i_have_ball       = function() return {action = "stop", reason = "spy:i_have_ball"} end,
    ball_is_free      = function() return {action = "stop", reason = "spy:ball_is_free"} end,
    team_has_ball     = function() return {action = "stop", reason = "spy:team_has_ball"} end,
    opponent_has_ball = function() return {action = "stop", reason = "spy:opponent_has_ball"} end,
}

team_setup = {
    ["kick off"] = function() return {action = "stop", reason = "spy:setup_kick_off"} end,
}
