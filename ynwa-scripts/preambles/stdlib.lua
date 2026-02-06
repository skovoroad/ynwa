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
