-- Test script: kick ball in random direction if I own it
-- Uses preamble functions: am_i_ball_owner(), GAME_DATA.field
function make_decision()
    -- Team library function: kick ball in random direction if I own it
    if am_i_ball_owner() then
        local target_x = math.random() * GAME_DATA.field.width
        local target_z = math.random() * GAME_DATA.field.length
        return {
            action = "kick",
            target = {x = target_x, z = target_z}
        }
    else
        -- Ball is not mine, just stop
        return {action = "stop"}
    end
end
