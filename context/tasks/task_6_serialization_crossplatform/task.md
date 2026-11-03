# Task

Ensure cross-platform determinism of record replay.

Extracted from the serialization task (`task_3_game_serialization`).

# Discussion Results — Proposals

## 1. Cross-Platform Determinism

The only real source of spread between platforms is transcendental functions (`sin`/`cos`
in kick direction and `Arc` zones); when the cross-platform requirement appears — switch to
`libm`.

`task_3_game_serialization` guarantees determinism within a single platform; this task
introduces a cross-platform guarantee.

## 2. Required Changes

- Replace transcendental functions with a deterministic implementation (`libm`).
- Document the cross-platform guarantee.
- Tests confirming identical replay between platforms (where possible).
