# Spec Delta

## ADDED Requirements

### Requirement: Player-referencing journal events are self-identifying

Every `JournalEvent` variant that references a player SHALL record that player's `team` and tactical `number` alongside the canonical `player_index`. `player_index` SHALL remain the key replay uses to target `player_states`, and the added fields SHALL NOT participate in replay equivalence guarantees.

#### Scenario: A decision names its player without the config
- **WHEN** a `DecisionAssigned` event is recorded for global player index `i`
- **THEN** the serialized event contains `player_index = i` together with `team` and `number` equal to `config.players[i].team` and `config.players[i].number`

#### Scenario: Possession change names the new possessor or the free ball
- **WHEN** a `PossessionChange` event is recorded
- **THEN** when `possessed_by` is `Some(i)` the event also carries the `team` and `number` of player `i`; when `possessed_by` is `None` the added `team` and `number` fields are `None`

#### Scenario: Kick outcome names the kicker
- **WHEN** a `KickOutcome` event is recorded for global player index `i`
- **THEN** the serialized event contains `player_index = i` together with `team` and `number` equal to `config.players[i].team` and `config.players[i].number`
