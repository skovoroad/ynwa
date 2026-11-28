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

local function pass_to_free_or_nearest(numbers)
    -- Intentionally written as loops to match the future DSL design: the priority-ordered
    -- `for player in teammates <numbers> [ if player free [ decision: pass to player ] ]`,
    -- then the same loop ordered by distance to me ascending.
    for player in each(numbers) do
        if is_teammate_free(player) then
            return pass_to(player)
        end
    end

    for player in order_by_distance_asc(numbers) do
        return pass_to(player)
    end
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
    for player in order_by_distance_asc({6, 8, 7, 11}) do
        if is_teammate_free(player) and is_teammate_ahead(player) then
            return pass_to(player)
        end
    end
    return run_to_attack_position()
end

local function pass_to_first_free_ahead(numbers)
    -- Intentionally written as a loop to match the future DSL design:
    -- `for player in teammates <numbers> [ if player free and player ahead [ decision: pass to player ] ]`.
    for player in each(numbers) do
        if is_teammate_free(player) and is_teammate_ahead(player) then
            return pass_to(player)
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
