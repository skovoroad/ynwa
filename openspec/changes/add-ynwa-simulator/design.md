# Design

## Context

See `proposal.md` for motivation and dependencies.
- **Prerequisite 1 (`configurable-match-teams`)**: Provides `FootballWorldBuilder::with_teams(team_a_id, team_b_id)` and `meta.toml` `display_name` loading.
- **Prerequisite 2 (`record-footer-stats`)**: Ensures `Game::finish_journal()` writes `elapsed_time`, `team_stats`, and `player_stats` into the trailing `"type": "footer"` line of the `.jsonl` recording.
- **Prerequisite 3 (`client-owned-match-duration`)**: Removes `GAME_DURATION` / `FootballEvent::GameEnd` from `ynwa-football` so the client loop controls match duration directly.

## Goals / Non-Goals

**Goals:**
- Provide a lightweight headless binary crate `ynwa-simulator` with zero windowing/graphics dependencies (`macroquad` is not included).
- Allow full control from shell scripts over teams, directories, RNG (`--seed`, `--temperature`), stepping (`--timestep`), match length (`--duration`), and output recording path (`--out`).
- Guarantee bounded execution by stepping in a simple loop (`while world.game().state().elapsed_time < cli.duration { world.step(cli.timestep); }`).

**Non-Goals:**
- Multi-match tournament scheduling or bracket management inside `ynwa-simulator` (orchestration of pairs and tournaments belongs in higher-level shell scripts).
- Visual playback or interactive controls (handled by `ynwa-player --replay`).

## Decisions

1. **Separate binary crate `ynwa-simulator`**:
   - Keeping `ynwa-simulator` separate from `ynwa-player` avoids linking `macroquad`/OpenGL/audio libraries or opening an X11/Wayland display window during headless batch runs, enabling fast startup and parallel execution across CPU cores from bash scripts.

2. **Client-driven model-time loop (`elapsed_time < duration`)**:
   - Because `World::step(timestep)` increments `game.state.elapsed_time` by `timestep` on every step regardless of `GameStage` (`Setup` or `Play`), stepping while `world.game().state().elapsed_time < cli.duration` (with finite `timestep > 0.0` and `duration > 0.0`) is guaranteed to terminate in `ceil(duration / timestep)` steps even if a player stalls during `Setup`.

3. **Strict script initialization (no placeholder fallback)**:
   - Unlike `ynwa-player` (which enables `.with_placeholder_fallback()` so the UI can still open if Lua initialization fails), `ynwa-simulator` uses `FootballWorldBuilder::new(&repo, &preambles).with_teams(&team_a, &team_b).with_rng(rng).build()` without `.with_placeholder_fallback()`, so broken team scripts fail loudly with exit code `1` instead of silently skewing tactical comparisons.

4. **Output path resolution (`--out`)**:
   - If `--out` is omitted, default to `<home>/.ynwa/games/<team_a>_vs_<team_b>_YYYYMMDD-HHMMSS[_2..].jsonl`.
   - If `--out <path>` has extension `.jsonl`, treat `<path>` as the exact target file (creating parent directories if needed).
   - Otherwise, treat `--out <path>` as a directory (creating it if needed) and generate `<path>/<team_a>_vs_<team_b>_YYYYMMDD-HHMMSS[_2..].jsonl`.

## Risks / Trade-offs

- **[Risk] Runtime script errors during `Play` ticks** → Mitigation: In `DecisionSystem`, runtime Lua script errors are recorded on `PlayerState::last_error`. `ynwa-simulator` should check whether any player has `last_error.is_some()` (or log/fail as appropriate) so broken scripts do not go unnoticed during batch runs.
