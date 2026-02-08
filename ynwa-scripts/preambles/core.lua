-- Core preamble: elementary functions for reading game state and creating decisions
-- These functions provide access to context data without any game logic

-- Field dimensions (standard football)
-- NOTE: X-axis is field WIDTH (68m), Z-axis is field LENGTH (105m)
-- This matches the orientation.rs flip_point_orientation function
FIELD_WIDTH = 68.0    -- meters (X axis)
FIELD_LENGTH = 105.0  -- meters (Z axis)

-- Returns the global index of the ball owner, or nil if ball is free
function ball_owner()
    return context.ball.owner_index
end

-- Returns my player number in the team (jersey number)
function my_number()
    return context.me.number
end

-- Returns my position as a table {x, y, z}
function my_position()
    return context.me.position
end

-- Returns my regions as a table {region_name = {min_x, max_x, min_z, max_z}}
function my_regions()
    return context.me.regions
end

-- Returns ball position as a table {x, y, z}
function ball_position()
    return context.ball.position
end

-- Returns array of teammates (each has: index, number, position)
function get_teammates()
    return context.teammates
end

-- Returns array of opponents (each has: index, number, position)
function get_opponents()
    return context.opponents
end

-- Returns my global player index
function my_index()
    return context.me.index
end

-- Factory: create "kick" decision
function kick_to(x, z, y)
    return {
        action = "kick",
        target = {x = x, z = z, y = y or 0}
    }
end

-- Factory: create "run to point" decision
function run_to_point(x, z, y)
    return {
        action = "run",
        target_type = "point",
        target = {x = x, z = z, y = y or 0}
    }
end

-- Factory: create "stop" decision
function stop()
    return {action = "stop"}
end

-- Factory: create "run to random position" decision
function run_to_random_position()
    local target_x = math.random() * FIELD_WIDTH   -- X axis: 0-68m
    local target_z = math.random() * FIELD_LENGTH  -- Z axis: 0-105m
    return {
        action = "run",
        target_type = "point",
        target = {x = target_x, z = target_z, y = 0}
    }
end

-- Geometry functions

function is_point_in_rectangle(x, z, rect)
    return x >= rect.min_x and x <= rect.max_x and z >= rect.min_z and z <= rect.max_z
end

function is_point_in_circle(x, z, circle)
    local dx = x - circle.center_x
    local dz = z - circle.center_z
    return dx * dx + dz * dz <= circle.radius * circle.radius
end

function is_point_in_arc(x, z, arc)
    local dx = x - arc.center_x
    local dz = z - arc.center_z
    local dist_sq = dx * dx + dz * dz
    if dist_sq > arc.radius * arc.radius then
        return false
    end

    local angle = math.atan(dz, dx)
    if angle < 0 then
        angle = angle + 2 * math.pi
    end

    local start_rad = math.rad(arc.start_angle)
    local end_rad = math.rad(arc.end_angle)

    if end_rad < start_rad then
        return angle >= start_rad or angle <= end_rad
    else
        return angle >= start_rad and angle <= end_rad
    end
end

-- Football-specific zone checks

function is_point_in_penalty_area(x, z, team)
    local suffix = team == "a" and "_a" or "_b"
    local zone = GAME_DATA.zones["penalty_area" .. suffix]
    if not zone then return false end
    return is_point_in_rectangle(x, z, zone)
end

function is_point_in_goal_area(x, z, team)
    local suffix = team == "a" and "_a" or "_b"
    local zone = GAME_DATA.zones["goal_area" .. suffix]
    if not zone then return false end
    return is_point_in_rectangle(x, z, zone)
end

function is_point_in_half(x, z, team)
    local suffix = team == "a" and "_a" or "_b"
    local zone = GAME_DATA.zones["half" .. suffix]
    if not zone then return false end
    return is_point_in_rectangle(x, z, zone)
end

function is_point_in_center_circle(x, z)
    local zone = GAME_DATA.zones["center_circle"]
    if not zone then return false end
    return is_point_in_circle(x, z, zone)
end
