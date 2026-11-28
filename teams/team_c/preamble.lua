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

local function nearest_teammate_number(n1, n2, n3, n4)
    local my_pos = my_position()
    local best_number = n1
    local best_distance = distance(my_pos, teammate_position(n1))

    local d2 = distance(my_pos, teammate_position(n2))
    if d2 < best_distance then
        best_number, best_distance = n2, d2
    end

    local d3 = distance(my_pos, teammate_position(n3))
    if d3 < best_distance then
        best_number, best_distance = n3, d3
    end

    local d4 = distance(my_pos, teammate_position(n4))
    if d4 < best_distance then
        best_number = n4
    end

    return best_number
end

local function pass_to_free_or_nearest(n1, n2, n3, n4)
    if is_teammate_free(n1) then
        return pass_to(n1)
    end
    if is_teammate_free(n2) then
        return pass_to(n2)
    end
    if is_teammate_free(n3) then
        return pass_to(n3)
    end
    if is_teammate_free(n4) then
        return pass_to(n4)
    end

    local number = nearest_teammate_number(n1, n2, n3, n4)
    return pass_to(number)
end

function fullback_with_ball()
    if is_in_opponent_penalty_area() then
        return pass_to_free_or_nearest(9, 10, 7, 11)
    end
    return run_to_opponent_penalty_area()
end

function center_back_with_ball()
    local my_pos = my_position()
    local best_number = nil
    local best_distance = nil

    if is_teammate_free(6) and is_teammate_ahead(6) then
        best_number = 6
        best_distance = distance(my_pos, teammate_position(6))
    end

    if is_teammate_free(8) and is_teammate_ahead(8) then
        local candidate_distance = distance(my_pos, teammate_position(8))
        if best_number == nil or candidate_distance < best_distance then
            best_number = 8
            best_distance = candidate_distance
        end
    end

    if is_teammate_free(7) and is_teammate_ahead(7) then
        local candidate_distance = distance(my_pos, teammate_position(7))
        if best_number == nil or candidate_distance < best_distance then
            best_number = 7
            best_distance = candidate_distance
        end
    end

    if is_teammate_free(11) and is_teammate_ahead(11) then
        local candidate_distance = distance(my_pos, teammate_position(11))
        if best_number == nil or candidate_distance < best_distance then
            best_number = 11
        end
    end

    if best_number ~= nil then
        return pass_to(best_number)
    end
    return run_to_attack_position()
end

local function pass_to_first_free_ahead(n1, n2)
    if is_teammate_free(n1) and is_teammate_ahead(n1) then
        return pass_to(n1)
    end
    if is_teammate_free(n2) and is_teammate_ahead(n2) then
        return pass_to(n2)
    end
    return nil
end

function wide_midfielder_with_ball()
    local pass = pass_to_first_free_ahead(9, 10)
    if pass ~= nil then
        return pass
    end
    if is_in_opponent_penalty_area() then
        return kick_to_opponent_goal()
    end
    return run_to_opponent_penalty_area()
end

function central_midfielder_with_ball()
    local pass = pass_to_first_free_ahead(9, 10)
    if pass == nil then
        pass = pass_to_first_free_ahead(7, 11)
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
