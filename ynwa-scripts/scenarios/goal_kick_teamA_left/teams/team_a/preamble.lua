-- Scenario: goal_kick_teamA_left
-- Team A: один игрок, берёт мяч, ведёт его к левому флангу ворот B,
-- бьёт мимо (low shot_accuracy гарантирует промах в сторону).
-- В фазе возобновления (goal_kick): возвращается на стартовую позицию,
-- но встаёт в 25м от центра (строка 10 в собственных координатах).

-- В игре: всегда бежать к мячу и бить в ворота
team_play = {
    i_have_ball = function()
        return kick_to_opponent_goal()
    end,
    team_has_ball = function()
        return chase_ball()
    end,
    ball_is_free = function()
        return chase_ball()
    end,
    opponent_has_ball = function()
        return chase_ball()
    end,
}

-- При возобновлении игры (goal_kick): встать на 25м от центра к своей половине.
-- Строка 10 в координатах Team A (z ≈ 26м, центр поля на строке 20).
team_setup = {
    start      = function() return run_to_region("N16", "N16") end,
    after_goal = function() return run_to_region("N16", "N16") end,
    throw_in   = function() return run_to_region("N10", "N10") end,
    goal_kick  = function() return run_to_region("N10", "N10") end,
    corner     = function() return run_to_region("N10", "N10") end,
}
