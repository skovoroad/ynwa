# Tasks

## 1. Resolve the approach

- [ ] 1.1 Decide between engine precision (snap the `on_ball` taker to the restart point) and framework tolerance (a final-state `within`/radius assertion); record the decision in `design.md` and add the corresponding spec delta, removing `skip_specs`

## 2. Implementation (after decision)

- [ ] 2.1 Implement the chosen approach and add unit tests; verify `cargo test` passes
- [ ] 2.2 Update the placement scenario fixtures to assert the taker's arrival on the restart point, and verify `cargo test -p ynwa-integration-testing` passes
