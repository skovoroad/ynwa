-- Scenario: goal_kick_teamA_left
-- Team B: один игрок, стоит на стартовой позиции.
-- В фазе возобновления (goal_kick): team B restarting → идёт к restart_position (к мячу).
-- Если не restarting — встаёт в 25м от центра к своей половине (строка 10 в их координатах).

-- В игре: стоять на месте, но если мяч достался — пинать в центр поля
team_play = {
    i_have_ball = function()
        local cx = GAME_DATA.field.width / 2
        local cz = GAME_DATA.field.length / 2
        return {action = "kick", target = {x = cx, z = cz, y = 0}, reason = "kick_to_center"}
    end,
    team_has_ball     = function() return {action = "stop"} end,
    ball_is_free      = function() return {action = "stop"} end,
    opponent_has_ball = function() return {action = "stop"} end,
}

-- При возобновлении: если наша команда бьёт — идти к мячу; иначе — в 25м от центра.
local function goal_kick_setup()
    if is_my_team_restarting() then
        -- Наш удар: идти к позиции мяча
        local rp = get_restart_position()
        if rp then
            return {action = "run", target_type = "point", target = {x = rp.x, z = rp.z, y = 0}}
        end
    end
    -- Не наш удар: отойти на 25м от центра в свою половину
    -- Строка 10 в координатах Team B (их z=0 у ворот B в абсолютных координатах)
    return run_to_region("N10", "N10")
end

team_setup = {
    start      = function() return run_to_region("N16", "N16") end,
    after_goal = function() return run_to_region("N16", "N16") end,
    throw_in   = goal_kick_setup,
    goal_kick  = goal_kick_setup,
    corner     = goal_kick_setup,
}
