-- Team B: dispatch tables for play and setup stages

team_play = {
    i_have_ball       = pass_to_nearest_teammate,
    ball_is_free      = press_or_defend,
    team_has_ball     = press_or_attack,
    opponent_has_ball = press_or_defend,
}

team_setup = {
    start      = run_to_start_position,
    after_goal = run_to_start_position,
}
