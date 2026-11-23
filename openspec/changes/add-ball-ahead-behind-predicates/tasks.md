# Tasks

## 1. Add ball directional predicates

- [x] 1.1 Add `is_ball_ahead()` (`ball_position().z > my_position().z`) and `is_ball_behind()` (`ball_position().z < my_position().z`) to the predicates section of `ynwa-scripts/preambles/stdlib.lua`; verify `cargo test -p ynwa-script-tests` compiles
- [x] 1.2 Add a test in `ynwa-script-tests/tests/stdlib_functions.rs` (extend `test_directional_predicates`) asserting ball-ahead, ball-behind, and exact-equality both-false; verify `cargo test -p ynwa-script-tests` passes

## 2. Documentation

- [x] 2.1 Document `is_ball_ahead()` / `is_ball_behind()` in the condition-checks table of `context/user-docs/scripting_player.md`; verify the table lists both signatures
