# Design

## Context

After `lead-targeting-stdlib` switches `chase_ball()` to emit `target_type = "point"`, `DecisionTarget::Ball` and the Lua `target_type = "ball"` branch have no remaining production callers. Removing them eliminates the only football-specific variant in `DecisionTarget`.

## Goals / Non-Goals

- **Goals**:
  - Remove `DecisionTarget::Ball` from `ynwa-core` and `ynwa-player`.
  - Remove `target_type = "ball"` from `decision_parser.rs`.
  - Simplify `resolve_target_point` by removing its `&BallState` parameter.
  - Update all affected test fixtures in `ynwa-core`, `ynwa-football`, and `ynwa-script-tests`.
- **Non-Goals**:
  - Changing `Decision::Run` or `Decision::Kick` behavior for `Point`, `GridCell`, or `Region`.

## Decisions

1. **Clean removal without compatibility shims**: Per project convention, `target_type = "ball"` is removed outright from `parse_decision` rather than mapped to a fallback; any script returning it produces an `Unknown target_type: ball` error.
2. **Test fixture replacement**: Existing serialization, replay, and lockstep tests that used `Decision::Run(DecisionTarget::Ball)` as an arbitrary `Run` fixture will use `Decision::Run(DecisionTarget::Point(...))` or `GridCell(...)`. Tests in `ynwa-script-tests/tests/basic_scripts.rs` that specifically asserted `target_type = "ball"` will be replaced with assertions that `target_type = "ball"` is rejected (or removed if redundant with `stdlib_functions.rs`).

