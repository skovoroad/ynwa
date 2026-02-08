-- Standard library: common utilities for all teams
-- Uses functions from core preamble

-- Returns true if I am the ball owner, false otherwise
function am_i_ball_owner()
    local owner = ball_owner()
    if owner == nil then
        return false
    end
    return my_index() == owner
end

-- Calculate distance between two positions (2D, ignoring Y)
function distance(pos1, pos2)
    local dx = pos1.x - pos2.x
    local dz = pos1.z - pos2.z
    return math.sqrt(dx * dx + dz * dz)
end

-- Returns true if the ball is owned by any player from my team
function is_ball_owned_by_my_team()
    local owner = ball_owner()
    if owner == nil then
        return false
    end
    
    -- Check if I own it
    if owner == my_index() then
        return true
    end
    
    -- Check if any teammate owns it
    for _, teammate in ipairs(get_teammates()) do
        if teammate.index == owner then
            return true
        end
    end
    
    return false
end

-- Find N nearest teammates to me, sorted by distance (closest first)
-- Returns array of {teammate, distance}
-- Find nearest opponent to me
-- Returns {opponent, distance} or {nil, math.huge} if no opponents
function find_nearest_opponent()
    local my_pos = my_position()
    local opponents = get_opponents()
    local nearest = nil
    local min_dist = math.huge
    
    for _, opp in ipairs(opponents) do
        local dist = distance(my_pos, opp.position)
        if dist < min_dist then
            min_dist = dist
            nearest = opp
        end
    end
    
    return {opponent = nearest, distance = min_dist}
end

-- Check if I am the closest teammate to the ball
function am_i_closest_teammate_to_ball()
    local ball_pos = ball_position()
    local my_pos = my_position()
    local my_dist = distance(my_pos, ball_pos)
    
    for _, tm in ipairs(get_teammates()) do
        if distance(tm.position, ball_pos) < my_dist then
            return false
        end
    end
    
    return true
end

-- Default prepare function for setup stage
-- Runs to start position if not there yet, stops when arrived
function prepare(reason)
    local my_pos = my_position()
    local start_pos = my_regions()["start position"]
    
    if start_pos == nil then
        -- No start position defined, just stop
        return {action = "stop"}
    end
    
    -- Check if we're already in start position (roughly)
    local center_x = (start_pos.min_x + start_pos.max_x) / 2
    local center_z = (start_pos.min_z + start_pos.max_z) / 2
    local dist = math.sqrt((my_pos.x - center_x)^2 + (my_pos.z - center_z)^2)
    
    -- If we're close to start position (within 1 meter), stop
    if dist < 1.0 then
        return {action = "stop"}
    end
    
    -- Otherwise, run to start position center
    return {
        action = "run",
        target_type = "point",
        target = {x = center_x, z = center_z, y = 0}
    }
end

-- Common behavior v2: improved tactical logic
-- 1. If I own the ball -> pass to nearest teammate (not closer than 15m)
-- 2. If I don't own the ball:
--    a) If I'm in top 3 closest teammates to ball -> run to ball
--    b) Otherwise:
--       - If my team owns ball -> run to attack position
--       - If opponent owns ball -> run to defence position
function common_behavior_v2()
    local ball_pos = ball_position()
    local my_pos = my_position()
    
    if am_i_ball_owner() then
        -- Find nearest teammate that is at least 15 meters away
        local teammates = get_teammates()
        local best_teammate = nil
        local best_distance = math.huge
        local MIN_PASS_DISTANCE = 15.0  -- minimum 15 meters
        
        for _, tm in ipairs(teammates) do
            local dist = distance(my_pos, tm.position)
            if dist >= MIN_PASS_DISTANCE and dist < best_distance then
                best_distance = dist
                best_teammate = tm
            end
        end
        
        if best_teammate then
            -- Pass to the teammate
            return {
                action = "kick",
                target = {x = best_teammate.position.x, z = best_teammate.position.z}
            }
        else
            -- No suitable teammate found, kick in random direction as fallback
            local target_x = math.random() * FIELD_WIDTH
            local target_z = math.random() * FIELD_LENGTH
            return {
                action = "kick",
                target = {x = target_x, z = target_z}
            }
        end
    end
    
    -- I don't own the ball
    -- Check if I'm in top 3 closest teammates to ball
    local my_dist_to_ball = distance(my_pos, ball_pos)
    local closer_count = 0
    
    for _, tm in ipairs(get_teammates()) do
        if distance(tm.position, ball_pos) < my_dist_to_ball then
            closer_count = closer_count + 1
        end
    end
    
    -- If less than 3 teammates are closer, I'm in top 3
    if closer_count < 3 then
        -- Run to the ball
        return {
            action = "run",
            target_type = "point",
            target = {x = ball_pos.x, z = ball_pos.z, y = 0}
        }
    end
    
    -- I'm not in top 3, decide based on ball ownership
    -- Use owner_team from context to determine ball possession
    local owner_team = get_ball_owner_team()
    local my_team = my_team_name()
    local regions = my_regions()
    
    if owner_team == my_team then
        -- My team has the ball -> run to attack position
        local attack_pos = regions["attack position"]
        if attack_pos then
            local center_x = (attack_pos.min_x + attack_pos.max_x) / 2
            local center_z = (attack_pos.min_z + attack_pos.max_z) / 2
            return {
                action = "run",
                target_type = "point",
                target = {x = center_x, z = center_z, y = 0}
            }
        end
    else
        -- Opponent has the ball (or neutral) -> run to defence position
        local defence_pos = regions["defence position"]
        if defence_pos then
            local center_x = (defence_pos.min_x + defence_pos.max_x) / 2
            local center_z = (defence_pos.min_z + defence_pos.max_z) / 2
            return {
                action = "run",
                target_type = "point",
                target = {x = center_x, z = center_z, y = 0}
            }
        end
    end
    
    -- Fallback: stop
    return {action = "stop"}
end


