# Design

## Context

`ynwa-integration-testing` matches `[[expect.journal]]` entries against recorded `JournalEntry` items in `compare.rs::entry_matches`, and final-state `setup` in `compare.rs::compare_final_setup`. Input DTOs live in `dto.rs`. Decisions are recorded as `JournalEvent::DecisionAssigned { player_index, decision: Decision, reason: Option<String> }`, where `Decision` is `Run(DecisionTarget)`, `Stop`, or `Kick(Point3D)` and `DecisionTarget` is `Region`/`GridCell`/`Point`/`Ball`. Restart positions are `Option<Point3D>` (`None` means center). See `proposal.md` for motivation.

## Goals / Non-Goals

**Goals:**
- Assert decision targets (point / cell / region / ball) in `decision_assigned`.
- Assert "absent" (`"none"`) for `restart_set.position`, `final_state.setup.restart_position`, and `decision_assigned.reason`, distinct from "not checked".

**Non-Goals:**
- No changes to `ynwa-core` or `ynwa-football`.
- No new stop criteria (see `scenario-stop-criteria`).
- No region-based player position checking (see `scenario-region-check`).

## Decisions

1. **`"none"` markers reuse the existing `NoneToken` pattern.** `TeamOrNoneDto` and `BallOwnerDto` already deserialize `"none"` through `#[serde(untagged)]`. Add `ReasonOrNoneDto` (`String` vs `NoneToken`) and `PointOrNoneDto` (`Point3D` vs `NoneToken`) the same way, rather than a separate boolean flag. This keeps the schema uniform and `deny_unknown_fields`-safe.

2. **`decision_assigned.target` is a tagged enum of four variants.** `{ kind = "point", point = {x, y, z} }`, `{ kind = "cell", cell = "M22" }`, `{ kind = "region", region = "A1:B2" }`, `{ kind = "ball" }`. This mirrors the grid notation in `coordinate-system` and the Lua decision protocol in `scripting-and-protocols`, and honors the code-style rule to keep variant-dependent fields inside enum variants.

3. **Target notation resolves at compare time.** Cell and region notation is parsed with `GridCell::from_notation` / `Region::from_grid_notation` (region parsing needs `GameConfig` field `GridDimensions`, which `compare.rs` already receives). Invalid notation is a hard `ScenarioError`, matching how an unresolvable player reference behaves.

4. **Target semantics.** `target` applies only to `Run` and `Kick`: `Run(Point | GridCell | Region | Ball)` matches the corresponding variant; `Kick(Point3D)` matches `point`; `Stop` carries no target, so a specified target never matches. `target` remains optional (wildcard) for backward compatibility.

## Risks / Trade-offs

- [Region/cell notation parsing requires field grid dimensions] → parse inside `compare.rs` where `GameConfig` is available; validate the notation format eagerly where possible and defer grid-bounds checks to match time.
- [Widening `reason` and `position` types] → existing scenario files stay valid because omitted fields default to wildcard; `deny_unknown_fields` is preserved since the new types remain strict enums.
