# Tasks

## 1. DTO

- [ ] 1.1 Add a player-placement DTO variant (exact `position` vs `region` string) for `final_state.players` in `src/dto.rs`, with deserialization tests in `src/tests/dto_tests.rs`

## 2. Matcher

- [ ] 2.1 In `src/compare.rs::compare_final_state`, support region entries by parsing the notation and asserting `Region::contains_point`, with unit tests in `src/tests/compare_tests.rs`

## 3. Verification

- [ ] 3.1 Run `cargo test -p ynwa-integration-testing` and verify all tests pass
- [ ] 3.2 Run `cargo clippy` and `cargo fmt -- --check` and verify no warnings
