## MODIFIED Requirements

### Requirement: `LuaDecision` Return Schema and Validation

Lua `make_decision()` SHALL return a table deserializable into `LuaDecision` (`action` in `"stop" | "run" | "kick"`) with optional top-level string `reason`, then parsed into domain `Decision` (still in team orientation). Invalid actions, missing targets, unknown `target_type` (including `"ball"`), non-finite coordinates, or malformed grid notations MUST return an error.

#### Scenario: Valid Lua decision variants
- **WHEN** `make_decision()` returns a decision table
- **THEN** `{action="stop", reason?}` -> `Decision::Stop`; `{action="run", target_type="point", target={x, z, y?=0}, reason?}` -> `Run(Point)`; `{action="run", target_type="cell", target="M10", reason?}` -> `Run(GridCell)`; `{action="run", target_type="region", target={from="A1", to="C3"}, reason?}` -> `Run(Region)`; and `{action="kick", target={x, z, y?=0}, reason?}` -> `Decision::Kick`

#### Scenario: Removed `"ball"` target_type rejected
- **WHEN** `make_decision()` returns `{action="run", target_type="ball"}`
- **THEN** decision parsing returns an error indicating an unknown `target_type`

