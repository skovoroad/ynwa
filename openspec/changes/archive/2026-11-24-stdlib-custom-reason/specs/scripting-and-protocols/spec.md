# Spec Delta

## ADDED Requirements

### Requirement: Stdlib actions accept a custom reason

Decision-producing `stdlib.lua` actions that currently hard-code a `reason` SHALL accept an optional trailing `reason` argument. When omitted, the decision SHALL carry the existing built-in reason. When provided, the returned `reason` SHALL join the custom reason and the built-in reason with `" | "` so each remains independently greppable. `stop` and `kick_to_position` SHALL keep their existing verbatim `reason` behavior.

#### Scenario: Built-in reason is preserved when no argument is given
- **WHEN** a script calls an action without a `reason` (e.g. `chase_ball()`)
- **THEN** the decision's `reason` is the action's existing built-in string (e.g. `"chase_ball"`)

#### Scenario: Custom reason is recorded and remains greppable
- **WHEN** a script calls an action with a custom `reason` (e.g. `chase_ball("recover")`)
- **THEN** the decision's `reason` is the custom string joined to the built-in string by `" | "` (e.g. `"recover | chase_ball"`)

#### Scenario: Composed sub-reason is preserved alongside the custom tag
- **WHEN** a script calls `default_goalkeeper_with_ball("build_up")` and the keeper passes to teammate `7`
- **THEN** the decision's `reason` is `"build_up | pass_to_#7"`, preserving both the team's tag and the built-in pass target
