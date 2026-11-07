# Spec Delta

## MODIFIED Requirements

### Requirement: Deterministic Execution, Script Error Fail-Fast, Tolerances, and Artifacts

Scenario execution SHALL build the world strictly (without placeholder fallback) at `temperature = 0.0` with a fixed RNG seed, evaluate `run.stop` in declaration order after each step, fail the scenario on any player Lua script error (unlike live play, which logs and continues), and always save `<home>/.ynwa/scenarios/<name>.jsonl` even when assertions fail. `YNWA_SCENARIO=<exact_name>` MUST run only that scenario (or fail listing available names).

#### Scenario: Stop satisfaction, comparison tolerances, accumulated diffs, and replay artifact
- **WHEN** a scenario finishes stepping and compares expectations
- **THEN** `steps` stop triggers at `steps >= limit` and `time` stop triggers at `elapsed_time >= limit`, positions/velocities/stat deltas compare within `1e-4` tolerance, non-empty `expect.journal` requires `1:1` parity in `"exact"` mode or ordered inclusion in `"subsequence"` mode, all mismatches are accumulated into a single readable diff report, and `<home>/.ynwa/scenarios/<name>.jsonl` remains on disk for `./replay.sh` even on failure

#### Scenario: Trailing simulation step after an event stop
- **WHEN** the first satisfied `run.stop` criterion on a step is a `when = "event"` criterion
- **THEN** the runner advances exactly one further `World::step` at the scenario `timestep` without evaluating `run.stop` criteria, reports the triggering event matcher as the stop reason, and includes that trailing step's journal entries, `total_steps`, and final state in the run outcome
