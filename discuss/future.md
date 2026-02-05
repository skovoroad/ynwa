# Future Architecture Considerations

This document outlines architectural analysis for two major features planned for future implementation:
1. Game serialization and replay
2. Server-side parallel game processing

## Current Architecture Analysis

### System Pipeline

The game runs through a deterministic pipeline of systems:
1. **PlayerReactionSystem** → sets `needs_decision = true` based on reaction_rate
2. **BallPossessionSystem** → determines ball ownership (has randomness!)
3. **DecisionSystem** → creates `Decision` via Lua scripts
4. **ActionSystem** → converts `Decision` to `Velocity`
5. **PhysicsSystem** → applies `Velocity` to `Position`

### Current State Structure

**PlayerState:**
- `position: Point3D`
- `velocity: Velocity3D`
- `current_decision: Option<Decision>`
- `needs_decision: bool`
- `decision_processed: bool`
- `last_decision_time: f32`
- `last_error: Option<String>`

**BallState:**
- `position: Point3D`
- `velocity: Velocity3D`
- `possessed_by: Option<usize>`
- `last_possession_change_time: f32`

**Sources of Randomness:**
- BallPossessionSystem: tackle success based on tackle_rate × random_multiplier
- PlaceholderDecisionMaker: random grid cell selection

---

## 1. Serialization and Replay System

### Recording Level Options

#### **Option A: Decision Level (Recommended)**

**Record:** timestamp + player_index + Decision + RNG results/seeds

**Pros:**
- Relatively compact (~100 bytes per decision × ~30 decisions/sec × 22 players ≈ 66 KB/sec)
- Replay: feed pre-recorded decisions to DecisionSystem (bypass Lua execution)
- Determinism: only need to record **RNG results for BallPossessionSystem**
- Easy to implement: `DecisionSystem.with_decision_maker(ReplayDecisionMaker)`

**Cons:**
- Still need to store random event results (ball possession)
- Replay may break if ActionSystem or PhysicsSystem logic changes

**Data structure:**
```rust
struct ReplayFrame {
    timestamp: f32,
    // For each player that needs a decision
    decisions: HashMap<usize, Decision>,
    // Random event results
    possession_changes: Vec<(usize, Option<usize>)>, // (player_idx, new_owner)
}
```

#### **Option B: Action Level (Velocities)**

**Record:** timestamp + player_index + Velocity3D

**Pros:**
- More deterministic: bypasses Decision → Action transformation
- Independent of ActionSystem changes

**Cons:**
- More data (3 floats per velocity × ~22 players × frames)
- Still depends on PhysicsSystem

#### **Option C: State Level (Full State Snapshot)**

**Record:** entire GameState every frame

**Pros:**
- Absolute determinism
- Can seek to any point instantly

**Cons:**
- **Huge data volume**: ~20 KB × 60 FPS = 1.2 MB/sec = 70+ MB/minute
- Unacceptable for long games and server scenarios

### 🎯 **Recommended Approach: Hybrid "Decisions + Checkpoints"**

**Strategy:**
1. **Decisions stream:** Record all decisions and RNG results
2. **Periodic checkpoints:** Full GameState every N seconds (e.g., every 10 seconds)
3. **Seekable replay:** Jump to checkpoint, then replay decisions forward

**Benefits:**
- Efficient storage: ~70 KB/sec (compressible)
- Fast seeking: max 10 seconds to replay from checkpoint
- Future-proof: checkpoints ensure compatibility even if logic changes

### Key Implementation Requirements

#### 1. Deterministic RNG
**Problem:** Current code uses `rand::random()` which is non-deterministic

**Solution:**
```rust
trait GameRng: Send {
    fn next_f32(&mut self) -> f32;
}

// For live games:
struct SeededRng {
    rng: StdRng,  // from rand crate with fixed seed
}

// For replay:
struct ReplayRng {
    values: Vec<f32>,
    index: usize,
}
```

**Changes needed:**
- BallPossessionSystem::with_rng() already exists ✓
- Need to add seed parameter to World creation
- Record seed in replay header

#### 2. Serialization Support
**Add serde derives to all game types:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState { ... }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Decision { ... }
```

#### 3. Replay File Format
```
ReplayFile {
    header: {
        version: u32,
        seed: u64,
        config: GameConfig,
    },
    checkpoints: Vec<Checkpoint {
        timestamp: f32,
        state: GameState,
    }>,
    frames: Vec<ReplayFrame {
        timestamp: f32,
        decisions: HashMap<usize, Decision>,
        rng_results: Vec<f32>,
    }>,
}
```

#### 4. ReplayDecisionMaker
```rust
struct ReplayDecisionMaker {
    replay_data: HashMap<f32, HashMap<usize, Decision>>,
}

impl DecisionMaker for ReplayDecisionMaker {
    fn make_decision(&mut self, game: &Game, player_index: usize) 
        -> Result<Decision, DecisionError> 
    {
        let timestamp = game.state().elapsed_time;
        self.replay_data
            .get(&timestamp)
            .and_then(|frame| frame.get(&player_index))
            .cloned()
            .ok_or(DecisionError::ReplayDataMissing)
    }
}
```

### Storage Estimates

**10-minute game:**
- Checkpoints (10s interval): 60 × 20 KB = 1.2 MB
- Decision frames: 600s × 66 KB/s = 39.6 MB
- Total: ~40 MB (compresses to ~15-20 MB)

---

## 2. Parallel Processing and Server Architecture

### Can We Parallelize?

✅ **YES! Architecture is already well-suited for parallelization**

**Current strengths:**
- `World` owns `Game` — isolated unit of work
- No global state
- Each game is independent
- Systems operate on single game instance

### Required Changes

#### **A. Thread Safety Bounds**

**Current code:**
```rust
Box<dyn System>  // NOT Send!
Box<dyn DecisionMaker>  // NOT Send!
```

**Needed:**
```rust
Box<dyn System + Send>
Box<dyn DecisionMaker + Send>
```

**Challenge:** `ScriptedDecisionMaker` contains `DecisionEngine` with Lua VM, which is **not Sync**

**Solutions:**
1. Each thread creates its own `ScriptedDecisionMaker` instance
2. Use `Send` without `Sync` and run games in separate threads (recommended)
3. For shared read-only data: `Arc<GameConfig>` is Send + Sync

#### **B. RNG Thread Safety**

**Current issue:** `rand::random()` uses thread-local state

**Solutions:**
1. **Per-game RNG** (recommended):
   ```rust
   struct World {
       game: Game,
       systems: Vec<Box<dyn System + Send>>,
       rng: Box<dyn GameRng>,  // owned by each World
   }
   ```

2. **Thread-local RNG:**
   ```rust
   thread_local! {
       static GAME_RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
   }
   ```

#### **C. Server Architecture Options**

##### **Option 1: Simple Parallel (rayon)**
```rust
struct GameServer {
    worlds: Vec<World>,
}

impl GameServer {
    fn step_all(&mut self, delta_time: f32) {
        self.worlds.par_iter_mut()  // rayon parallel iterator
            .for_each(|world| {
                world.step(delta_time);
            });
    }
}
```

**Pros:**
- Simple to implement
- Automatic work stealing
- Good CPU utilization

**Cons:**
- All games must use same delta_time
- No individual game control

##### **Option 2: Actor Model (tokio)**
```rust
struct GameActor {
    world: World,
    commands: mpsc::Receiver<GameCommand>,
    events: mpsc::Sender<GameEvent>,
}

impl GameActor {
    async fn run(mut self) {
        loop {
            select! {
                Some(cmd) = self.commands.recv() => self.handle_command(cmd),
                _ = tokio::time::sleep(Duration::from_millis(16)) => {
                    self.world.step(0.016);
                }
            }
        }
    }
}

struct GameServer {
    games: HashMap<GameId, mpsc::Sender<GameCommand>>,
}
```

**Pros:**
- Individual game control
- Can pause/resume/adjust speed per game
- Clean message passing
- Scales to thousands of games

**Cons:**
- More complex
- Actor overhead

##### **Option 3: Hybrid**
- **rayon** for batch simulation (CPU-bound)
- **tokio** for network I/O and client connections (I/O-bound)

```rust
struct GameServer {
    worlds: Arc<Mutex<Vec<World>>>,
    network: tokio::Runtime,
}

// Simulation thread (CPU-bound)
fn simulation_loop(worlds: Arc<Mutex<Vec<World>>>) {
    loop {
        let mut worlds = worlds.lock().unwrap();
        worlds.par_iter_mut().for_each(|w| w.step(0.016));
        thread::sleep(Duration::from_millis(16));
    }
}

// Network thread (I/O-bound)
async fn handle_client(socket: TcpStream, worlds: Arc<Mutex<Vec<World>>>) {
    // Send game state updates to client
}
```

### 🎯 **Recommended Server Architecture**

**Phase 1: Simple Parallel**
- Use rayon for parallel game processing
- Single-threaded network (or one thread per client)
- Good for 10-1000 games

**Phase 2: Actor Model**
- Migrate to tokio actors
- Scales to 10,000+ games
- Requires more refactoring

### Does This Require Radical Redesign?

**NO!** Current architecture is already well-structured for parallelization.

**Required changes:**
1. Add `+ Send` bounds to trait objects (minor)
2. Remove any `Rc/RefCell` if present (replace with `Arc/Mutex` if needed)
3. Make RNG per-game or thread-local (small change)
4. Add serialization derives (tedious but straightforward)

**No radical redesign needed** ✅

---

## Integration: Serialization + Parallelization

**Good news:** These features are compatible!

### 1. Replay Helps Test Parallelization

**Strategy:**
- Record game in single-threaded mode
- Replay in multi-threaded mode
- Results should be identical (proves determinism)

### 2. Shared RNG Abstraction

```rust
trait GameRng: Send {
    fn next_f32(&mut self) -> f32;
    fn fork(&self) -> Box<dyn GameRng>;  // for parallel branches
}

// For live games (parallel-safe):
struct SeededRng {
    rng: StdRng,
    seed: u64,
}

// For replay:
struct ReplayRng {
    values: Vec<f32>,
    index: usize,
}
```

### 3. Checkpoint Benefits for Server

**Use case:** Server crash recovery
- Periodic checkpoints saved to disk
- On restart: load checkpoint + replay recent decisions
- No need to replay entire game from start

### 4. Network Protocol Efficiency

**Send to clients:**
- Full state on connect (checkpoint)
- Decision diffs thereafter
- Client can reconstruct full state

**Bandwidth:**
- Initial: 20 KB (checkpoint)
- Updates: ~1-2 KB/sec (decisions only)
- Very efficient for spectating!

---

## Implementation Roadmap

### For Serialization

**Step 1: Add serde support**
- [ ] Add serde derives to all game types
- [ ] Test serialization/deserialization
- [ ] Benchmark serialization performance

**Step 2: Deterministic RNG**
- [ ] Create `GameRng` trait
- [ ] Implement `SeededRng` and `ReplayRng`
- [ ] Inject RNG into systems via constructor
- [ ] Remove all uses of `rand::random()`

**Step 3: Replay system**
- [ ] Define replay file format
- [ ] Implement `ReplayRecorder` system
- [ ] Implement `ReplayDecisionMaker`
- [ ] Add checkpoint generation

**Step 4: Replay player**
- [ ] Create replay loading functionality
- [ ] Add seek/pause/resume controls
- [ ] Test determinism (same replay = same result)

### For Parallelization

**Step 1: Thread safety**
- [ ] Add `+ Send` to `System` and `DecisionMaker` traits
- [ ] Audit codebase for `Rc/RefCell` usage
- [ ] Replace with `Arc/Mutex` where necessary
- [ ] Add `#[derive(Clone)]` where needed for `Arc`

**Step 2: Simple parallel server**
- [ ] Create `GameServer` struct
- [ ] Implement rayon-based parallel stepping
- [ ] Add batch game creation
- [ ] Test scaling (10, 100, 1000 games)

**Step 3: Network layer**
- [ ] Define network protocol (protobuf/msgpack)
- [ ] Implement state serialization for network
- [ ] Create client connection handler
- [ ] Stream state updates to clients

**Step 4: Advanced features**
- [ ] Game pause/resume/speed control
- [ ] Dynamic game creation/destruction
- [ ] Load balancing across CPU cores
- [ ] Consider migration to actor model if needed

---

## Performance Estimates

### Single Game
- Current: 60 FPS easily achievable
- CPU usage: <5% per game on modern CPU

### Parallel Server (8-core CPU)
- Conservative: 500-1000 games at 60 FPS
- Optimistic: 2000-5000 games at 30 FPS
- Bottleneck: Lua script execution

### Network Bandwidth
- Per spectator: 1-2 KB/sec (decision stream)
- Per spectator: 20 KB initial (checkpoint)
- 1000 spectators = 2 MB/sec = 16 Mbps (very manageable)

### Storage
- 10-minute game: ~15-20 MB compressed
- 1000 games = 15-20 GB
- Reasonable for server storage

---

## Risks and Mitigations

### Risk 1: Non-Deterministic Behavior
**Mitigation:** Comprehensive replay tests. Any change must pass replay verification.

### Risk 2: Lua Script Performance
**Mitigation:** 
- Timeout already implemented (100ms)
- Consider script caching/compilation
- Profile and optimize hot paths

### Risk 3: Memory Usage at Scale
**Mitigation:**
- Each game ~100 KB memory
- 10,000 games = 1 GB (acceptable)
- Monitor and set limits

### Risk 4: Floating-Point Determinism
**Mitigation:**
- Use fixed-point for critical calculations if needed
- Test across platforms (x86, ARM)
- Document any known platform differences

---

## Conclusion

The current architecture is **well-positioned** for both serialization and parallelization:

✅ Games are isolated (`World` owns everything)  
✅ Systems are already injectable  
✅ No global state  
✅ RNG is mockable (some systems)  

**Required work is incremental, not revolutionary.**

The main challenges are:
1. Making all types `Serialize + Send`
2. Replacing `rand::random()` with deterministic RNG
3. Testing determinism thoroughly

**Estimated effort:**
- Serialization: 1-2 weeks
- Parallelization: 1 week
- Testing and polish: 1 week
- **Total: 3-4 weeks** for solid implementation

This is a **manageable scope** that doesn't require architectural redesign.
