# Tasks

## 1. Compiled journal matcher

- [ ] 1.1 Add runtime types `CompiledJournalPattern`, `ResolvedOwner`, `ResolvedTarget` and `compile_journal_pattern(&JournalEntryPatternDto, &GameConfig) -> Result<CompiledJournalPattern, ScenarioError>` in `src/compare.rs`; verify unit tests cover an unknown player, a duplicate player, and malformed `cell`/`region` notation
- [ ] 1.2 Replace `entry_matches` with infallible `journal_entry_matches(&CompiledJournalPattern, &JournalEntry) -> bool`, reusing the existing field helpers; verify existing `src/tests/compare_tests.rs` still passes

## 2. Expectation path

- [ ] 2.1 Update `compare_journal` to compile each `[[expect.journal]]` entry once and match with `journal_entry_matches`; verify `cargo test -p ynwa-integration-testing compare::tests` passes

## 3. Stop criteria compilation

- [ ] 3.1 Change `compile_criteria` to take `&GameConfig` and compile `StopCriterionDto::Journal` into `JournalMatcher::new(CompiledJournalPattern)`; pass the already-cloned config from `run`; verify `cargo test -p ynwa-integration-testing runner::tests` passes
- [ ] 3.2 Change `StopCriterion::evaluate` to return `Option<StopReason>` and remove matcher-error propagation from the step loop; verify `cargo test -p ynwa-integration-testing --lib` passes
- [ ] 3.3 Add a runner test that an invalid journal stop criterion (unknown player or malformed `region`) fails `run` before stepping; verify that test passes

## 4. Verification

- [ ] 4.1 Run `cargo test -p ynwa-integration-testing` and verify all tests pass
- [ ] 4.2 Run `cargo clippy` and `cargo fmt -- --check` and verify no warnings
