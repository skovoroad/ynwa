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

-- Default get_setup_position for setup stage
-- Runs to center of "start position" region.
-- Stopping is handled by the engine automatically once the player is within 0.5m of the target.
function get_setup_position(reason)
    local start_pos = my_regions()["start position"]

    if start_pos == nil then
        return {action = "stop"}
    end

    local center_x = (start_pos.min_x + start_pos.max_x) / 2
    local center_z = (start_pos.min_z + start_pos.max_z) / 2

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
                target = {x = best_teammate.position.x, z = best_teammate.position.z},
                reason = "Passing to #" .. best_teammate.index .. ", distance=" .. string.format("%.2f", best_distance)
            }
        else
            -- No suitable teammate found, kick in random direction as fallback
            local target_x = math.random() * GAME_DATA.field.width
            local target_z = math.random() * GAME_DATA.field.length
            return {
                action = "kick",
                target = {x = target_x, z = target_z},
                reason = "No teammate >=15m away, kicking randomly"
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
        -- Chase the ball: ActionSystem resolves the target to the ball's current position
        -- at the moment of processing; arrival check in DecisionSystem tests against
        -- the live ball position each tick, so the player stops when they actually reach it.
        return {
            action = "run",
            target_type = "ball",
            reason = "Chasing ball, rank=" .. (closer_count + 1) .. ", dist=" .. string.format("%.2f", my_dist_to_ball)
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
                target = {x = center_x, z = center_z, y = 0},
                reason = "My team owns ball, moving to attack position"
            }
        end
    else
        -- Opponent has the ball (or neutral) -> run to defence position
        local defence_pos = regions["defence position"]
        if defence_pos then
            local center_x = (defence_pos.min_x + defence_pos.max_x) / 2
            local centre_z = (defence_pos.min_z + defence_pos.max_z) / 2
            return {
                action = "run",
                target_type = "point",
                target = {x = center_x, z = centre_z, y = 0},
                reason = "Opponent owns ball (owner_team=" .. tostring(owner_team) .. "), moving to defence position"
            }
        end
    end
    
    -- Fallback: stop
    return {action = "stop", reason = "No valid position found, stopping"}
end
