# Spec Delta

## MODIFIED Requirements

### Requirement: Deterministic Execution, Script Error Fail-Fast, Tolerances, and Artifacts

Scenario execution SHALL build the world strictly (without placeholder fallback) at `temperature = 0.0` with a fixed RNG seed, compile and validate every `run.stop` criterion against the loaded `GameConfig` before the first step, evaluate `run.stop` in declaration order after each step, fail the scenario on any player Lua script error (unlike live play, which logs and continues), and always save `<home>/.ynwa/scenarios/<name>.jsonl` even when assertions fail. `YNWA_SCENARIO=<exact_name>` MUST run only that scenario (or fail listing available names).

#### Scenario: Stop satisfaction, comparison tolerances, accumulated diffs, and replay artifact
- **WHEN** a scenario finishes stepping and compares expectations
- **THEN** `steps` stop triggers at `steps >= limit` and `time` stop triggers at `elapsed_time >= limit`, positions/velocities/stat deltas compare within `1e-4` tolerance, non-empty `expect.journal` requires `1:1` parity in `"exact"` mode or ordered inclusion in `"subsequence"` mode, all mismatches are accumulated into a single readable diff report, and `<home>/.ynwa/scenarios/<name>.jsonl` remains on disk for `./replay.sh` even on failure

#### Scenario: Stop criteria are compiled and validated before stepping
- **WHEN** a scenario's `[[run.stop]]` criteria are compiled against the loaded `GameConfig`
- **THEN** journal stop criteria resolve every `{team, number}` reference and parse every `cell`/`region` target once before the first step, and an unknown or duplicate player, or an unparsable `cell`/`region` notation, fails the scenario before any step is executed
