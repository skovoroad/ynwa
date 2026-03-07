player_setup = {
    goal_kick = function()
        if is_my_team_restarting() then
            local rp = get_restart_position()
            if rp then
                return {action = "run", target_type = "point", target = {x = rp.x, z = rp.z, y = 0}}
            end
        end
        return run_to_region("N10", "N10")
    end,
}
