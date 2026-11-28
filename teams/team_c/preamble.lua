-- Team C: target tactic roles implemented through the public stdlib API

local function am_i_top3_closest_to_ball()
    local ball = ball_position()
    return count_teammates_around(ball, distance(my_position(), ball)) < 3
end

local function press_ball_or_defend()
    if am_i_top3_closest_to_ball() then
        return chase_ball()
    end
    return run_to_defence_position()
end

team_play = {
    ball_is_free      = press_ball_or_defend,
    team_has_ball     = run_to_attack_position,
    opponent_has_ball = press_ball_or_defend,
}

goalkeeper_play = {
    i_have_ball       = default_goalkeeper_with_ball,
    team_has_ball     = default_goalkeeper_cover_position,
    opponent_has_ball = default_goalkeeper_cover_position,
    ball_is_free      = default_goalkeeper_cover_position,
}

local function nearest_teammate_number(numbers)
    -- Intentionally written as a loop to match the future DSL design:
    -- `for player in teammates <numbers> order by distance to me ascending`.
    local my_pos = my_position()
    local ordered = {}
    for i, n in ipairs(numbers) do
        ordered[i] = {number = n, key = distance(my_pos, teammate_position(n)), idx = i}
    end
    table.sort(ordered, function(a, b)
        if a.key ~= b.key then return a.key < b.key end
        return a.idx < b.idx
    end)
    return ordered[1].number
end

local function pass_to_free_or_nearest(numbers)
    -- Intentionally written as loops to match the future DSL design: the priority-ordered
    -- `for player in teammates <numbers> [ if player free [ decision: pass to player ] ]`,
    -- then the same loop ordered by distance to me ascending.
    for _, n in ipairs(numbers) do
        if is_teammate_free(n) then
            return pass_to(n)
        end
    end

    return pass_to(nearest_teammate_number(numbers))
end

function fullback_with_ball()
    if is_in_opponent_penalty_area() then
        return pass_to_free_or_nearest({9, 10, 7, 11})
    end
    return run_to_opponent_penalty_area()
end

function center_back_with_ball()
    -- Intentionally written as a loop to match the future DSL design:
    -- `for player in teammates 6, 8, 7, 11 order by distance to me ascending
    --  [ if player free and player ahead [ decision: pass to player ] ]`.
    local my_pos = my_position()
    local ordered = {}
    local count = 0
    for i, n in ipairs({6, 8, 7, 11}) do
        if is_teammate_free(n) and is_teammate_ahead(n) then
            count = count + 1
            ordered[count] = {number = n, key = distance(my_pos, teammate_position(n)), idx = i}
        end
    end
    if count == 0 then
        return run_to_attack_position()
    end
    table.sort(ordered, function(a, b)
        if a.key ~= b.key then return a.key < b.key end
        return a.idx < b.idx
    end)
    return pass_to(ordered[1].number)
end

local function pass_to_first_free_ahead(numbers)
    -- Intentionally written as a loop to match the future DSL design:
    -- `for player in teammates <numbers> [ if player free and player ahead [ decision: pass to player ] ]`.
    for _, n in ipairs(numbers) do
        if is_teammate_free(n) and is_teammate_ahead(n) then
            return pass_to(n)
        end
    end
    return nil
end

function wide_midfielder_with_ball()
    local pass = pass_to_first_free_ahead({9, 10})
    if pass ~= nil then
        return pass
    end
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function central_midfielder_with_ball()
    local pass = pass_to_first_free_ahead({9, 10})
    if pass == nil then
        pass = pass_to_first_free_ahead({7, 11})
    end
    if pass ~= nil then
        return pass
    end
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function forward_with_ball()
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end
