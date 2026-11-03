# Processing One Game Step (core level)

This document describes what happens inside one `World::step(delta_time)` call: in what order
the systems run, how a player's decision is born, how it turns into movement, and how events
and phase changes intervene in this flow.

Everything described applies to `ynwa-core` and does not depend on football. The football layer hooks in at exactly
one point — as the first system in the list (see §2).

Companion documents: [`project_overview.md`](project_overview.md) (general project context),
[`coordinate_system.md`](coordinate_system.md) (coordinate conventions).

---

## 1. Entry Point: `World::step(delta_time)`

The game loop belongs to the client — the core does not run its own timer. The client decides with what
step and how often to call `step`. This is the source of determinism: with the same
`delta_time` sequence the simulation is reproducible (up to RNG, see §7).

The step is extremely simple:

```
new_timestamp = game.state.elapsed_time + delta_time
for each system in insertion order:
    system.update(&mut game, new_timestamp)
game.state.elapsed_time = new_timestamp
```

Two non-obvious decisions are baked in here:

- **Systems receive `&mut Game`, not `&mut World`.** Otherwise the borrow checker would not allow
  iterating the system list while handing each the whole world.
- **Systems receive an absolute `timestamp`, not `delta_time`.** Each system decides for itself
  what "time has passed" means: `PlayerReactionSystem` compares against the player's `last_decision_time`,
  `BallPossessionSystem` — against the ball's `last_possession_change_time`, `PhysicsSystem` keeps its own
  `last_update` and derives dt from it.
- **`elapsed_time` is updated only at the very end.** That is, all systems within one step see
  the still *old* `game.state.elapsed_time` but receive the *new* `timestamp` as an argument. If a system
  needs the current step time — it must take the argument, not the state field.

State is passed between systems via the shared `GameState`: there are no message queues;
systems communicate through flags and fields (`needs_decision`, `decision_processed`, `is_ready`,
`current_decision`). System order is part of the contract, not an implementation detail.

---

## 2. System Order Within a Step

```
1. Game manager (sport-specific, e.g. FootballGameManager)  — phases and events
2. PlayerReactionSystem                                     — "whose turn to think"
3. BallPossessionSystem                                     — who possesses the ball
4. DecisionSystem                                           — what the player decided
5. ActionSystem                                             — decision → velocity/kick
6. PhysicsSystem                                            — velocity → position
```

The logic of the order is the "rules → intent → action → world" pipeline:

- the manager runs **first**, so decisions are made in the current game phase, not the one
  from the previous step;
- ball possession is determined **before** decisions: the player must decide knowing who has the ball
  right now;
- physics runs **last**, so everything accumulated during the step (new velocities, a kick on the ball)
  applies to positions once and consistently.

The core itself does not impose the system list — the game layer assembles it. But the described order is
the only one under which the pipeline works without a one-frame delay at each link.

---

## 3. Step 1: Game Manager — Phases and Events

The core knows phases exist but not their meaning. Phases are described by the `GameStage` enum:

| Phase | Meaning for the core |
|---|---|
| `Play` | Normal play. Scripts are called, ball possession is computed, everything works. |
| `Setup(String)` | Preparation/positioning. The string is the reason (for football: `"kick off"`, `"throw in"`, `"corner"`, `"goal kick"`); the core does not interpret it, just carries it. |
| `GameOver` | The match is over. |

What a phase means **for the core** is a set of exactly three behavioral differences, baked into the systems:

1. `BallPossessionSystem` does nothing at all in `Setup` (the ball is pinned, possession is meaningless).
2. `PlayerReactionSystem` in `Setup` does not use the reaction timer: a decision is requested only from
   a player who does not yet have one.
3. `DecisionSystem` in `Setup` **does not call scripts**. Decisions in this phase are handed out by the game manager
   directly. The scripted `DecisionMaker`, if called in `Setup`, deliberately returns an error with
   an explicit message — that is a sign of a caller bug, and it must not be masked.

Everything else (who, where, and why is positioned in `Setup`; when the phase changes; what counts as
a goal) is the responsibility of the sport-specific manager.

### Events

There is no event type in the core: `FootballEvent` and its handling live in the game layer. For the core,
an event is just a set of `GameState` changes the manager makes at the start of the step.
What matters is **which fields** the manager is allowed to touch and how that breaks the downstream pipeline:

- `state.stage` — phase change; affects all three points above already in this same step;
- `state.restart_position` / `state.restart_team` — where and whose restart; the core only stores these fields;
- resetting `current_decision = None`, `is_ready = false`, `needs_decision = true` on players — this is
  a way to tell the pipeline "previous intents are invalid, think again";
- `ball_state` — forced position/velocity/possession setting;
- `team_stats` / `player_stats` — counters (`StatSet`); the game layer defines their keys, the core does not
  understand them and does **not pass them to Lua**.

An important consequence of the order: since the manager runs first, an event detected in this step
(e.g., a goal) fully takes effect before the players in this same step start making
decisions. No one-frame delay arises.

### The `Setup → Play` Phase Change

The core does not switch phases itself. The mechanics (implemented in the football manager, but relying on
core fields) is as follows: a player is marked `is_ready` when their current decision has become `Stop`. And `Stop`
appears automatically as soon as the player has reached the target (§4.1). That is, "everyone in place"
is expressed through the already existing arrival mechanism, not through a separate distance check.

---

## 4. Steps 2–4: Birth of a Decision

Decision-making is split across three systems: "is it time to think" (`PlayerReactionSystem`), "what is known
about the ball" (`BallPossessionSystem`), and "what was decided" (`DecisionSystem`). The split is deliberate:
coordination (when) is separated from strategy (what).

### 4.1 `PlayerReactionSystem` — whose turn to think

Its only task — raise the `needs_decision` flag.

- In `Play`: the flag is raised if the player's reaction interval has passed since `last_decision_time`.
  The interval depends linearly on the `reaction_rate` attribute: `0.5 s` at 100 and `3.0 s` at 10.
  This is the player's "slowness" — a slow player reacts to changed circumstances with a delay
  of up to three seconds.
- In `Setup`: the timer is not used. The flag is raised only if the player has no decision at all;
  otherwise it is cleared. This is a coarse filter — the final word is `DecisionSystem`'s.

The system does not clear the flag in `Play`: clearing is the consumer's (`DecisionSystem`'s) responsibility, so a
flag raised by someone else (e.g., on a ball-possession change) is not lost.

### 4.2 `BallPossessionSystem` — who possesses the ball

Determines `ball_state.possessed_by` before the players start thinking.

Basics:
- in `Setup` the system exits immediately;
- a 1 s cooldown between possession changes (`last_possession_change_time`) applies — this is debouncing,
  otherwise the ball would flicker between two adjacent players every tick;
- candidates are players within a 1 m radius of the ball; if the ball is already held, the candidates are **only
  the owner's opponents**: teammates do not take the ball away;
- with several candidates, the winner is chosen probabilistically: score = `tackle_rate × k` via
  `RngManager`, where `k ∈ [0.5, 1.5]` at `temperature = 1.0` and compresses toward 1.0 as it decreases —
  a strong tackler wins almost always, but a weak one keeps a chance; at `temperature = 0.0`
  the score is proportional to `tackle_rate`, and equal scores are broken in favor of the smaller index;
- **a possession change raises `needs_decision` for all players at once**. This is the only mechanism
  that "interrupts" the reaction timer: everyone reacts to a possession change immediately, without waiting for
  their interval.

Separately, `last_possessing_team`: the team that last possessed the ball. It is **not reset**
when the ball is released (pass, kick) — thanks to this, the scripts can distinguish "the ball is flying from one of ours"
from "the ball is neutral". It is reset only by the game manager on transition to `Setup`.

### 4.3 `DecisionSystem` — what was decided

For each player, in order, the following happens.

**First, the arrival check — every tick, unconditionally.** If the current decision is `Run`, its target
resolves to a point and is compared with the player's position horizontally. If closer than 0.5 m, the decision
is immediately replaced with `Stop` (marked "not processed" so `ActionSystem` zeroes the
velocity), and the player's processing ends for this step.

This is a key non-trivial decision: without it the player would run past the target, waiting for the next script
call — up to three seconds at low `reaction_rate`. The check runs regardless of phase and
reaction timer. Additionally, in `Setup` `needs_decision` is cleared here — so a player who reached
their position is pinned in the `Stop` state, which signifies readiness.

Target resolution is extracted into a common function: for `Point`/`GridCell`/`Region` the point or center
is taken, and for `Ball` — the ball's **current** position. That is, the arrival check for the ball always verifies where
the ball is now, not where it was at decision time.

**Second, the phase.** In `Setup` nothing further happens: `needs_decision` is cleared, the script is not
called.

**Third, the decision proper** — only if `needs_decision` is raised. It calls
`DecisionMaker::make_decision(game, player_index)`, returning the pair "decision + optional
reason".

`DecisionMaker` is a core extension point, a public trait; the implementation is injected via
`DecisionSystem::with_decision_maker`. The core has a stub (`PlaceholderDecisionMaker`, runs to a
random cell) and a scripted implementation on top of `ynwa-decisions`.

The resulting decision arrives **in the player's team perspective** and is converted to the canonical
orientation (Team A's perspective), in which all game state is stored. For Team A this is
a no-op; for B the point, cell, and region are mirrored, and for `Kick` — its own target point. The `Ball` target
is not mirrored: it resolves in world coordinates.

Then the following are set: `current_decision`, `decision_reason`, `decision_processed = false`,
`needs_decision = false`, `last_decision_time = timestamp`.

**Error handling.** A script error (syntax, exception, timeout) is treated as
"an attempt that happened": `needs_decision` is cleared, `last_decision_time` is updated, and the error text
is written to `last_error`. Otherwise a crashing script would be called every tick — a storm. The reaction to an error
is configured by the handler; by default — a message to stderr and no decision.

### 4.4 What Decisions Are

| Decision | Meaning |
|---|---|
| `Stop` | Stop in place. |
| `Run(Point)` | Run to the point with coordinates. |
| `Run(GridCell)` | Run to the center of the grid cell. |
| `Run(Region)` | Run to the center of the rectangular region. |
| `Run(Ball)` | Run after the ball. |
| `Kick(Point)` | Kick the ball toward the point. |

A decision lives until it is replaced: by a new script decision, an automatic `Stop` on
arrival, or a reset by the game manager. The `decision_processed` flag separates "decision
made" from "decision already turned into an action" — thanks to it the decision executes exactly once,
rather than re-initializing velocity every tick.

Separately there is `reason` — an optional explanatory string the script attaches to the decision.
The core only stores it and exposes it (UI, logs). It is the main tool for understanding why a player
behaves in a certain way.

---

## 5. Step 5: `ActionSystem` — the decision becomes an action

The system iterates over players and processes only decisions with `decision_processed == false`, after
which it marks them processed.

- **`Stop`** → velocity is zeroed.
- **`Run(target)`** → the target resolves to a point, a vector toward it is built, normalized, and multiplied
  by the player's speed: `speed_rate / 100 × 10 m/s` (max ≈ 36 km/h). If the target is closer than 0.5 m,
  velocity is zero — a guard against jitter in place.
- **`Kick(target)`** → executes **only if the player possesses the ball**; otherwise the decision is silently
  ignored (no action and no error).

The key asymmetry important for understanding the movement model: the direction is set **once**, at
decision processing time, and the player then runs in a straight line. For the `Ball` target this means the player
runs to where the ball was at decision time and corrects course only on the next script call.
This is a deliberate model — angular correction with a delay, not perfect pursuit. At the same time
the *arrival* check (§4.3) uses the ball's current position — that is, the player "catches up" with the target
in bursts, but stops precisely.

The kick model (base formulas — in `physics_util`, randomness — from `RngManager`):
- power: `shot_power / 5` m/s with a ±25% spread, scaled by temperature;
- accuracy: symmetric direction deviation from ±5° at `shot_accuracy = 100` to ±45° at 10;
- after the kick, possession is **released**, and `last_possession_change_time` is moved to the current moment.
  The second is mandatory: otherwise `BallPossessionSystem` on the very next tick would give the ball back to the kicker,
  who is standing right next to it. The possession cooldown is used here as "the ball just flew away, don't
  touch it for a second".

---

## 6. Step 6: `PhysicsSystem` — the action changes the world

Physics is deliberately minimal: custom kinematics without an external engine.

- dt is computed as `timestamp - last_update` (the system keeps its own `last_update`). A non-positive
  dt — exit with no changes.
- Players: `position += velocity × dt`. No inertia, acceleration, collisions, or field boundaries —
  a player can run out of the field; the core does not forbid it.
- Ball: if someone possesses it, the ball's position is equated to the owner's position, and the velocity
  is zeroed — that is, "dribbling" is not physics but attachment. If the ball is free, it rolls with
  constant deceleration 2 m/s² along the current velocity vector, after which displacement is applied.

Essential for understanding the core: there is no gravity or bounces, although the coordinates are three-dimensional. The Y axis
is present in the model and in decisions, but is not managed by anything — lofted passes are not implemented.
The ball leaving the field boundaries is not handled by physics: it is an event, handled by the game layer on
the next step.

---

## 7. What Makes the Step Deterministic (and What Does Not)

Determinism is ensured by the fact that the whole step is a pure function of the state and `delta_time`:
there are no real-time internal timers, the system order is fixed, and player iteration order is by
index.

Sources of non-determinism, all explicit:
- winner selection in the ball contest (`BallPossessionSystem`);
- kick power and accuracy spread (`ActionSystem`);
- the `PlaceholderDecisionMaker` stub.

All three go through the single `RngManager`, injected into `Game` by the constructor. At `temperature = 0.0`
randomness degenerates: `next()` returns `0.5`, `randomize` returns the base value,
`randomize_range` returns `0`, so the step becomes fully reproducible. Tests use exactly
this configuration (at zero temperature the stub always chooses the same cell), while the game —
a non-zero temperature (currently `0.7`, set in the football layer).

Separately: the player's Lua VM does not preserve state between calls (the script is reloaded every time),
so the decision depends only on the passed context — the player has no accumulated "memory".

---

## 8. Summary of One Step

```
step(dt):
  timestamp = elapsed_time + dt

  [game manager]      phase? event? → possible stage change,
                      decision reset, ball repositioning, counters

  [reaction]          Play : reaction_rate interval passed → needs_decision = true
                      Setup: no decision → needs_decision = true, otherwise false

  [ball possession]   Setup → skip
                      1s cooldown; candidates within 1m radius (owner's opponents only);
                      winner by tackle_rate × rnd (RngManager); change → needs_decision for all

  [decisions]         for each player:
                        reached target (< 0.5 m)? → Stop, stop thinking
                        Setup? → do not call script
                        needs_decision? → DecisionMaker → mirror to canonical
                                          orientation → current_decision,
                                          decision_processed = false

  [actions]           unprocessed decisions → player velocity,
                      or a kick (owner only) + possession release

  [physics]           positions += velocity × dt; possessed ball sticks,
                      free ball rolls with friction

  elapsed_time = timestamp
```
