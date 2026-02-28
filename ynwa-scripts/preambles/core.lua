-- Core preamble: elementary functions for reading game state and creating decisions
-- These functions provide access to context data without any game logic

-- Field dimensions are provided by the engine via GAME_DATA.field (not hardcoded here)
-- GAME_DATA.field.width  -- X axis (meters)
-- GAME_DATA.field.length -- Z axis (meters)

-- Returns the global index of the ball owner, or nil if ball is free
function ball_owner()
    return context.ball.owner_index
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

-- Returns my global player index
function my_index()
    return context.me.index
end

-- Returns my team name ("A" or "B")
function my_team_name()
    return context.me.team
end

-- Returns the team that owns the ball ("A", "B", or "None")
function get_ball_owner_team()
    return context.ball.owner_team
end

-- Opponent goal always has larger Z in the current player's coordinate system.
function get_opponent_goal()
    if my_team_name() == "A" then
        return GAME_DATA.zones.goal_b
    else
        return GAME_DATA.zones.goal_a
    end
end

function get_own_goal()
    if my_team_name() == "A" then
        return GAME_DATA.zones.goal_a
    else
        return GAME_DATA.zones.goal_b
    end
end
