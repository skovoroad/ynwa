# Proposal

## Why

Team B scripts get a `GAME_DATA.zones` view of the field turned 180° about its center, so the same Lua code works for both teams. Points, circles, rectangles and arc centers follow this rule (`x' = W − x`, `z' = L − z`). Arc angles do not: they are converted with `180° − a`, which mirrors X only. A 180° turn maps a direction angle `a` to `a + 180°`. As a result, every arc in a Team B script's view points the wrong way. For example, `penalty_arc_b` faces Team B's own goal line instead of the field center, and `corner_arc_bottom_a` faces out of the field.

The defect is latent today. Rendering uses the canonical field and is correct, and no Lua code reads arc angles yet. But the main specs currently state the wrong formula as normative, and it will break as soon as scripts (or the future user language) use arcs.

## What Changes

- Arc zones in Team B `GAME_DATA.zones` get angles rotated by 180° (`start + 180°`, `end + 180°`), normalized so that `start_angle` falls in `[-180°, 180°)` and the span (`end − start`) is preserved.
- Team A `GAME_DATA.zones`, the canonical `Field`, rendering, and all non-arc zone types are unchanged.
- The `coordinate-system` and `scripting-and-protocols` specs drop the `180 − end` / `180 − start` formula and state the corrected rule, with scenarios that check arc direction against the field center.
- Unit tests are added for arc serialization for both teams (no tests cover it today).
- **BREAKING** (Lua contract only): Team B scripts now receive different numeric `start_angle`/`end_angle` values. No current script or preamble reads these fields.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `coordinate-system`: the Team B reflection requirement changes its arc-angle rule from `180° − a` to `a + 180°` (normalized). Arcs are equal to the original modulo 360° after a double flip instead of strictly equal.
- `scripting-and-protocols`: the `GAME_DATA.zones` arc JSON schema for Team B changes to the corrected angle rule.

## Impact

- Code: `ynwa-core/src/systems/decision/scripted_decision_maker.rs` (arc branch of the per-team zone serialization).
- Tests: new companion test module for `scripted_decision_maker.rs` in `ynwa-core/src/tests/`.
- Lua contract: numeric values of `GAME_DATA.zones.<arc>.start_angle/end_angle` for Team B players. No preamble or team script uses them.
- Not affected: `ynwa-player` rendering, `ynwa-football` field construction, record/replay format (`GAME_DATA` is not recorded).

