# Code Requirements

1. Code should be as simple as possible. If a template or trait can be avoided in favor of plain structures and functions, it should be.
2. Avoid obvious comments: comment only non-obvious places. Function comments must not state where the function is called from. Comment only non-obvious things that cannot be understood from the code, and the API.
3. Use self-explanatory identifiers.
4. The project is inherently highly combinatorial, so strive for maximum test coverage, preferably parameterized. Unit tests go in separate `*_tests.rs` files next to the implementation and are included via
`#[cfg(test)] #[path = "..."] mod tests;` — this reduces implementation file size and agent context. Trivial assignments are not tested; logic, boundaries, and integration are.
In new code, tests cover all paths. Where appropriate (input conditions fundamentally affect the result), tests should be parameterized.
5. Naturally, all tests must pass.
6. `cargo clippy` and `cargo fmt` produce no warnings.
7. Type safety (e.g., `uom` instead of `f32`), validation in constructors via `assert!`,
8. Data-driven description over algorithms, idiomatic Rust (`Option`/`Result`), O(1) on hot paths,
9. YAGNI — implement only what is explicitly requested in the task or assignment. This is important! Do not invent anything beyond the specification; do not add unrequested changes. If they are needed, first discuss with a human.
10. When writing tests, reuse existing infrastructure: helpers, builders, etc. Extract them into a test-utilities file when necessary.
11. Type methods (`impl`) live in the same file where the type is declared; do not add an inherent `impl` for a type from another module/file. This does not apply to trait implementations — separate companion modules are acceptable for those.
12. Model data so that invalid states are unrepresentable. If the set of meaningful fields depends on a variant (discriminant), use an enum with fields in the variants rather than a "flat" `{ kind, field1?, field2?, ... }` structure with manual validation. Do not introduce fields that are meaningful only for some values, and do not rely on runtime checks where the invariant is expressible in the type.
13. Keep input DTOs (types deserialized from an external format — TOML/JSON, etc.) in a separate module (e.g., `dto.rs`) and name them with the `Dto` suffix (e.g., `StopCriterionDto`). Do not mix DTOs with runtime types in one module, and do not use the `Def` suffix for DTOs: `Def` is reserved for engine entity definitions (such as `PlayerDef`).
