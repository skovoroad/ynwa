# Design

## Context

The fixed per-step quantum is threaded through several layers under inconsistent names (see
proposal.md - Why): `dt` in scenario TOML and the integration-test DTO/comparison layer,
`fixed_dt` in `RecordHeader` and the recording codec, `step_delta` in the player's
`SimulationControl`, and `delta_time` in `World::step`/`Game::step`. Two neighboring concepts must
keep their existing names: `PhysicsSystem`'s internal `delta_time = timestamp - last_update`
(computed each tick) and the player's real-time tempo (`rate`, `pace()`, `accumulate(delta_time)`).

The serialized field name and the Rust struct field name are coupled by `serde` derives and
`deny_unknown_fields`, so the rename must be applied to schema, DTO, core, player, and test
helpers together rather than piecemeal.

## Goals / Non-Goals

**Goals:**

- Rename exactly the first quantity (the fixed per-step quantum) to `timestep`, end to end.
- Keep the change a pure identifier/field rename with no behavior, physics, or replay change.

**Non-Goals:**

- Do **not** rename `PhysicsSystem`'s internal `delta_time` (the second term).
- Do **not** rename the player tempo terms `rate`, `pace()`, or `accumulate(delta_time)` (the third term).
- Do **not** add back-compat aliases for the old `dt`/`fixed_dt` names.

## Decisions

### D1 — One target name for one quantity: `timestep`

Use `timestep` everywhere the fixed quantum appears. `timestep` is the standard discrete-simulation
term for "fixed quantum", avoids the false `fixed_dt` vs "non-fixed dt" opposition, and is distinct
from `delta_time` (the computed physics delta).

Alternatives: `step_duration` (clear but longer; `run.timestep` reads better than
`run.step_duration`), `step_delta` (still confusable with the physics delta). Rejected.

### D2 — Rename schema and code atomically, no compat shim

The scenario TOML field (`[run].dt` → `[run].timestep`), the recording header field
(`"fixed_dt"` → `"timestep"`), and all Rust identifiers change in one commit. Old on-disk files are
not accepted via aliases because `deny_unknown_fields` would reject them anyway and the project is
pre-1.0.

### D3 — Boundary between `World::step`/`Game::step` and `PhysicsSystem`

`World::step(&mut self, delta_time)` and `Game::step(&mut self, delta_time)` take the *caller-supplied
fixed quantum*, so their argument becomes `timestep`. `PhysicsSystem::update` keeps its local
`delta_time = timestamp - self.last_update`, which is a *computed* delta and remains the second term.

### D4 — Sweep residual identifiers, docs, and spec prose

The rename also covers local variable names (`run.dt`, `plan.dt`, `fixed_dt`, `step_delta`),
doc comments, docstrings, and the `recording-and-replay` spec Purpose prose that names the
`RecordHeader { config, initial_state, fixed_dt }` shape. A final grep for `fixed_dt`, `step_delta`,
and the standalone `dt`/`delta_time` (where it means the step quantum) catches stragglers.

## Risks / Trade-offs

- [Old `scenario.toml` files (`dt =`) and `.jsonl` recordings (`"fixed_dt"`) stop deserializing] →
  Pre-1.0 breaking change; update the six scenario fixtures and regenerate/ignore old recordings.
- [A `dt`/`fixed_dt`/`step_delta` reference is missed] → A dedicated grep task plus `cargo test` and
  `cargo clippy` gate the change.
- [`core-simulation`'s physics requirement still spells its internal delta `dt` while code uses
  `delta_time`] → Out of scope (second term); leave unchanged and do not confuse it with `timestep`.

## Migration Plan

1. Rename the serialized fields and code identifiers per proposal.md.
2. Update all `scenario.toml` files (`[run] dt` → `[run] timestep`) and the integration-test fixtures.
3. Re-record any `.jsonl` artifacts relied on by tests/docs; old recordings are not readable.
4. Verify with `cargo test`, `cargo clippy`, and `cargo fmt`.
