# Proposal

## Why

The project's correctness depends heavily on tests because game events have high combinatorial complexity. Today tests are written alongside implementation, which invites fitting tests to the implementation after the fact. We want to anchor test quality to the specification instead: make spec scenarios the acceptance tests, and force every automated test to trace back to a scenario or requirement.

## What Changes

- Add a new `testing-discipline` capability with two durable requirements:
  - **Scenario-first acceptance testing**: externally-observable behavior is specified as spec scenarios (WHEN/THEN) in the change's spec delta before implementation; those scenarios are the acceptance tests and the source of truth.
  - **Test-to-scenario traceability**: every automated test (unit, script, and integration) traces to a spec scenario or requirement; a test that cannot be traced is removed or explicitly justified.
- No engine, crate, or existing spec behavior changes. This is a spec-only change that establishes engineering-practice requirements.

## Capabilities

### New Capabilities

- `testing-discipline`: the contract between spec scenarios and automated tests — scenario-first acceptance testing and test-to-scenario traceability.

### Modified Capabilities

<!-- none -->

## Impact

- `openspec/specs/testing-discipline/spec.md` (new capability).
- No code, crate, or dependency changes. The red-green execution discipline (write the failing test first, use `todo!()` stubs, do not weaken assertions) lives in `openspec/config.yaml` under `context` and `operations.apply.guidance`, separate from this durable-requirements change.
