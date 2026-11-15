# Spec Delta

## MODIFIED Requirements

### Requirement: Lua VM Lifecycle and Stage Gating

At `DecisionEngine::new`, each VM SHALL execute `"{core_preamble}\n{stdlib_preamble}\n{team_preamble}"` **once** and bind `static_data` **once** to global `GAME_DATA`. On each `DecisionEngine::make_decision(idx, context)` call, the VM MUST overwrite global `context`, re-execute `script.lua`, and invoke `make_decision()`. `ScriptedDecisionMaker` MUST delegate in `Play` and return `Err(RuntimeError)` in `Setup(_)`.

#### Scenario: Preamble persistence vs player script re-execution
- **WHEN** `DecisionEngine::make_decision` runs across consecutive ticks for the same player
- **THEN** tables/functions declared in `core.lua`, `stdlib.lua`, and team `preamble.lua` persist across ticks within that player's VM, while top-level statements in `script.lua` re-execute on every call before `make_decision()` runs
