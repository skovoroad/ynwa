# Rendering Coordinate System

## Field → Screen Mapping

Field uses Y-up right-handed coordinates: X = width (left→right), Z = length (Team A goal → Team B goal).

Screen mapping (defined in `render_field`):
```
to_screen_x = |field_x| offset_x + field_x * scale   // field X → screen X (horizontal)
to_screen_y = |field_z| offset_y + (field_length - field_z) * scale  // field Z → screen Y (inverted)
```

**Note:** Despite the parameter name `field_z` in the closure definition, `to_screen_x` is called with `field_x` values and `to_screen_y` with `field_z` values throughout the codebase. The names are misleading — treat them as `to_horizontal(x)` and `to_vertical_inverted(z)`.

Result: field Z=0 (Team A goal) is at the **top** of the screen, field Z=length (Team B goal) is at the **bottom**.

## Angle Conversion for Arcs

Field angles: standard math convention, CCW, 0 = +X, PI/2 = +Z (= downward on screen).

Screen Y is inverted, so field angle `a` maps to screen angle `-a`. CCW in field becomes CW on screen.

`draw_arc(x, y, sides, radius, rotation_deg, thickness, arc_deg, color)` — draws CCW from `rotation` by `arc` degrees (both in degrees, `arc` must be positive).

**Conversion rule:**
```
screen_rotation = -end_field   // start from the negated end angle
screen_arc = end_field - start_field  // span (ensure positive; add TAU if negative)
```

This renders the field arc [start..end] correctly by going CCW on screen from `-end` to `-start`, which corresponds to CW from `-start` to `-end` — matching the field direction after Y-flip.

**Critical:** `span` must be positive for `draw_arc` to work. If `end < start` in field coords, the arc will produce `span + TAU` (wrong — nearly full circle). Always define arcs in field data with `start < end`.

## Arc Definitions in field_builder.rs

All arcs must have `start < end` to ensure positive span in the renderer.

**Penalty arcs** — center at penalty spot, radius 9.15m, only the portion outside the penalty area:
- Half-angle from +X axis: `dz = penalty_area_length - penalty_spot_distance`, `dx = sqrt(r²-dz²)`
- Team A: `start = atan2(dz, dx) ≈ 34°`, `end = atan2(dz, -dx) ≈ 146°` — arc faces toward center (away from Team A goal, i.e. toward +Z)
- Team B: `start = atan2(-dz, -dx) ≈ -146°`, `end = atan2(-dz, dx) ≈ -34°` — arc faces toward center (away from Team B goal, i.e. toward -Z)

**Corner arcs** — center at corner flag, radius 1.0m, quarter-circle pointing into the field:
- Team A bottom-left `(0, 0)`:   `0 → PI/2`
- Team A top-right `(W, 0)`:     `PI/2 → PI`
- Team B bottom-left `(0, L)`:   `-PI/2 → 0`
- Team B top-right `(W, L)`:     `PI → 3PI/2`
