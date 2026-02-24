-- Standard library: common utilities for all teams
-- Uses functions from core preamble

-- Returns true if I am the ball owner, false otherwise
function am_i_ball_owner()
    local owner = ball_owner()
    if owner == nil then
        return false
    end
    return my_index() == owner
end

-- Calculate distance between two positions (2D, ignoring Y)
function distance(pos1, pos2)
    local dx = pos1.x - pos2.x
    local dz = pos1.z - pos2.z
    return math.sqrt(dx * dx + dz * dz)
end

-- Returns a Kick decision aimed at the center of the opponent's goal.
-- Works transparently for both teams: get_opponent_goal() handles coordinate perspective.
function kick_to_opponent_goal()
    local goal = get_opponent_goal()
    local target_x = (goal.min_x + goal.max_x) / 2
    local target_z = (goal.min_z + goal.max_z) / 2
    return {
        action = "kick",
        target = {x = target_x, z = target_z}
    }
end

-- Default get_setup_position for setup stage
-- Runs to center of "start position" region.
-- Stopping is handled by the engine automatically once the player is within 0.5m of the target.
function default_get_setup_position(reason)
    local start_pos = my_regions()["start position"]

    if start_pos == nil then
        return {action = "stop"}
    end

    local center_x = (start_pos.min_x + start_pos.max_x) / 2
    local center_z = (start_pos.min_z + start_pos.max_z) / 2

    return {
        action = "run",
        target_type = "point",
        target = {x = center_x, z = center_z, y = 0}
    }
end

-- Action: run to the ball
function chase_ball()
    return {action = "run", target_type = "ball"}
end

-- Action: chase ball if in top-3 closest, otherwise run to attack position
function press_or_attack()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_attack_position()
end

-- Action: chase ball if in top-3 closest, otherwise run to defence position
function press_or_defend()
    return am_i_top3_closest_to_ball() and chase_ball() or run_to_defence_position()
end

-- Action: run to center of "attack position" region
function run_to_attack_position()
    local pos = my_regions()["attack position"]
    if pos == nil then return {action = "stop"} end
    return {
        action = "run",
        target_type = "point",
        target = {x = (pos.min_x + pos.max_x) / 2, z = (pos.min_z + pos.max_z) / 2, y = 0}
    }
end

-- Action: run to center of "defence position" region
function run_to_defence_position()
    local pos = my_regions()["defence position"]
    if pos == nil then return {action = "stop"} end
    return {
        action = "run",
        target_type = "point",
        target = {x = (pos.min_x + pos.max_x) / 2, z = (pos.min_z + pos.max_z) / 2, y = 0}
    }
end

-- Action: run to center of "start position" region
function run_to_start_position()
    return default_get_setup_position(nil)
end

-- Returns true if this player is among the 3 closest teammates to the ball
function am_i_top3_closest_to_ball()
    local my_dist = distance(my_position(), ball_position())
    local closer = 0
    for _, tm in ipairs(get_teammates()) do
        if distance(tm.position, ball_position()) < my_dist then
            closer = closer + 1
        end
    end
    return closer < 3
end

-- Action: pass to nearest teammate at least 15m away; kick toward opponent goal if none found
function pass_to_nearest_teammate()
    local my_pos = my_position()
    local best = nil
    local best_dist = math.huge
    for _, tm in ipairs(get_teammates()) do
        local d = distance(my_pos, tm.position)
        if d >= 15.0 and d < best_dist then
            best_dist = d
            best = tm
        end
    end
    if best then
        return {action = "kick", target = {x = best.position.x, z = best.position.z}}
    end
    return kick_to_opponent_goal()
end

-- Dispatcher for Play stage.
-- Determines possession state and calls the appropriate handler from player_play or team_play.
-- Priority: player_play[state] -> team_play[state] -> error()
-- Valid states: "i_have_ball", "ball_is_free", "team_has_ball", "opponent_has_ball"
function make_decision()
    local state
    if am_i_ball_owner() then
        state = "i_have_ball"
    elseif get_ball_owner_team() == "None" then
        state = "ball_is_free"
    elseif get_ball_owner_team() == my_team_name() then
        state = "team_has_ball"
    else
        state = "opponent_has_ball"
    end

    -- player_play takes priority over team_play
    if player_play and player_play[state] then
        return player_play[state]()
    end
    if team_play and team_play[state] then
        return team_play[state]()
    end
    error("make_decision: no handler for state '" .. state .. "'")
end

-- Parse column label (e.g. "A"→1, "Z"→26, "AA"→27) — case-insensitive.
function parse_col(s)
    local col = 0
    s = s:upper()
    for i = 1, #s do
        col = col * 26 + (string.byte(s, i) - string.byte('A') + 1)
    end
    return col
end

-- Parse grid notation (e.g. "M22") → col (number), row (number).
function parse_notation(n)
    local col_str, row_str = n:match("^([A-Za-z]+)(%d+)$")
    if not col_str then error("invalid region notation: " .. n) end
    return parse_col(col_str), tonumber(row_str)
end

-- Returns true if my position is inside the region defined by grid notation (e.g. "A1", "Z44").
function is_in_region(from_notation, to_notation)
    local cell_w = GAME_DATA.field.width / GAME_DATA.field.columns
    local cell_h = GAME_DATA.field.length / GAME_DATA.field.rows

    local from_col, from_row = parse_notation(from_notation)
    local to_col,   to_row   = parse_notation(to_notation)

    local min_x = (from_col - 1) * cell_w
    local max_x = to_col * cell_w
    local min_z = (from_row - 1) * cell_h
    local max_z = to_row * cell_h

    local pos = my_position()
    return pos.x >= min_x and pos.x < max_x and pos.z >= min_z and pos.z < max_z
end

-- Returns a Run decision targeting a grid region.
function run_to_region(from_notation, to_notation)
    return {
        action = "run",
        target_type = "region",
        target = {from = from_notation, to = to_notation}
    }
end

-- Dispatcher for Setup stage.
-- Priority: player_setup[reason] -> team_setup[reason] -> default_get_setup_position(reason)
function get_setup_position(reason)
    if player_setup and player_setup[reason] then
        return player_setup[reason]()
    end
    if team_setup and team_setup[reason] then
        return team_setup[reason]()
    end
    return default_get_setup_position(reason)
end
