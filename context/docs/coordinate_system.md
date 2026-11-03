# Coordinate System and General Conventions

All project layers share one coordinate system and one key convention about "orientation" —
from whose perspective coordinates are written. Understanding this document is required for
reading any project code.

- Right-handed coordinate system, Y up:
  - **X** — field width,
  - **Y** — height above the pitch (3D physics, 2D top-down rendering),
  - **Z** — field length; Team A's goal at Z ≈ 0, Team B's goal at Z ≈ length.
- "Display orientation" (canonical) = Team A's perspective. All internal game state
  is stored only in it.
- Team B's perspective is obtained by mirroring: `x' = width - x`, `z' = length - z`, `y' = y`.
  Mirroring is applied **only at the decision-system boundaries**:
  the input (context/GAME_DATA/regions) is mirrored, the output (decision) is mirrored back.
  This allows both teams' scripts to be written identically.
- Physical quantities are `uom` (`Length`, `Velocity`, `Angle`), not bare `f32`.
- Field grid: columns along X, rows along Z, 1-based numbering, columns in Excel notation
  (A=1 … Z=26, AA=27). A cell is `"M22"`, a region is `"A1:B2"`.

Companion documents: [`project_overview.md`](project_overview.md) (general project context),
[`game_step.md`](game_step.md) (one simulation step), [`protocols.md`](protocols.md)
(data formats, including grid notation).
