# Design

## Context

`stdlib.lua` already exposes directional predicates for teammates and opponents (`is_teammate_ahead(number)` etc.), each looking up a player by squad number and comparing against `my_position()` with a 0.5 m tolerance. The ball has no such predicate, and `ball_position()` / `my_position()` are already user-facing Layer 1 functions delivered in team orientation. See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**
- Add `is_ball_ahead()` / `is_ball_behind()` with no tolerance and no argument.
- Keep each a thin Layer 2 wrapper over the existing `ball_position()` and `my_position()`.

**Non-Goals:**
- No `is_ball_left`/`is_ball_right` (the ball line has no lateral dimension).
- No changes to `core.lua`, the wire format, or team preambles.
- No tolerance parameter — the strict comparison is the spec.

## Decisions

- **No tolerance**: `is_ball_ahead()` = `ball_position().z > my_position().z`; `is_ball_behind()` = `ball_position().z < my_position().z`. Rationale: the ball line is used for a binary "ahead vs else" branch, and exact equality is the unambiguous "on the line" case (both `false`). The 0.5 m tolerance used by player predicates is hysteresis against two players oscillating; it is not wanted here.
- **Naming**: `is_ball_ahead`/`is_ball_behind` mirror the `is_teammate_ahead`/`is_teammate_behind` shape (subject = ball, direction relative to me). The user-facing task "if I am ahead of the ball line" is written `if is_ball_behind() then ... else ...`.
- **No `core.lua` change**: both operands are already public, so the predicates are pure `stdlib.lua` additions.

## Risks / Trade-offs

- [A player exactly level with the ball gets `false` for both predicates] → Intended and documented as the "else" case.
- [No hysteresis can flip the branch when a player oscillates across the ball line] → Acceptable: the result is a branch, not a target, and the engine re-evaluates per tick.
