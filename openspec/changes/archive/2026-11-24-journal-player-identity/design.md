# Design

## Context

`JournalEvent` (`ynwa-core/src/journal.rs`) has eight variants; the three that reference a specific player — `DecisionAssigned`, `KickOutcome`, and `PossessionChange` (via `possessed_by`) — carry only `player_index: usize`. `GameConfig.players[i]` already holds the invariant `team` + `number` for every index and is serialized in the recording header. Replay (`ynwa-core/src/replay.rs`) keys off `player_index` to address `player_states[i]`.

See proposal.md for motivation.

## Goals / Non-Goals

**Goals:**

- Make each player-referencing journal event self-identifying (team + tactical number) so recordings and the UI are readable without joining against the header.
- Keep `player_index` as the replay key and keep replay determinism unchanged.

**Non-Goals:**

- Replacing `player_index` with team+number as the identity (index stays canonical).
- Changing replay equivalence guarantees to cover the new fields.
- Restructuring the player UI panel (that is `events-panel-all-modes`).

## Decisions

1. **Keep `player_index` and add inline `team: Team` + `number: u32`** to `DecisionAssigned` and `KickOutcome`; for `PossessionChange` add `possessed_by_team: Option<Team>` + `possessed_by_number: Option<u32>` mirroring `possessed_by: Option<usize>`.
   - *Alternative considered:* a `PlayerRef { index, team, number }` struct used by all three variants. Rejected: it renames `player_index` and ripples through every recording site, replay match arm, and test without adding value over inline fields.
2. **Source team/number from `game.config().players[i]` at each recording site.** Config is immutable during a match, so the fields cannot drift from `player_index`.
3. **Replay ignores the new fields.** `ReplayDriver::apply_event` match arms for `DecisionAssigned`/`PossessionChange`/`KickOutcome` add `..`; behavior is unchanged.
4. **UI label format `"{team_label}{number}"`** (e.g. `A#7`). `event_text` in `ynwa-player/src/ui.rs` switches from `#<index>` to this label using the event's own `team`/`number`.

## Risks / Trade-offs

- [Wire-format change breaks existing `.jsonl` recordings] → Accepted: no backward compatibility required (project rule 11).
- [Duplicated identity data could drift from index] → Sourced from the single immutable `config.players` at record time; no drift path.
- [Larger recordings] → Negligible (two small fields per player event).
