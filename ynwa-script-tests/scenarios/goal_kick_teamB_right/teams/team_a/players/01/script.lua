player_setup = {
    goal_kick = function()
        if is_my_team_restarting() then
            return run_to_restart_position()
        end
        return run_to_start_position()
    end,
}
