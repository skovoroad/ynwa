# Proposal

## Why

Player scripts can express directional relations between two players (`is_teammate_ahead/behind`, `is_opponent_ahead/behind`) but cannot express the player's position relative to the ball line — e.g. "if I am ahead of the ball line, then … else …". The only workaround (`ball_position().z > my_position().z`) violates the authoring rule forbidding raw coordinate reads, and no existing predicate has the ball as its subject.

## What Changes

- Add two stdlib predicates that take no argument and use **no tolerance**:
  - `is_ball_ahead()` — `ball_position().z > my_position().z` (ball strictly closer to the opponent goal than me)
  - `is_ball_behind()` — `ball_position().z < my_position().z` (ball strictly closer to the own goal than me)
- At exact equality (`ball.z == my.z`) both return `false`, so "if I am ahead of the ball line, then … else …" is expressed unambiguously as `if is_ball_behind() then … else …`.
- Deliberately omit `is_ball_left/right`: the ball *line* has no lateral dimension; left/right of the ball (a point) is out of scope.
- Document the two predicates in the `scripting_player.md` condition-checks table.
- Add a script test in `ynwa-script-tests` covering ahead, behind, and exact equality.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `scripting-and-protocols`: add `is_ball_ahead`/`is_ball_behind` to the Layer 2 stdlib predicate catalog and define their exact no-tolerance semantics.

## Impact

- `ynwa-scripts/preambles/stdlib.lua`: two new public predicates.
- `ynwa-script-tests/tests/stdlib_functions.rs`: extend the directional-predicate test.
- `context/user-docs/scripting_player.md`: condition-checks table.
- No engine, wire-format, or team-preamble changes; the ball position is already delivered in team orientation, so Team A/B symmetry is automatic.
