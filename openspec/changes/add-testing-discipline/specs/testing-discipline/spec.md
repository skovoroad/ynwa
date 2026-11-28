# Spec Delta

## Purpose

Defines the engineering-practice requirements that bind spec scenarios and automated tests: scenario-first acceptance testing and test-to-scenario traceability.

## ADDED Requirements

### Requirement: Scenario-First Acceptance Testing

Externally-observable behavior SHALL be specified as spec scenarios (WHEN/THEN) in the change's spec delta before implementation. Spec scenarios are the acceptance tests and the source of truth for the implementation.

#### Scenario: Behavior is specified before implementation

- **WHEN** a change modifies externally-observable behavior
- **THEN** the change's spec delta contains scenarios describing the new or changed behavior before any implementation task is marked complete

#### Scenario: Scenarios are not silently weakened

- **WHEN** an implementation conflicts with a spec scenario
- **THEN** the scenario is revised deliberately as a spec change and the conflict is discussed, rather than the scenario being weakened to match the implementation

### Requirement: Test-to-Scenario Traceability

Every automated test (unit, script, and integration) SHALL trace to a spec scenario or requirement that it verifies. A test that cannot be traced to a scenario or requirement SHALL be removed or explicitly justified.

#### Scenario: A new test cites its scenario

- **WHEN** a new automated test is added
- **THEN** the test references the scenario or requirement it verifies (for example, a doc comment or test name citing the requirement or scenario)

#### Scenario: An untraced test is removed or justified

- **WHEN** an automated test cannot be traced to any spec scenario or requirement
- **THEN** the test is removed or its justification is recorded in the change
