-- Team B: dispatch tables for play and setup stages

-- Returns true if this player is among the 3 closest teammates to the ball
local function am_i_top3_closest_to_ball()
    local my_dist = distance(my_position(), ball_position())
    local closer = 0
    for _, tm in ipairs(get_teammates()) do
        if distance(tm.position, ball_position()) < my_dist then
            closer = closer + 1
        end
    end
    return closer < 3
end

-- Action: chase ball if in top-3 closest, otherwise run to attack position
local function press_or_attack()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_attack_position()
end

-- Action: chase ball if in top-3 closest, otherwise run to defence position
local function press_or_defend()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_defence_position()
end

-- Action: pass to nearest teammate at least 15m away; kick toward opponent goal if none found
local function pass_to_nearest_teammate()
    local my_pos = my_position()
    local best = nil
    local best_dist = math.huge
    for _, tm in ipairs(get_teammates()) do
        local d = distance(my_pos, tm.position)
        if d >= 15.0 and d < best_dist then
            best_dist = d
            best = tm
        end
    end
    if best then
        return {
            action = "kick",
            target = {x = best.position.x, z = best.position.z},
            reason = "pass_to_#" .. best.number
        }
    end
    return kick_to_opponent_goal()
end

-- Assign via player_play in script.lua: player_play = goalkeeper_play
goalkeeper_play = {
    i_have_ball       = function() return pass_to_nearest_teammate() end,
    team_has_ball     = function() return run_to_defence_position() end,
    opponent_has_ball = function() return default_goalkeeper_cover_position() end,
}

function nonforward_with_ball()
    local fwd10 = get_teammate_by_number(10)
    local fwd11 = get_teammate_by_number(11)
    local my_z  = my_position().z

    -- If a forward is ahead of me, pass to them
    if fwd10 and fwd10.position.z > my_z then return pass_to_teammate(fwd10) end
    if fwd11 and fwd11.position.z > my_z then return pass_to_teammate(fwd11) end

    -- I'm ahead: push into penalty area or shoot
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function forward_with_ball()
    -- Highest priority: already in opponent penalty area → shoot immediately
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    -- In attack position → shoot
    local pos = my_regions()["attack position"]
    if pos and is_in_region_obj(pos) then
        return kick_to_opponent_goal()
    end
    -- Otherwise: run to attack position
    return run_to_attack_position()
end

team_play = {
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}

team_setup = {
    start      = run_to_start_position,
    after_goal = run_to_start_position,
    throw_in   = run_to_start_position,
    goal_kick  = run_to_start_position,
    corner     = run_to_start_position,
}
