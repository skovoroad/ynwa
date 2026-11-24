# Design

## Context

`stdlib.lua` (Layer 2) is the team-facing API over `core.lua`. Its action functions build decision tables with a `reason` field that flows unchanged into `LuaDecision.reason` → `DecisionMaker::make_decision` → `PlayerState.decision_reason` → `JournalEvent::DecisionAssigned { reason }`. Only `stop(reason)` and `kick_to_position(pos, reason)` currently accept a reason; the rest hard-code it.

See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- Let teams tag any decision with a custom string that survives into the journal and is greppable.
- Preserve existing built-in reason strings (and existing scripts) when no custom reason is passed.

**Non-Goals:**

- Structured/typed reasons (plain string only).
- Changing the Rust-side reason flow (already complete).
- Adding reasons to query/predicate functions (they produce no decision).

## Decisions

1. **Optional trailing `reason` on every hard-coded action**: `chase_ball`, `run_to_region`, `kick_to_opponent_goal`, `run_to_start_position`, `run_to_attack_position`, `run_to_defence_position`, `run_to_opponent_penalty_area`, `default_goalkeeper_cover_position`, `default_goalkeeper_with_ball`.
2. **Concatenation helper** `compose_reason(custom, builtin)` returning `builtin` when `custom` is nil/empty, otherwise `custom .. " | " .. builtin`. Separator `" | "`.
3. **`stop` and `kick_to_position` stay verbatim**: they already accept `reason` and are documented/tested to use it directly, with no composed sub-reason.
4. **Dynamic built-in reason** for `default_goalkeeper_with_ball` is the composed `"pass_to_#" .. n` (not a fixed action name), so a custom reason yields `"custom | pass_to_#7"`.
5. **Fallback paths keep their own reason.** `run_to_*_position` returning `stop("no_*_position")` on a missing region keeps `"no_*_position"`; the custom reason applies to the success path only.

## Risks / Trade-offs

- [A custom reason containing `" | "` could be ambiguous under grep] → Documented separator; acceptable for a string tag.
- [Reason strings grow slightly] → Negligible.
- [Existing scripts/tests rely on exact built-in strings] → Default preserves the current strings, so no regression.
