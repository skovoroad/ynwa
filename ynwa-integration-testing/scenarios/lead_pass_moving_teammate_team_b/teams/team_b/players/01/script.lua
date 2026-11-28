-- Hold the ball for the first decision so the receiver is already moving when the pass is taken.
decision_count = (decision_count or 0) + 1

player_play = {
    i_have_ball = function()
        if decision_count == 1 then
            return stop("hold")
        end
        return pass_to(2)
    end,
    ball_is_free      = function() return stop("wait") end,
    team_has_ball     = function() return stop("wait") end,
    opponent_has_ball = function() return stop("wait") end,
}
