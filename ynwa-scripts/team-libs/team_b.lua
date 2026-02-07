-- Team B specific functions

-- Square game behavior: tactical decision making
function square_game_behavior()
    if am_i_ball_owner() then
        -- Scenario 1: I have the ball
        -- Find 3 nearest teammates to me
        local nearest_teammates = find_n_nearest_teammates(3)
        
        if #nearest_teammates == 0 then
            -- No teammates available, just stop
            return stop()
        end
        
        -- Find which of these 3 is farthest from opponents
        local best_teammate = find_farthest_from_opponents(nearest_teammates)
        
        if best_teammate then
            -- Kick to the best teammate
            local target = best_teammate.position
            return kick_to(target.x, target.z)
        else
            -- Fallback: stop
            return stop()
        end
        
    elseif is_ball_owned_by_my_team() then
        -- Scenario 3: Ball is with my team (but not me)
        -- Just run in random direction
        return run_to_random_position()
        
    else
        -- Scenario 2: Ball is free
        -- Check if I'm among 3 nearest teammates to ball
        local ball_pos = ball_position()
        local my_pos = my_position()
        local my_dist_to_ball = distance(my_pos, ball_pos)
        
        -- Count how many teammates are closer to ball than me
        local closer_count = 0
        for _, tm in ipairs(get_teammates()) do
            if distance(tm.position, ball_pos) < my_dist_to_ball then
                closer_count = closer_count + 1
            end
        end
        
        -- If less than 3 teammates are closer, I'm among top 3
        if closer_count < 3 then
            -- Run to the ball
            return run_to_point(ball_pos.x, ball_pos.z)
        else
            -- I'm not in top 3, run randomly
            return run_to_random_position()
        end
    end
end

function find_n_nearest_teammates(n)
    local my_pos = my_position()
    local teammates = get_teammates()
    local distances = {}
    
    -- Calculate distances to all teammates
    for _, tm in ipairs(teammates) do
        local dist = distance(my_pos, tm.position)
        table.insert(distances, {teammate = tm, distance = dist})
    end
    
    -- Sort by distance
    table.sort(distances, function(a, b) return a.distance < b.distance end)
    
    -- Return first N
    local result = {}
    for i = 1, math.min(n, #distances) do
        table.insert(result, distances[i])
    end
    
    return result
end

-- Find which teammate from the list is farthest from all opponents
-- Returns the teammate who has the maximum distance to their nearest opponent
function find_farthest_from_opponents(teammate_list)
    local opponents = get_opponents()
    local best_teammate = nil
    local best_min_distance = -1
    
    for _, tm_data in ipairs(teammate_list) do
        local tm_pos = tm_data.teammate.position
        
        -- Find minimum distance from this teammate to any opponent
        local min_dist_to_opponent = math.huge
        for _, opp in ipairs(opponents) do
            local dist = distance(tm_pos, opp.position)
            if dist < min_dist_to_opponent then
                min_dist_to_opponent = dist
            end
        end
        
        -- Keep the teammate with the maximum of these minimum distances
        if min_dist_to_opponent > best_min_distance then
            best_min_distance = min_dist_to_opponent
            best_teammate = tm_data.teammate
        end
    end
    
    return best_teammate
end

-- Calculate a position away from a given position
-- Returns {x, z} that is 'away_distance' meters away from 'from_pos'
function calculate_away_position(from_pos, away_distance)
    local my_pos = my_position()
    
    -- Calculate direction vector from opponent to me
    local dx = my_pos.x - from_pos.x
    local dz = my_pos.z - from_pos.z
    
    -- Normalize the vector
    local length = math.sqrt(dx * dx + dz * dz)
    if length < 0.01 then
        -- If too close, pick a random direction
        dx = math.random() - 0.5
        dz = math.random() - 0.5
        length = math.sqrt(dx * dx + dz * dz)
    end
    
    dx = dx / length
    dz = dz / length
    
    -- Calculate target position
    local target_x = my_pos.x + dx * away_distance
    local target_z = my_pos.z + dz * away_distance
    
    -- Clamp to field boundaries (with small margin)
    local margin = 2.0
    target_x = math.max(margin, math.min(FIELD_LENGTH - margin, target_x))
    target_z = math.max(margin, math.min(FIELD_WIDTH - margin, target_z))
    
    return {x = target_x, z = target_z}
end

-- Calculate a position away from opponent, but not farther than max_distance from ball
-- Returns {x, z} that is away from 'from_pos' but within 'max_ball_distance' of ball
function calculate_away_position_near_ball(from_pos, away_distance, max_ball_distance)
    local my_pos = my_position()
    local ball_pos = ball_position()
    
    -- Calculate direction vector from opponent to me
    local dx = my_pos.x - from_pos.x
    local dz = my_pos.z - from_pos.z
    
    -- Normalize the vector
    local length = math.sqrt(dx * dx + dz * dz)
    if length < 0.01 then
        -- If too close, pick a random direction
        dx = math.random() - 0.5
        dz = math.random() - 0.5
        length = math.sqrt(dx * dx + dz * dz)
    end
    
    dx = dx / length
    dz = dz / length
    
    -- Calculate target position
    local target_x = my_pos.x + dx * away_distance
    local target_z = my_pos.z + dz * away_distance
    
    -- Check distance from ball
    local dist_to_ball = math.sqrt((target_x - ball_pos.x)^2 + (target_z - ball_pos.z)^2)
    
    -- If too far from ball, adjust position to be on the circle around ball
    if dist_to_ball > max_ball_distance then
        -- Direction from ball to target
        local ball_dx = target_x - ball_pos.x
        local ball_dz = target_z - ball_pos.z
        local ball_length = math.sqrt(ball_dx * ball_dx + ball_dz * ball_dz)
        
        if ball_length > 0.01 then
            ball_dx = ball_dx / ball_length
            ball_dz = ball_dz / ball_length
            
            -- Place target on the circle edge
            target_x = ball_pos.x + ball_dx * max_ball_distance
            target_z = ball_pos.z + ball_dz * max_ball_distance
        end
    end
    
    -- Clamp to field boundaries (with small margin)
    local margin = 2.0
    target_x = math.max(margin, math.min(FIELD_LENGTH - margin, target_x))
    target_z = math.max(margin, math.min(FIELD_WIDTH - margin, target_z))
    
    return {x = target_x, z = target_z}
end


function common_behavior()
    if am_i_ball_owner() then
        -- I have the ball, kick it in random direction
        local target_x = math.random() * FIELD_LENGTH
        local target_z = math.random() * FIELD_WIDTH
        return {
            action = "kick",
            target = {x = target_x, z = target_z}
        }
    elseif is_ball_owned_by_my_team() then
        return {
            action = "run",
            target_type = "region",
            target = {from = "K19", to = "R27"}
        }
    else
        local ball_pos = ball_position()
        local my_pos = my_position()
        local my_dist_to_ball = distance(my_pos, ball_pos)
        
        -- Count how many teammates are closer to ball than me
        local closer_count = 0
        for _, tm in ipairs(get_teammates()) do
            if distance(tm.position, ball_pos) < my_dist_to_ball then
                closer_count = closer_count + 1
            end
        end
        
        -- If less than 3 teammates are closer, I'm among top 3
        if closer_count < 3 then
            -- Run to the ball
            return run_to_point(ball_pos.x, ball_pos.z)
        else
            -- I'm not in top 3, run randomly
            return run_to_random_position()
        end
    end
end