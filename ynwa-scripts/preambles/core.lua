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
