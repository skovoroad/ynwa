# Task

Implement record-format versioning and version checks on deserialization.

Extracted from the serialization task (`task_3_game_serialization`).

# Discussion Results — Proposals

## 1. Versioning

The record envelope contains:

- `format_version` — the journal/snapshot schema version (serialization).
- `engine_version` — the simulation semantics version (`ynwa-core` + `ynwa-football`), maintained
  manually and bumped on any behavior change (rules, constants, formulas). Not a code hash —
  that gives false failures on refactoring.
- The embedded [`GameConfig`](ynwa-core/src/game.rs:228) — the record is self-contained and does not depend on
  changed team/preamble files.

On deserialization — a failure on `engine_version` mismatch with an explicit message; optionally a
forced-read mode for debugging.

Snapshots do not fix logical version divergence — versioning closes that.

## 2. Required Changes (extracted from task_3)

- `format_version`/`engine_version` fields in the `Record` model.
- Constants `FORMAT_VERSION`, `ENGINE_VERSION`, and `Record::validate_versions()`.
- A forced-read mode (`RecordCodec`/API with `force`).
