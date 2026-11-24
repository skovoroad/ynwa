# Proposal

## Why

Match recordings and the player UI identify players by their internal global index (`player_index: usize`), not by tactical number + team. A viewer cannot correlate a recorded event with the player shown on screen, and external analysis of a `.jsonl` recording must join against `RecordHeader.config` to resolve identities.

## What Changes

- Embed `team` and tactical `number` in every `JournalEvent` variant that references a player: `DecisionAssigned`, `KickOutcome`, and the `PossessionChange` possessor. **BREAKING** (wire-format change; no backward compatibility required).
- Keep `player_index` as the canonical replay key; the added `team`/`number` fields do not affect replay equivalence.
- Render the replay events panel using `team` + `number` (e.g. `A#7`) instead of the raw index.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `recording-and-replay`: journal events that reference a player now self-identify that player by `team` and tactical `number` in addition to `player_index`.

## Impact

- `ynwa-core` — `journal.rs`, `systems/decision/decision_system.rs`, `systems/action.rs`, `systems/ball_possession.rs`, `replay.rs`: wire type and recording sites.
- `ynwa-core` tests — `codec_tests.rs`, `journal_tests.rs`, `replay_tests.rs`, `decision_system_tests.rs`, `action_tests.rs`, `ball_possession_tests.rs`.
- `ynwa-player` — `ui.rs`: event label rendering.
