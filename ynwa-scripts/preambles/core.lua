-- Core preamble: elementary functions for reading game state and creating decisions
-- These functions provide access to context data without any game logic
--
-- RULE: team preambles and player scripts must NOT access `context` or `GAME_DATA` directly.
-- Use the functions below instead. Direct access by raw key (e.g. `GAME_DATA.zones.goal_a`)
-- is team-specific and breaks portability between teams.
--
-- User-facing API (the only Layer 1 functions team preambles and player scripts may call):
--   my_position, ball_position, my_team_name, get_ball_owner_team
-- Functions marked INTERNAL are used by stdlib.lua and are not part of that whitelist.

-- Field dimensions are provided by the engine via GAME_DATA.field (not hardcoded here)
-- GAME_DATA.field.width  -- X axis (meters)
-- GAME_DATA.field.length -- Z axis (meters)

-- INTERNAL: global index of the ball owner, or nil if the ball is free
function ball_owner()
    return context.ball.owner_index
end

-- Returns my position as a table {x, y, z}
function my_position()
    return context.me.position
end

-- INTERNAL: my velocity as a table {x, z} in team orientation
function my_velocity()
    return context.me.velocity
end

-- INTERNAL: my regions as a table {region_name = {min_x, max_x, min_z, max_z}}
function my_regions()
    return context.me.regions
end

-- Returns ball position as a table {x, y, z}
function ball_position()
    return context.ball.position
end

-- INTERNAL: ball velocity as a table {x, z} in team orientation
function ball_velocity()
    return context.ball.velocity
end

-- INTERNAL: array of teammates including me (each has: index, number, position)
function get_teammates()
    return context.teammates
end

-- INTERNAL: array of opponents (each has: index, number, position)
function get_opponents()
    return context.opponents
end

-- INTERNAL: my global player index
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

-- INTERNAL: opponent goal always has larger Z in the current player's coordinate system.
function get_opponent_goal()
    if my_team_name() == "A" then
        return GAME_DATA.zones.goal_b
    else
        return GAME_DATA.zones.goal_a
    end
end

-- INTERNAL: own goal always has smaller Z in the current player's coordinate system.
function get_own_goal()
    if my_team_name() == "A" then
        return GAME_DATA.zones.goal_a
    else
        return GAME_DATA.zones.goal_b
    end
end

-- INTERNAL: opponent penalty area, in the current player's coordinate system.
function get_opponent_penalty_area()
    if my_team_name() == "A" then
        return GAME_DATA.zones.penalty_area_b
    else
        return GAME_DATA.zones.penalty_area_a
    end
end
