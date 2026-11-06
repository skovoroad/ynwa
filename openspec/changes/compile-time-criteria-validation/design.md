# Design

## Context

Journal stop criteria are compiled from `StopCriterionDto::Journal(JournalStopPatternDto)` into `StopCriterion::OnJournal(JournalMatcher)`. `JournalMatcher` currently stores the DTO pattern and delegates matching to `compare::journal_pattern_matches`, which calls `entry_matches`. The fallible steps inside `entry_matches` are:

- `player_matches` / `owner_matches` → `loader::resolve_player_index` (unknown or duplicate player).
- `decision_target_matches` → `GridCell::from_notation` / `Region::from_grid_notation` (malformed notation).

Those are static configuration errors, but they are evaluated on every step inside `StopCriterion::evaluate`, which therefore returns `Result<Option<StopReason>, ScenarioError>`. `run()` already clones `world.game().config()` before stepping, so the config is available when criteria are compiled.

## Goals / Non-Goals

**Goals:**

- Resolve and validate journal stop patterns once, at `compile_criteria` time.
- Make `StopCriterion::evaluate` infallible (`Option<StopReason>`), removing `?` from the step loop.
- Keep one journal matcher shared by stop criteria and `[[expect.journal]]` comparison (no duplicated dispatch).

**Non-Goals:**

- No change to the TOML schema or `dto.rs` DTOs.
- No change to matching semantics, tolerances, or the accumulated-diff reporting.
- No change to how `[[run.stop]]` evaluation is ordered or how replay artifacts are written.

## Decisions

1. **Introduce a compiled journal pattern.** Add a runtime (non-DTO) `CompiledJournalPattern` that stores resolved values instead of TOML references:
   - `player` fields resolve `Option<PlayerRefDto>` → `Option<usize>`.
   - `possessed_by` resolves `Option<BallOwnerDto>` → `ResolvedOwner` (`Any` | `Free` | `Player(usize)`).
   - `target` resolves `Option<DecisionTargetDto>` → `Option<ResolvedTarget>` (`Point` | `Cell(GridCell)` | `Region(Region)` | `Ball`).
   - `football_event` precomputes `EventMatcher`.
   - `at: Option<f32>` is carried verbatim for expectation timestamp checks (always `None` for stop patterns).

   *Alternative rejected:* resolve inside the matcher per call (status quo) — keeps the hot loop fallible and repeats work.

2. **One compile function, one infallible matcher.** Add `compile_journal_pattern(&JournalEntryPatternDto, &GameConfig) -> Result<CompiledJournalPattern, ScenarioError>` and `journal_entry_matches(&CompiledJournalPattern, &JournalEntry) -> bool`. The existing infallible field helpers (`decision_field_matches`, `reason_matches`, `velocity_field_matches`, `team_matches`, `stage_matches`, etc.) are reused unchanged; only the three fallible helpers are absorbed into compilation.

   *Alternative rejected:* compile only stop patterns and keep `entry_matches` for expectations — would duplicate the match dispatch instead of sharing it.

3. **Compile stop criteria with the config in scope.** Change `compile_criteria` to `compile_criteria(run: &RunPlanDto, config: &GameConfig)`. `run()` passes the `config` it already clones. `JournalMatcher` holds a `CompiledJournalPattern`; `StopCriterion::evaluate` returns `Option<StopReason>`.

   *Alternative rejected:* keep `compile_criteria` config-free and validate lazily — that is the current fallible design this change removes.

4. **Expectation matching reuses the compiled matcher.** `compare_journal` compiles each `[[expect.journal]]` entry once (with `?` inside `compare_outcome`) and then matches with the infallible `journal_entry_matches`. This removes repeated resolution from the expectation path too and keeps a single source of truth.

## Risks / Trade-offs

- [Refactor touches the `[[expect.journal]]` path in `compare.rs`] → Behavior is unchanged and existing compare tests cover exact/subsequence matching and hard errors; the only difference is errors surface from compilation rather than per comparison.
- [Invalid journal stop criterion now fails before the first step instead of mid-run] → Intended; add a runner/compile test asserting `run()` returns the error before stepping.
- [Compiled type adds a parallel structure to the DTO] → Mitigated by keeping the compiled type minimal (resolved values only) and deriving it solely from the DTO through one compile function.

## Migration Plan

None. This is an internal refactor with no scenario format, config, or data migration.
