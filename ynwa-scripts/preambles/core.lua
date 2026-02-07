-- Core preamble: elementary functions for reading game state and creating decisions
-- These functions provide access to context data without any game logic

-- Field dimensions (standard football)
-- TODO: eliminate code duplication (these values are also in ynwa-core)
FIELD_LENGTH = 105.0  -- meters (X axis)
FIELD_WIDTH = 68.0    -- meters (Z axis)

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
    local target_x = math.random() * FIELD_LENGTH
    local target_z = math.random() * FIELD_WIDTH
    return {
        action = "run",
        target_type = "point",
        target = {x = target_x, z = target_z, y = 0}
    }
end
