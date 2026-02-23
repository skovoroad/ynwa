-- Team A specific functions

-- Find a teammate ahead (closer to opponent goal) with free space around (5m radius)
-- Returns teammate table or nil if none found
function find_teammate_ahead_in_free_space()
    local my_pos = my_position()
    local teammates = get_teammates()
    local opponents = get_opponents()
    
    for _, tm in ipairs(teammates) do
        local is_ahead = tm.position.z > my_pos.z
        
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