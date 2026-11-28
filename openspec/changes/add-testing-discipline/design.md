# Design

## Context

The project already mandates heavy, parameterized test coverage (`openspec/config.yaml` code-style rules 4, 5, 7), deterministic integration scenarios (`integration-testing` spec), and a scenario-first authoring convention (code-style rules 13-15). This change promotes two of those conventions — scenario-first acceptance and test-to-scenario traceability — into durable, checkable requirements.

## Goals / Non-Goals

**Goals:**
- Make scenario-first acceptance testing and test-to-scenario traceability normative requirements that survive review and archive.
- Keep the requirements minimal and behavior-level, so they remain meaningful rather than aspirational.

**Non-Goals:**
- No enforcement tooling (linter, CI check, or script) in this change.
- No retrofit of the existing active changes; the requirements apply to new changes.
- No red-green execution machinery in the spec — that is a temporal process discipline and stays in `config.yaml`.

## Decisions

### 1. New `testing-discipline` capability instead of extending `integration-testing`
- **Decision**: Create a new capability that owns the contract between specs and tests.
- **Rationale**: Test-to-scenario traceability spans unit (`*_tests.rs`), script (`ynwa-script-tests`), and integration (`ynwa-integration-testing`) tests, not just the integration framework. `integration-testing` is narrowly about the TOML scenario format and matching rules; folding process requirements into it would blur that purpose.
- **Alternative considered**: Extending `integration-testing`. Rejected because it would couple a cross-cutting discipline to one test level's format.

### 2. Red-green discipline stays in `config.yaml`, not in the spec
- **Decision**: "Write the failing test first, use `todo!()` stubs, do not weaken assertions" is recorded only in `config.yaml` (`context` rule 13 and `operations.apply.guidance`), not as a spec requirement.
- **Rationale**: A spec requirement is a durable contract about observable behavior and artifact structure. Temporal ordering ("test before implementation, observed red") is execution guidance, not a durable property; encoding it in the spec would make the spec untestable as a contract.

### 3. Two requirements, no broader "test everything" clause
- **Decision**: Limit the capability to scenario-first acceptance and traceability.
- **Rationale**: YAGNI. Both are individually checkable and directly serve the stated goal (anchor tests to the spec, avoid fitting tests to implementation).

### 4. Traceability is a convention, not yet a machine-checkable format
- **Decision**: Require a test to "reference" the scenario or requirement it verifies (doc comment or test name), without prescribing an exact annotation syntax.
- **Rationale**: A rigid syntax would need enforcement tooling, which is explicitly out of scope. The convention can harden into a checked format in a follow-up change.

## Risks / Trade-offs

- [Spec-only change reads as process, not behavior] -> Precedent exists: the `architecture` spec already holds engineering-practice constraints (crate graph, isolation). This capability is analogous.
- [Traceability is advisory, not enforced] -> Checked at review and verify/archive; a machine check can be added later without changing these requirements.
- [Existing active changes predate the requirement] -> Applied to new changes only; do not retrofit.

## Test strategy

This change introduces no code and therefore no internal-boundary or edge-case tests. The requirements themselves are the acceptance contract and are validated at review and verify/archive (a behavior change has scenarios before implementation; each test traces to a scenario or requirement). For future code changes, internal-boundary and edge-case test intent continues to live in that change's `design.md` "Test strategy" and `tasks.md` (config code-style rule 15).
