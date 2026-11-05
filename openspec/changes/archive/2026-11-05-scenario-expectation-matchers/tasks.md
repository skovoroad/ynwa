# Tasks

## 1. DTO types

- [x] 1.1 Add `ReasonOrNoneDto` (`String` vs `"none"`) and `PointOrNoneDto` (`{x, y, z}` vs `"none"`) untagged enums in `src/dto.rs`, and cover their TOML deserialization in `src/tests/dto_tests.rs`
- [x] 1.2 Add `DecisionTargetDto` tagged enum (`point`/`cell`/`region`/`ball`) in `src/dto.rs`, and cover its deserialization plus `deny_unknown_fields` rejection in `src/tests/dto_tests.rs`
- [x] 1.3 Extend `ExpectedEventDto::DecisionAssigned` (`target`, `reason: ReasonDto`), `ExpectedEventDto::RestartSet` (`position: PointOrNoneDto`), and `SetupDto` (`restart_position: PointOrNoneDto`); update `validate` and the DTO tests for the new shapes

## 2. Loader mapping

- [x] 2.1 In `src/loader.rs` map `PointOrNoneDto` for `initial_state.setup.restart_position` to `PartialSetupSpec.restart_position` (`"none"` becomes `None`), and verify with a loader test in `src/tests/loader_tests.rs`

## 3. Matchers

- [x] 3.1 Implement `decision_target_matches` in `src/compare.rs` comparing `DecisionTargetDto` against `Decision::Run(DecisionTarget)` and `Decision::Kick(Point3D)`, with unit tests in `src/tests/compare_tests.rs` for every target variant and `Stop`
- [x] 3.2 Update `entry_matches` to compare `target` and to treat `reason = "none"` as "the recorded reason is absent", with unit tests
- [x] 3.3 Update `compare_final_setup` so `restart_position = "none"` asserts `actual.restart_position` is `None`, with unit tests

## 4. Verification

- [x] 4.1 Run `cargo test -p ynwa-integration-testing` and verify all unit and scenario tests pass
- [x] 4.2 Run `cargo clippy` and `cargo fmt -- --check` and verify no warnings
