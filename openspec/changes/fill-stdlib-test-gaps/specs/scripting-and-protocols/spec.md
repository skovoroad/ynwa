# Spec Delta

## ADDED Requirements

### Requirement: Standard Library Test Coverage

Every public `stdlib.lua` function SHALL have a co-located test in `ynwa-script-tests` that asserts the exact `Decision` and `reason` it produces (or, for pure queries and predicates, its exact return value). Adding or changing a public `stdlib.lua` function MUST add or update its test in the same change.

#### Scenario: A new standard-library function is introduced
- **WHEN** a public function is added to `stdlib.lua`
- **THEN** a test in `ynwa-script-tests` exercising it is added in the same change, and `cargo test -p ynwa-script-tests` covers it

#### Scenario: A standard-library function's behavior changes
- **WHEN** the semantics of an existing public `stdlib.lua` function are modified
- **THEN** its test in `ynwa-script-tests` is updated in the same change to assert the new behavior
