# Design

## Context

Team preambles are Lua (`teams/<id>/preamble.lua`) executed inside the per-player VM after `core.lua` + `stdlib.lua`. The current loops were introduced by `rewrite-tactics-with-dsl-loops`, which inlined the future DSL's two loop shapes: a priority `ipairs` over an explicit list, and a distance-ordered search expressed as a hand-built `{number, key, idx}` table plus `table.sort` with an index tie-breaker. The public `stdlib.lua` API is a single `do ... end` block; `teammate_numbers()` / `opponent_numbers()` already return dense, in-context-order number lists.

## Goals / Non-Goals

**Goals:**
- Hide the Lua iteration/sorting mechanics behind two public `stdlib.lua` iterator helpers used identically at the call site (`for player in <helper>(<source>) do`).
- Keep the hand-written Lua close to the future DSL's `for player in teammates ... [order by ...]`.
- Preserve exact tactical behavior (order, tie-break, pass reasons, fallbacks) across all four teams.

**Non-Goals:**
- Implementing the DSL parser/transpiler or a macro/preprocessing layer.
- Generalizing the distance ordering by target or direction; that is deferred to new named iterators when a concrete need appears.
- Building a general-purpose list/collection utility library.

## Decisions

### 1. Two public iterator helpers, one call-site shape
- **Decision**: `each(numbers)` yields each number in list order; `order_by_distance_asc(numbers)` yields numbers sorted by ascending `distance` to me, breaking ties by list order. Both return a closure usable directly in `for player in ... do`.
- **Rationale**: One uniform `for player in helper(source) do` shape means the author sees only game logic. The iterators are the single place where Lua mechanics (`ipairs`, `_`, `table.sort`, comparator, tie-break) live.
- **Alternatives considered**: higher-order `first(list, predicate)` / `nearest(list, predicate)` — rejected because passing a predicate re-exposes function syntax and loses `decision:`-style early `return`; a declarative `pick(...)` data table — rejected as a mini-DSL that re-implements the future language inside Lua.

### 2. Naming encodes intent (`each`, `order_by_distance_asc`)
- **Decision**: keep `each` for priority order; name the distance ordering `order_by_distance_asc` rather than `sorted_by_distance`/`sort_by_distance`.
- **Rationale**: `order_by_distance_asc` states the intent and echoes the DSL's `order by distance to me ascending`. The loop variable is written as `player`, matching the DSL.
- **Alternative considered**: a generic `sort_by_distance` — rejected as too vague about target/direction and about the fact that it returns an iterator, not a list.

### 3. Iterators yield a single provisional player number, not a key/value pair
- **Decision**: each iterator yields exactly one value per step (the squad number), terminating on `nil`.
- **Rationale**: removes the `_`/`(key, value)` nuance from user code. Squad numbers remain the provisional player identifier (see `scripting-and-protocols`).

### 4. Intent over transpilation
- **Decision**: these helpers are the transitional, human-readable spelling of the loop intent. We accept that the eventual DSL transpiler may emit these helpers or may inline `ipairs`/`table.sort`; neither is mandated now, and the helpers must not constrain the DSL. We are willing to add more named iterators when a distinct intent (e.g. `order by distance to opposite_goal descending`) arises.
- **Rationale**: the prior `design.md` rejected a `sort_by_distance` helper on the grounds that `order by` can target different objects and directions; that concern is real for the transpiler's output, not for a human-facing iterator whose name fixes one intent. This design re-opens that decision in a narrow, intent-scoped way.

### 5. Implementation shape
- `each(numbers)`: a closure capturing an index `i`, returning `numbers[i]` then `nil`. Sources are dense lists (`teammate_numbers()`, `opponent_numbers()`, explicit `{...}`).
- `order_by_distance_asc(numbers)`: at call time, compute `my_position()` once, build `{number, key = distance(my_pos, teammate_position(n)), idx = i}`, `table.sort` by `key` then `idx`, then lazily yield `.number`.
- Both live in the public API section of `stdlib.lua`, next to the existing queries; no engine or protocol changes.

## Risks / Trade-offs

- [Closure iterators are single-use/stateful; reusing a stored iterator in a second `for` loop yields nothing] → Recommend naming the *list* (`local forwards = {9, 10, 11}`) for reuse, or calling the helper inline in the `for` expression; cover the empty-exhaustion case in tests.
- [Eager distance computation at iterator creation] → Bounded by squad size (≤ a handful of players), negligible within the existing 100 ms decision timeout.
- [`order_by_distance_asc` bakes in "ascending, to me"] → Accept as YAGNI; a different intent gets a new named helper, not a parameter, keeping the call site declarative.
- [Hand-written Lua may diverge from future transpiler output] → Accepted: readability of the transitional Lua is the current priority; the helpers are cheap to drop if the transpiler later inlines instead.

## Test strategy

All tests are Lua assertions run through the existing `ynwa-script-tests` harness (`create_test_game_with_all_preambles` + `run_caller`), placed in `ynwa-script-tests/tests/stdlib_functions.rs`.

- **`each` order and single value** — `for player in each({10, 11})` collects into `{10, 11}`. Traces to scenario "`each` yields the list in order".
- **`order_by_distance_asc` ordering + tie-break** — place teammates at distances with one exact tie and assert the output order matches ascending distance with input order on ties. Traces to scenario "`order_by_distance_asc` sorts by ascending distance with stable tie-break".
- **Empty source** — `each({})` and `order_by_distance_asc({})` execute zero body iterations. Traces to scenario "Empty source yields no iterations".
- **Composition with full-team lists** — iterate `order_by_distance_asc(teammate_numbers())` and `each(opponent_numbers())` and assert the numbers match `teammate_numbers()` / `opponent_numbers()` (excluding me). Traces to scenario "Iterators compose with full-team number lists".
- **Behavior regression** — after rewriting the four team preambles, run `cargo test -p ynwa-script-tests`, plus a differential `before`/`after` match run (`--temperature 0`, fixed seed) for the six team pairs, to confirm identical decisions/stats. Traces to the authoring-rules scenario "DSL-aligned loop via iterator helpers" and to the requirement that behavior is unchanged.
