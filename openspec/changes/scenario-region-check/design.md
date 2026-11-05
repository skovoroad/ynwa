# Design

## Context

`compare.rs::compare_final_state` checks `final_state.players` as exact `Point3D` positions via `point_close`. `ynwa_core::Region::contains_point(grid_dims, field_width_meters, point)` and `GameConfig::field.grid_dimensions()` already provide region containment. See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Assert a player lies inside a grid-notation region in `final_state.toml`.

**Non-Goals:**
- No region assertions in `initial_state.toml` (exact placement remains sufficient there).
- No changes to how regions are defined in `tactical.toml`.

## Decisions

1. **Two mutually exclusive placement forms.** A `final_state.players` entry carries either `position = {x, y, z}` (exact, existing) or `region = "A1:B2"` (containment). `deny_unknown_fields` plus a tagged enum keeps the two forms exclusive. `initial_state.players` keeps the existing exact-only `PlayerPlacementDto`; the region-capable variant is a separate final-state placement type, so region assertions do not leak into `initial_state.toml`.

2. **Resolve region notation at compare time.** Parse `region` with `Region::from_grid_notation(notation, grid_dims)`, where `grid_dims` comes from `GameConfig::field` already passed into `compare_final_state`. Invalid notation is a hard `ScenarioError`.

3. **Containment uses `Region::contains_point`.** Compare the player's final position against the parsed region using the field width and grid dimensions, ignoring height (Y).

## Risks / Trade-offs

- [Region containment is looser than an exact position] → scenarios that need exact placement keep using `position`; region assertions are opt-in.
- [Notation depends on the pitch grid] → grid bounds are validated by `Region::from_grid_notation` at compare time, surfacing errors as `ScenarioError`.
