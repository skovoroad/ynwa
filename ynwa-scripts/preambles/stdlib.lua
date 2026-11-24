-- Standard library: common utilities for all teams
-- Uses functions from core preamble
--
-- The whole file is wrapped in a `do ... end` block: the internal section holds `local`
-- helpers and constants, and the public API section defines the global functions team
-- preambles and player scripts may call. Locals do not leak into the concatenated chunk.

do
    -- =========================================================================
    -- Internal implementation (not part of the public API)
    -- =========================================================================

    local FREE_PLAYER_RADIUS = 5.0
    local DIRECTION_TOLERANCE = 0.5

    -- Parse column label (e.g. "A"→1, "Z"→26, "AA"→27) — case-insensitive.
    local function parse_col(s)
        local col = 0
        s = s:upper()
        for i = 1, #s do
            col = col * 26 + (string.byte(s, i) - string.byte('A') + 1)
        end
        return col
    end

    -- Parse grid notation (e.g. "M22") → col (number), row (number).
    local function parse_notation(n)
        local col_str, row_str = n:match("^([A-Za-z]+)(%d+)$")
        if not col_str then error("invalid region notation: " .. n) end
        return parse_col(col_str), tonumber(row_str)
    end

    -- Returns true if my position is inside the region object {min_x, max_x, min_z, max_z}.
    local function is_in_region_obj(region)
        local pos = my_position()
        return pos.x >= region.min_x and pos.x < region.max_x
           and pos.z >= region.min_z and pos.z < region.max_z
    end

    -- Action: run to center of a region object {min_x, max_x, min_z, max_z}.
    local function run_to_region_obj(r, reason)
        return {action="run", target_type="point",
                target={x=(r.min_x+r.max_x)/2, z=(r.min_z+r.max_z)/2, y=0},
                reason=reason}
    end

    -- Join a team-supplied reason with the action's built-in reason.
    local function compose_reason(custom, builtin)
        if custom == nil or custom == "" then return builtin end
        return custom .. " | " .. builtin
    end

    -- Returns the teammate object with the given number, or nil if not found.
    local function get_teammate_by_number(n)
        for _, tm in ipairs(get_teammates()) do
            if tm.number == n then return tm end
        end
        return nil
    end

    -- Returns the opponent object with the given number, or nil if not found.
    local function get_opponent_by_number(n)
        for _, opp in ipairs(get_opponents()) do
            if opp.number == n then return opp end
        end
        return nil
    end

    -- Returns true if I am the ball owner, false otherwise.
    local function am_i_ball_owner()
        local owner = ball_owner()
        if owner == nil then
            return false
        end
        return my_index() == owner
    end

    -- =========================================================================
    -- Public API
    -- =========================================================================

    -- -----------------------------------------------------------------------------
    -- Queries
    -- -----------------------------------------------------------------------------

    -- Calculate distance between two positions (2D, ignoring Y)
    function distance(pos1, pos2)
        local dx = pos1.x - pos2.x
        local dz = pos1.z - pos2.z
        return math.sqrt(dx * dx + dz * dz)
    end

    -- Squad numbers are a provisional player identifier pending the DSL "player" type.
    function teammate_position(number)
        local tm = get_teammate_by_number(number)
        if tm == nil then error("unknown teammate number: " .. tostring(number)) end
        return tm.position
    end

    function opponent_position(number)
        local opp = get_opponent_by_number(number)
        if opp == nil then error("unknown opponent number: " .. tostring(number)) end
        return opp.position
    end

    -- Squad number of the player who possesses the ball, or nil when nobody possesses it.
    -- An in-flight pass reports no owner even though get_ball_owner_team() still names the
    -- passing team. The DSL type list has no "nothing" value; revisit with the DSL "player" type.
    function ball_owner_number()
        local owner_index = ball_owner()
        if owner_index == nil then
            return nil
        end
        for _, tm in ipairs(get_teammates()) do
            if tm.index == owner_index then return tm.number end
        end
        for _, opp in ipairs(get_opponents()) do
            if opp.index == owner_index then return opp.number end
        end
        return nil
    end

    -- Number of teammates (including me) strictly closer than `radius` to `pos`.
    function count_teammates_around(pos, radius)
        local count = 0
        for _, tm in ipairs(get_teammates()) do
            if distance(tm.position, pos) < radius then
                count = count + 1
            end
        end
        return count
    end

    -- Number of opponents strictly closer than `radius` to `pos`.
    function count_opponents_around(pos, radius)
        local count = 0
        for _, opp in ipairs(get_opponents()) do
            if distance(opp.position, pos) < radius then
                count = count + 1
            end
        end
        return count
    end

    -- -----------------------------------------------------------------------------
    -- Predicates
    -- -----------------------------------------------------------------------------

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

    function is_in_opponent_penalty_area()
        return is_in_region_obj(get_opponent_penalty_area())
    end

    -- Returns true if no opponent is strictly within FREE_PLAYER_RADIUS of the teammate.
    function is_teammate_free(number)
        return count_opponents_around(teammate_position(number), FREE_PLAYER_RADIUS) == 0
    end

    -- Returns true if no teammate (including me) is strictly within FREE_PLAYER_RADIUS of the opponent.
    function is_opponent_free(number)
        return count_teammates_around(opponent_position(number), FREE_PLAYER_RADIUS) == 0
    end

    -- "Ahead" means closer to the opponent goal line (larger Z).
    function is_teammate_ahead(number)
        return teammate_position(number).z > my_position().z + DIRECTION_TOLERANCE
    end

    function is_teammate_behind(number)
        return teammate_position(number).z < my_position().z - DIRECTION_TOLERANCE
    end

    -- "Left" means smaller X (X = 0 is the left wing).
    function is_teammate_left(number)
        return teammate_position(number).x < my_position().x - DIRECTION_TOLERANCE
    end

    function is_teammate_right(number)
        return teammate_position(number).x > my_position().x + DIRECTION_TOLERANCE
    end

    function is_opponent_ahead(number)
        return opponent_position(number).z > my_position().z + DIRECTION_TOLERANCE
    end

    function is_opponent_behind(number)
        return opponent_position(number).z < my_position().z - DIRECTION_TOLERANCE
    end

    function is_opponent_left(number)
        return opponent_position(number).x < my_position().x - DIRECTION_TOLERANCE
    end

    function is_opponent_right(number)
        return opponent_position(number).x > my_position().x + DIRECTION_TOLERANCE
    end

    -- Ball on the opponent-goal side of my Z line. No tolerance: level with the ball is
    -- neither ahead nor behind, so `if is_ball_behind() then ... else ...` has no gap.
    function is_ball_ahead()
        return ball_position().z > my_position().z
    end

    function is_ball_behind()
        return ball_position().z < my_position().z
    end

    -- -----------------------------------------------------------------------------
    -- Primitive actions  (no tactical regions)
    -- -----------------------------------------------------------------------------

    function stop(reason)
        return {action = "stop", reason = reason}
    end

    function chase_ball(reason)
        return {action = "run", target_type = "ball", reason = compose_reason(reason, "chase_ball")}
    end

    -- Returns a Run decision targeting a grid region.
    function run_to_region(from_notation, to_notation, reason)
        return {
            action = "run",
            target_type = "region",
            target = {from = from_notation, to = to_notation},
            reason = compose_reason(reason, "run_to_region:" .. from_notation .. "-" .. to_notation)
        }
    end

    -- Returns a Kick decision aimed at an opaque position.
    function kick_to_position(pos, reason)
        local reason_text = reason or string.format("kick_to_position(%.1f,%.1f)", pos.x, pos.z)
        return {
            action = "kick",
            target = {x = pos.x, z = pos.z},
            reason = reason_text
        }
    end

    -- Returns a Kick decision aimed at the center of the opponent's goal.
    -- Works transparently for both teams: get_opponent_goal() handles coordinate perspective.
    function kick_to_opponent_goal(reason)
        local goal = get_opponent_goal()
        local target_x = (goal.min_x + goal.max_x) / 2
        local target_z = (goal.min_z + goal.max_z) / 2
        return {
            action = "kick",
            target = {x = target_x, z = target_z},
            reason = compose_reason(reason, string.format("kick_to_goal(%.1f,%.1f)", target_x, target_z))
        }
    end

    -- -----------------------------------------------------------------------------
    -- Tactical actions  (use my_regions() or tactical profile)
    -- -----------------------------------------------------------------------------

    -- Action: run to center of "start" region
    function run_to_start_position(reason)
        local pos = my_regions()["start"]
        if pos == nil then return stop("no_start_position") end
        return run_to_region_obj(pos, compose_reason(reason, "run_to_start_position"))
    end

    -- Action: run to center of "attack" region
    function run_to_attack_position(reason)
        local pos = my_regions()["attack"]
        if pos == nil then return stop("no_attack_position") end
        return run_to_region_obj(pos, compose_reason(reason, "run_to_attack_position"))
    end

    -- Action: run to center of "defence" region
    function run_to_defence_position(reason)
        local pos = my_regions()["defence"]
        if pos == nil then return stop("no_defence_position") end
        return run_to_region_obj(pos, compose_reason(reason, "run_to_defence_position"))
    end

    -- Run to center of opponent penalty area.
    function run_to_opponent_penalty_area(reason)
        local pa = get_opponent_penalty_area()
        return run_to_region_obj(pa, compose_reason(reason, "run_to_opponent_penalty_area"))
    end

    -- Action: stay on the goal line at defence Z, tracking ball X clamped to goal width.
    function default_goalkeeper_cover_position(reason)
        local defence = my_regions()["defence"]
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
            reason = compose_reason(reason, "goalkeeper_cover")
        }
    end

    -- Temporary helper: pass to the nearest teammate at least 15 m away, otherwise shoot.
    function default_goalkeeper_with_ball(reason)
        local my_pos = my_position()
        local best_number, best_position, best_distance = nil, nil, math.huge
        for _, tm in ipairs(get_teammates()) do
            local d = distance(my_pos, tm.position)
            if d >= 15.0 and d < best_distance then
                best_distance = d
                best_number = tm.number
                best_position = tm.position
            end
        end
        if best_number == nil then
            return kick_to_opponent_goal()
        end
        return kick_to_position(best_position, compose_reason(reason, "pass_to_#" .. best_number))
    end

    -- -----------------------------------------------------------------------------
    -- Play stage
    -- -----------------------------------------------------------------------------

    -- Dispatcher for Play stage.
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
end
