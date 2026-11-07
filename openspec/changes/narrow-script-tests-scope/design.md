# Design

## Context

`ynwa-script-tests` is a test-only crate (`src/lib.rs` exists solely so integration tests can import its helpers). Its tests live in `tests/basic_scripts.rs`, `tests/dispatch_tests.rs`, `tests/stdlib_functions.rs`, and `tests/team_orientation.rs`. It loads the real preambles from `<workspace>/ynwa-scripts/preambles/{core,stdlib}.lua` via relative `CARGO_MANIFEST_DIR` paths, plus team fixtures from `fixtures/`. A `scenarios/` directory holds four `goal_kick_team*` team sets that no test references.

The same concerns are also covered elsewhere:

- `ynwa-core/src/systems/decision/decision_parser.rs` unit tests cover JSON -> `Decision` parsing (stop/cell/point/kick/region/reason).
- `ynwa-core/src/systems/decision/scripted_decision_maker.rs` unit tests cover context building, region/zone `GAME_DATA`, and team orientation.
- `ynwa-integration-testing` scenarios cover parsing end-to-end (`decision_targets_*`), team symmetry (`decision_symmetry_*`), and full physical runs.

## Goals / Non-Goals

**Goals:**

- Reduce `ynwa-script-tests` to tests that only the real-preamble layer can provide.
- Remove the `scenarios/` directory and the duplicated parsing/orientation/context tests.
- Keep (and where needed, backfill) coverage so no behavior regresses.

**Non-Goals:**

- Do not rename `ynwa-script-tests` or change the crate/folder composition.
- Do not move preamble loading to a data crate or change how `core.lua`/`stdlib.lua` are read.
- Do not modify `ynwa-core`, `ynwa-football`, `ynwa-decisions`, or `ynwa-player` behavior (only add unit tests in `ynwa-core` where removal would otherwise leave a gap).

## Decisions

### 1. Keep boundary: "Lua preamble contract tests"

Keep a test only if it executes the real `core.lua` or `stdlib.lua` and asserts an exact `Decision`/`reason`, a pure-function boundary, or `make_decision()` dispatch routing.

- **Keep all of** `tests/dispatch_tests.rs` (5 tests) — `make_decision()` state routing and `player_play` over `team_play`.
- **Keep all of** `tests/stdlib_functions.rs` (27 tests) — `core.lua`/`stdlib.lua` function semantics and boundaries.
- **Keep from** `tests/basic_scripts.rs`: `test_kick_if_ball_owner` (uses `am_i_ball_owner`/`ball_owner`) and `test_get_opponent_goal_returns_valid_zone` (uses `get_opponent_goal`).

### 2. Remove list

- `tests/team_orientation.rs` (all 6 tests) — team coordinate orientation is a Rust-side concern already covered by `scripted_decision_maker.rs` orientation tests and the `decision_symmetry_*`/`decision_targets_*_team_b` scenarios.
- From `tests/basic_scripts.rs`: the JSON -> `Decision` parsing tests (`test_stop_decision`, `test_run_to_cell_decision`, `test_run_to_region_decision`, `test_run_to_point_decision`, `test_kick_decision`, `test_run_to_ball_decision`, `test_run_to_ball_no_target_field`, `test_run_to_ball_with_reason`) and the context-shape tests (`test_context_structure`, `test_ball_owner_team_context`), which directly access `context` and duplicate `scripted_decision_maker.rs`.
- `scenarios/` (4 directories) — delete; unreferenced by any test.

### 3. Consolidate surviving `basic_scripts.rs` tests

After removal, `basic_scripts.rs` holds only two preamble-function tests. Move them into `tests/stdlib_functions.rs` (which already mixes `core.lua` and `stdlib.lua` function tests) and delete `tests/basic_scripts.rs`. Result: `dispatch_tests.rs` + `stdlib_functions.rs`, matching the narrowed role.

### 4. Close the coverage gaps removal would otherwise create

Removing the parsing/context tests would silently drop two boundary checks that currently have no equivalent unit test elsewhere. Backfill them in `ynwa-core` as part of this change:

- `target_type = "ball"` branch (including the missing-`target` edge) — add `decision_parser.rs` unit tests; the happy path is already covered end-to-end by `decision_targets_run_ball_*` scenarios, but the parser branch itself is untested.
- Complete context schema (`context.me.*`, `teammates`, `opponents`, `ball.*`, `game.elapsed_time`) — add a `scripted_decision_maker.rs` unit test asserting the full field set, replacing the direct-`context`-access Lua test.

### 5. Remove now-unused helpers

After the removals, prune helper functions from `src/lib.rs` that no test references (expected: `create_test_game_with_script` and `create_test_game_with_script_and_stage`). Keep `load_test_script`, `request_decisions_for_all`, `deterministic_rng`, and the preamble-loading helpers that the surviving tests use. Verify `cargo clippy` is clean (dead code would fail it).

## Risks / Trade-offs

- [Removing `run_to_ball`/context tests loses coverage] -> Backfill the equivalent `decision_parser.rs` and `scripted_decision_maker.rs` unit tests (Decision 4).
- [Pruning helpers may remove something still referenced] -> Run `cargo test -p ynwa-script-tests` and `cargo clippy --workspace` after the edit; any unused helper is caught as a dead-code warning.
- [Spec drift while narrowing] -> This change updates `scripting-and-protocols` and `integration-testing` in lockstep with the code, so the spec and the crate agree on the narrowed role.
