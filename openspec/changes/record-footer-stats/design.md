# Design

## Context

See `proposal.md` for motivation. Currently in `ynwa-core`:
- `Game::finish_journal(&mut self)` detaches `self.journal_sink` and calls `sink.finish()`, which takes no arguments.
- `FileJournalRecorder::finish` and `CollectJournalRecorder::finish` only pass `self.steps` (`total_steps: u64`) to `RecordWriter::finish(total_steps)` and `EventsCollection::set_total_steps(steps)`.
- `RecordLine::Footer` in `ynwa-core/src/codec.rs` only serializes `{ total_steps: u64 }`.

## Goals / Non-Goals

**Goals:**
- Capture final `elapsed_time`, `team_stats`, and `player_stats` from `GameState` when `Game::finish_journal()` is called and write them into the `Footer` of the recording.
- Respect layer boundaries: `Game` and `JournalSink` operate solely on domain types (`&GameState`), while `RecordFooter` lives in the recording/codec layer (`record.rs` / `codec.rs`) as the symmetric counterpart to `RecordHeader`.
- Group all trailer fields in `Record { header: RecordHeader, footer: RecordFooter, journal: Vec<JournalEntry> }` cleanly without legacy backward-compatibility shims.
- Ensure deterministic JSON key ordering for `team_stats` in the `footer` line.

**Non-Goals:**
- Backward compatibility with older `.jsonl` footer formats or keeping a deprecated top-level `Record::total_steps` field.
- Implementing specific new football metrics (e.g., xG, possession percentage, distance run) in this change; this change builds the recording footer transport so any stat added to `GameState::team_stats` or `player_stats` is automatically persisted.

## Decisions

1. **Strict layer separation between `JournalSink::finish` (`&GameState`) and `RecordWriter::finish` (`&RecordFooter`)**:
   - **Domain / Journal layer (`game.rs` <-> `journal.rs`)**: `Game` does not know about `Record`, `RecordHeader`, or `RecordFooter`, nor does it track `total_steps`. `Game::finish_journal(&mut self)` passes `&self.state` to `JournalSink::finish(self: Box<Self>, final_state: &GameState)`.
   - **Sink implementation layer (`FileJournalRecorder` / `CollectJournalRecorder`)**: Owns the accumulated `self.steps` counter and receives `final_state: &GameState` on `finish`. `FileJournalRecorder::finish` constructs `RecordFooter::from_state(self.steps, final_state)` and passes `&footer` to `self.writer.finish(&footer)`. `CollectJournalRecorder::finish` stores `self.steps` and the final stats from `final_state` into `EventsCollection`.
   - **Recording / Codec layer (`record.rs` <-> `codec.rs`)**: Defines `RecordFooter { pub total_steps: u64, pub elapsed_time: f32, pub team_stats: HashMap<Team, StatSet>, pub player_stats: Vec<StatSet> }` as the symmetric trailer counterpart to `RecordHeader`, and updates `RecordWriter::finish(&mut self, footer: &RecordFooter)`.

2. **Symmetric `Record` structure (`header`, `footer`, `journal`)**:
   - Replace `pub total_steps: u64` on `Record` with `pub footer: RecordFooter` so `Record` mirrors the 3-part stream structure (`header`, `journal`, `footer`). All call sites (`ynwa-player`, `ynwa-integration-testing`, tests) are updated to `record.footer.total_steps`.
   - In `RecordLine::Footer(RecordFooter)`, serialize `team_stats` via `#[serde(serialize_with = "crate::game::sorted_collections::map")]` (making `sorted_collections` `pub(crate)`) for deterministic output. No `#[serde(default)]` shims are added for legacy files; when `JsonRecordReader::read` recovers a crashed recording that has no `"footer"` line at all, `footer` defaults to `RecordFooter::default()`.

## Risks / Trade-offs

- **[Risk] Interrupted/crashed recordings have no footer line** → Mitigation: By design, crashed recordings recover up to the last flushed event with `total_steps = 0`, `elapsed_time = 0.0`, and empty footer stats; normal completions (including early `Escape` exit in `ynwa-player --record` and `GameOver` in `ynwa-simulator`) call `finish_journal()` and write the full footer.

