-- Test script: kick ball in random direction if I own it
-- Uses preamble functions: am_i_ball_owner(), FIELD_LENGTH, FIELD_WIDTH
function make_decision()
    -- Team library function: kick ball in random direction if I own it
    if am_i_ball_owner() then
        local target_x = math.random() * FIELD_LENGTH
        local target_z = math.random() * FIELD_WIDTH
        return {
            action = "kick",
            target = {x = target_x, z = target_z}
        }
    else
        -- Ball is not mine, just stop
        return {action = "stop"}
    end
end
