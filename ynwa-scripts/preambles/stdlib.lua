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

-- Returns opponent team name ("A" or "B")
function opponent_team()
    return my_team_name() == "A" and "B" or "A"
end

-- Check if a point is in a specific zone
-- zone_name: name of the zone (e.g., "penalty_area_a", "goal_area_b")
-- x, z: coordinates to check (optional, defaults to my position)
function is_in_zone(zone_name, x, z)
    local pos_x = x or my_position().x
    local pos_z = z or my_position().z
    
    local zone = GAME_DATA.zones[zone_name]
    if not zone then
        return false
    end
    
    if zone.type == "rectangle" then
        return is_point_in_rectangle(pos_x, pos_z, zone)
    elseif zone.type == "circle" then
        return is_point_in_circle(pos_x, pos_z, zone)
    elseif zone.type == "arc" then
        return is_point_in_arc(pos_x, pos_z, zone)
    end
    
    return false
end

-- Check if I am in opponent's penalty area
function am_i_in_opponent_penalty_area()
    local opponent = opponent_team()
    local zone_name = "penalty_area_" .. string.lower(opponent)
    return is_in_zone(zone_name)
end

-- Returns a random point on the goal line for shooting
-- The point is on the front line of the goal (not deep inside)
-- opponent_team: "A" or "B" (case-insensitive)
function get_random_shot_target_to_goal(opponent_team)
    local team_lower = string.lower(opponent_team)
    
    local goal_zone
    local goal_x
    
    if team_lower == "a" then
        goal_zone = GAME_DATA.zones["goal_a"]
        if not goal_zone or goal_zone.type ~= "rectangle" then
            return nil
        end
        goal_x = goal_zone.max_x  -- Front of Team A goal (toward field)
    else
        goal_zone = GAME_DATA.zones["goal_b"]
        if not goal_zone or goal_zone.type ~= "rectangle" then
            return nil
        end
        goal_x = goal_zone.min_x  -- Front of Team B goal (toward field)
    end
    
    -- Random point along the width of the goal (z-axis)
    local goal_width = goal_zone.max_z - goal_zone.min_z
    local random_offset = math.random() * goal_width
    local goal_z = goal_zone.min_z + random_offset
    
    return {x = goal_x, z = goal_z, y = 0}
end

-- Default get_setup_position for setup stage
-- Runs to start position if not there yet, stops when arrived
function get_setup_position(reason)
    local my_pos = my_position()
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
        -- Run to the ball
        return {
            action = "run",
            target_type = "point",
            target = {x = ball_pos.x, z = ball_pos.z, y = 0},
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
