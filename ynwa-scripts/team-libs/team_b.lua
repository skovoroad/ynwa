-- Team B: role-based decision functions
-- Each function currently delegates to common_behavior_v2().
-- Customize individual functions here to develop team tactics.

-- Setup stage: delegate to stdlib default
function get_setup_position(reason)
    return default_get_setup_position(reason)
end

-- Role: Goalkeeper (number 1)
function make_goalkeeper_decision()
    return common_behavior_v2()
end

-- Role: Left back (number 2)
function make_left_back_decision()
    return common_behavior_v2()
end

-- Role: Center back left (number 3)
function make_center_back_left_decision()
    return common_behavior_v2()
end

-- Role: Center back right (number 4)
function make_center_back_right_decision()
    return common_behavior_v2()
end

-- Role: Right back (number 5)
function make_right_back_decision()
    return common_behavior_v2()
end

-- Role: Left midfielder (number 6)
function make_left_midfielder_decision()
    return common_behavior_v2()
end

-- Role: Center midfielder (number 7)
function make_center_midfielder_decision()
    return common_behavior_v2()
end

-- Role: Right midfielder (number 8)
function make_right_midfielder_decision()
    return common_behavior_v2()
end

-- Role: Left winger (number 9)
function make_left_winger_decision()
    return common_behavior_v2()
end

-- Role: Striker (number 10)
function make_striker_decision()
    return common_behavior_v2()
end

-- Role: Right winger (number 11)
function make_right_winger_decision()
    return common_behavior_v2()
end
