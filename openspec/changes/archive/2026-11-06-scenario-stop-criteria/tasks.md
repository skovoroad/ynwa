# Tasks

## 1. Stop-criterion DTOs

- [x] 1.1 Add a `journal` variant to `StopCriterionDto` and `StopExpectationDto` in `src/dto.rs` reusing the journal matcher fields, with deserialization tests in `src/tests/dto_tests.rs`
- [x] 1.2 Update `RunPlanDto::validate` and `StopExpectationDto::validate` for the new variant, with validation tests

## 2. Criterion domain

- [x] 2.1 Add a journal-event matcher and `StopCriterion::OnJournal` / `StopReason::Journal` in `src/criterion.rs`, with unit tests

## 3. Runner and comparison

- [x] 3.1 In `src/runner.rs`, evaluate journal stop criteria against the per-step journal entries, and verify with a runner test in `src/tests/runner_tests.rs`
- [x] 3.2 In `src/compare.rs::compare_stop`, handle the `Journal` stop reason, and verify with a compare test in `src/tests/compare_tests.rs`

## 4. Verification

- [x] 4.1 Run `cargo test -p ynwa-integration-testing` and verify all tests pass
- [x] 4.2 Run `cargo clippy` and `cargo fmt -- --check` and verify no warnings
