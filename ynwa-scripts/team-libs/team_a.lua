-- Team A specific functions

-- Find a teammate ahead (closer to opponent goal) with free space around (5m radius)
-- Returns teammate table or nil if none found
function find_teammate_ahead_in_free_space()
    local my_pos = my_position()
    local my_team = my_team_name()
    local teammates = get_teammates()
    local opponents = get_opponents()
    
    -- Team A attacks toward increasing Z (toward 105m)
    local forward_direction = 1
    
    for _, tm in ipairs(teammates) do
        -- Check if teammate is ahead (higher Z coordinate)
        local is_ahead = (tm.position.z - my_pos.z) * forward_direction > 0
        
        if is_ahead then
            -- Check if teammate has free space (5m radius)
            local has_free_space = true
            for _, opp in ipairs(opponents) do
                if distance(tm.position, opp.position) < 5.0 then
                    has_free_space = false
                    break
                end
            end
            
            if has_free_space then
                return tm
            end
        end
    end
    
    return nil
end

-- Find teammate ahead (closer to opponent goal) who has free space around (5m radius)
-- Returns teammate object or nil if none found
function find_teammate_ahead_in_free_space()
    local my_pos = my_position()
    local my_team = my_team_name()
    local teammates = get_teammates()
    local opponents = get_opponents()
    
    -- For Team A, "ahead" means larger Z coordinate (attacking toward Z=105)
    local forward_direction = (my_team == "A") and 1 or -1
    
    for _, tm in ipairs(teammates) do
        -- Check if teammate is ahead
        local is_ahead = (tm.position.z - my_pos.z) * forward_direction > 0
        
        if is_ahead then
            -- Check if teammate has free space (no opponent within 5m)
            local has_free_space = true
            for _, opp in ipairs(opponents) do
                if distance(tm.position, opp.position) < 5.0 then
                    has_free_space = false
                    break
                end
            end
            
            if has_free_space then
                return tm
            end
        end
    end
    
    return nil
end
