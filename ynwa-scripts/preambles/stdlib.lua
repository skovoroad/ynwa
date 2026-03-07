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
        target = {x = target_x, z = target_z},
        reason = string.format("kick_to_goal(%.1f,%.1f)", target_x, target_z)
    }
end

-- Default get_setup_position for setup stage
-- Runs to center of "start position" region.
-- Stopping is handled by the engine automatically once the player is within 0.5m of the target.
function default_get_setup_position(reason)
    local start_pos = my_regions()["start position"]

    if start_pos == nil then
        return {action = "stop", reason = "no_start_position"}
    end

    local center_x = (start_pos.min_x + start_pos.max_x) / 2
    local center_z = (start_pos.min_z + start_pos.max_z) / 2

    return {
        action = "run",
        target_type = "point",
        target = {x = center_x, z = center_z, y = 0},
        reason = "run_to_start_position:" .. start_pos.display_notation
    }
end

-- Action: run to the ball
function chase_ball()
    return {action = "run", target_type = "ball", reason = "chase_ball"}
end

-- Action: run to the ball
function stop(reason)
    return {action = "stop", reason = reason}
end

-- Action: run to center of a region object {min_x, max_x, min_z, max_z}.
function run_to_region_obj(r, reason)
    return {action="run", target_type="point",
            target={x=(r.min_x+r.max_x)/2, z=(r.min_z+r.max_z)/2, y=0},
            reason=reason}
end

-- Action: run to center of "attack position" region
function run_to_attack_position()
    local pos = my_regions()["attack position"]
    if pos == nil then return {action = "stop", reason = "no_attack_position"} end
    return run_to_region_obj(pos, "run_to_attack_position")
end

-- Action: run to center of "defence position" region
function run_to_defence_position()
    local pos = my_regions()["defence position"]
    if pos == nil then return {action = "stop", reason = "no_defence_position"} end
    return run_to_region_obj(pos, "run_to_defence_position")
end

-- Action: run to center of "start position" region
function run_to_start_position()
    return default_get_setup_position(nil)
end

-- Returns the teammate object with the given number, or nil if not found.
function get_teammate_by_number(n)
    for _, tm in ipairs(get_teammates()) do
        if tm.number == n then return tm end
    end
    return nil
end

-- Action: pass to a specific teammate object.
function pass_to_teammate(tm)
    return {action = "kick", target = {x = tm.position.x, z = tm.position.z}, reason = "pass_to_#" .. tm.number}
end

-- Action: pass to nearest teammate among given numbers; kick to goal if none found.
function pass_to_players_by_numbers(numbers)
    local my_pos = my_position()
    local best, best_dist = nil, math.huge
    for _, tm in ipairs(get_teammates()) do
        for _, n in ipairs(numbers) do
            if tm.number == n then
                local d = distance(my_pos, tm.position)
                if d < best_dist then best_dist = d; best = tm end
            end
        end
    end
    if best then
        return {action = "kick", target = {x = best.position.x, z = best.position.z}, reason = "pass_to_#" .. best.number}
    end
    return kick_to_opponent_goal()
end

-- Action: stay on the goal line at defence_position Z, tracking ball X clamped to goal width.
function default_goalkeeper_cover_position()
    local defence = my_regions()["defence position"]
    if defence == nil then return run_to_defence_position() end
    local goal = get_own_goal()
    local target_z = (defence.min_z + defence.max_z) / 2
    -- clamp ball X to goal post positions so keeper never steps outside
    local ball_x = ball_position().x
    local clamped_x = math.max(goal.min_x, math.min(goal.max_x, ball_x))
    return {
        action = "run",
        target_type = "point",
        target = {x = clamped_x, z = target_z, y = 0},
        reason = "goalkeeper_cover"
    }
end

function is_in_opponent_penalty_area()
    return is_in_region_obj(get_opponent_penalty_area())
end

-- Run to center of opponent penalty area.
function run_to_opponent_penalty_area()
    local pa = get_opponent_penalty_area()
    return {
        action = "run",
        target_type = "point",
        target = {x = (pa.min_x + pa.max_x) / 2, z = (pa.min_z + pa.max_z) / 2, y = 0},
        reason = "run_to_opponent_penalty_area"
    }
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

-- Returns true if my position is inside the region object {min_x, max_x, min_z, max_z}.
function is_in_region_obj(region)
    local pos = my_position()
    return pos.x >= region.min_x and pos.x < region.max_x
       and pos.z >= region.min_z and pos.z < region.max_z
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
        target = {from = from_notation, to = to_notation},
        reason = "run_to_region:" .. from_notation .. "-" .. to_notation
    }
end

-- Returns a Kick decision aimed at the center of a grid cell.
function kick_to_cell(notation)
    local cell_size = GAME_DATA.field.width / GAME_DATA.field.columns
    local col, row = parse_notation(notation)
    return {
        action = "kick",
        target = {x = (col - 0.5) * cell_size, z = (row - 0.5) * cell_size, y = 0},
        reason = "kick_to_cell:" .. notation
    }
end

-- Returns a Kick decision aimed at the center of a grid region.
function kick_to_region(from_notation, to_notation)
    local cell_size = GAME_DATA.field.width / GAME_DATA.field.columns
    local from_col, from_row = parse_notation(from_notation)
    local to_col,   to_row   = parse_notation(to_notation)
    local cx = ((from_col - 1) + to_col) / 2 * cell_size
    local cz = ((from_row - 1) + to_row) / 2 * cell_size
    return {
        action = "kick",
        target = {x = cx, z = cz, y = 0},
        reason = "kick_to_region:" .. from_notation .. "-" .. to_notation
    }
end

-- Returns {x, z} of the restart point, or nil if setup_info is absent (start/after_goal).
function get_restart_position()
    if not context.game.setup_info then return nil end
    return {x = context.game.setup_info.restart_x, z = context.game.setup_info.restart_z}
end

-- Returns true if my team initiates the restart, nil if setup_info is absent.
function is_my_team_restarting()
    if not context.game.setup_info then return nil end
    return context.game.setup_info.restarting_team == my_team_name()
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

-- Default goal kick setup: uses tactical profile regions when available.
-- Restarting team: go to "goal kick own position" or start position.
-- Defending team:  go to "goal kick opp position" or defence position.
function default_goal_kick_setup()
    if is_my_team_restarting() then
        local r = my_regions()["goal kick own position"]
        if r then return run_to_region_obj(r, "goal_kick_own_position") end
        return run_to_start_position()
    else
        local r = my_regions()["goal kick opp position"]
        if r then return run_to_region_obj(r, "goal_kick_opp_position") end
        return run_to_defence_position()
    end
end
