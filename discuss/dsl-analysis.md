# DSL Analysis: Simplified Language for Player Scripts

## Context

**Current state:**
- Players are programmed in Lua
- Three-level preamble system: Core → Stdlib → Team → User Script
- Contract: `context` (JSON input) → `make_decision()` → decision table (JSON output)
- Target audience: advanced users comfortable with scripting

**Goal:**
- Allow non-programmers to create player behavior
- Maintain Lua compatibility (power users can still use full Lua)
- DSL should have fewer capabilities but be much simpler
- Reuse existing infrastructure (JSON contract, preambles)

## Approach Comparison

### Option 1: Custom DSL Interpreter (in Rust)

**Implementation:**
- Write parser/lexer in Rust (using nom, pest, or lalrpop)
- AST → Direct execution in Rust
- No Lua involved for DSL scripts

**Example DSL Syntax:**
```
WHEN closest_to_ball:
    RUN TO ball
    
WHEN in_zone("defense"):
    RUN TO my_zone
    
OTHERWISE:
    STOP
```

**Pros:**
- ✅ Full control over language design
- ✅ Can make it extremely simple
- ✅ Best performance (no Lua overhead)
- ✅ Type safety, better error messages
- ✅ Can add game-specific validations at parse time

**Cons:**
- ❌ Significant development effort (2-3 weeks)
- ❌ Need to maintain parser, type checker, runtime
- ❌ Harder to extend (changes require Rust compilation)
- ❌ Two completely separate execution paths (Lua vs DSL)
- ❌ Debugging tools need to be built from scratch

**Architecture:**
```
DSL Script → Rust Parser → AST → Rust Interpreter → Decision
Lua Script → mlua → Lua VM → Decision
```

**Incompatibility risk:** Medium-High
- Different semantics between DSL and Lua
- Hard to mix DSL and Lua in one script

---

### Option 2: DSL Transpiler to Lua (Recommended ⭐)

**Implementation:**
- Write transpiler in Rust (or even in Lua!)
- DSL → Lua code generation → Use existing Lua pipeline
- Single execution path

**Example DSL Syntax:**
```
WHEN closest_to_ball:
    RUN TO ball
    
WHEN in_zone("defense"):
    RUN TO my_zone
    
OTHERWISE:
    STOP
```

**Transpiles to:**
```lua
function make_decision()
    if am_i_closest_to_ball() then
        local ball_pos = ball_position()
        return run_to_point(ball_pos.x, ball_pos.z)
    end
    
    if in_zone("defense") then
        local zone = my_zone()
        return run_to_region(zone.from, zone.to)
    end
    
    return stop()
end
```

**Pros:**
- ✅ Reuses entire Lua infrastructure (VM, sandbox, timeout)
- ✅ Preambles work automatically (DSL uses preamble functions)
- ✅ Single execution path = simpler architecture
- ✅ Can mix DSL and Lua (advanced users can edit transpiled code)
- ✅ Debugging: show both DSL source and generated Lua
- ✅ Moderate development effort (1-2 weeks)
- ✅ Easy to extend (add new DSL constructs → new Lua templates)

**Cons:**
- ❌ DSL limited by Lua capabilities (actually not a problem)
- ❌ Generated Lua might be verbose
- ❌ Transpilation step adds complexity to workflow

**Architecture:**
```
DSL Script → Rust/Lua Transpiler → Lua Code → mlua → Decision
Lua Script → mlua → Decision
```

**Compatibility:** Perfect
- DSL is just syntactic sugar over Lua
- Users can start with DSL, then graduate to Lua
- Can mix: generate Lua from DSL, then hand-edit

**Implementation sketch:**
```rust
struct DslTranspiler {
    preamble_functions: HashSet<String>, // from core.lua, stdlib.lua
}

impl DslTranspiler {
    fn transpile(&self, dsl_source: &str) -> Result<String, TranspileError> {
        let ast = self.parse(dsl_source)?;
        let lua_code = self.generate_lua(&ast)?;
        Ok(lua_code)
    }
}

// Usage in ScriptedDecisionMaker:
let lua_code = if script.ends_with(".dsl") {
    transpiler.transpile(&script)?
} else {
    script // already Lua
};
```

---

### Option 3: Use Existing DSL with Lua Target

**Candidates:**
- **MoonScript** - CoffeeScript-like language that compiles to Lua
- **Fennel** - Lisp dialect that compiles to Lua
- **Teal** - Typed Lua (transpiles to Lua)
- **Haxe** - Multi-target language (can output Lua)

**Analysis:**

#### MoonScript
```moonscript
make_decision = ->
  if am_i_closest_to_ball!
    ball_pos = ball_position!
    run_to_point ball_pos.x, ball_pos.z
  else
    stop!
```

**Pros:**
- ✅ Cleaner syntax than Lua
- ✅ Mature, stable
- ✅ Good Lua interop

**Cons:**
- ❌ Still requires programming knowledge (classes, OOP)
- ❌ Not simpler for non-programmers
- ❌ Need to bundle MoonScript compiler

#### Fennel
```fennel
(fn make-decision []
  (if (am-i-closest-to-ball)
      (let [ball-pos (ball-position)]
        (run-to-point ball-pos.x ball-pos.z))
      (stop)))
```

**Pros:**
- ✅ Very powerful (Lisp macros)
- ✅ Excellent Lua interop

**Cons:**
- ❌ Lisp syntax scary for non-programmers
- ❌ Overkill for simple conditions

#### Teal
```teal
function make_decision(): Decision
    if am_i_closest_to_ball() then
        local ball_pos = ball_position()
        return run_to_point(ball_pos.x, ball_pos.z)
    end
    return stop()
end
```

**Pros:**
- ✅ Type safety
- ✅ Almost identical to Lua

**Cons:**
- ❌ Not simpler - just typed Lua
- ❌ Not for non-programmers

**Verdict on existing languages:**
❌ **None solve our problem**
- They're for programmers who want better Lua
- We need something for **non-programmers**
- Our use case is very specific (condition → action)

---

### Option 4: Hybrid Approach (Recommended+ ⭐⭐)

**Combine Options 2 and 3:**
1. **Simple DSL** for beginners (transpiles to Lua)
2. **Full Lua** for advanced users
3. **Visual editor** that generates DSL (future)

**DSL Design Principles:**
- Declarative, not imperative
- Condition-action pairs (like behavior trees)
- Limited but safe constructs
- Reads like English

**Proposed DSL Syntax:**
```
# Simple condition-action rules
# Evaluated top to bottom, first match wins

RULE "chase ball"
  WHEN I am closest to ball
  THEN run to ball

RULE "defend zone"
  WHEN my role is "defender"
  AND ball is in "opponent half"
  THEN run to my zone

RULE "follow target"
  WHEN time > 30 seconds
  THEN run to cell K7

RULE "default"
  ALWAYS
  THEN stop
```

**Transpiles to:**
```lua
function make_decision()
    -- RULE: chase ball
    if am_i_closest_to_ball() then
        local ball_pos = ball_position()
        return run_to_point(ball_pos.x, ball_pos.z)
    end
    
    -- RULE: defend zone
    if my_role() == "defender" and ball_in_opponent_half() then
        local zone = my_zone()
        return run_to_region(zone.from, zone.to)
    end
    
    -- RULE: follow target
    if game_time() > 30 then
        return run_to_cell("K7")
    end
    
    -- RULE: default
    return stop()
end
```

**DSL Features:**

1. **Conditions:**
   - `I am closest to ball` → `am_i_closest_to_ball()`
   - `my role is "X"` → `my_role() == "X"`
   - `ball is in "zone"` → `ball_in_zone("zone")`
   - `time > N` → `game_time() > N`
   - `AND`, `OR`, `NOT` for combining

2. **Actions:**
   - `run to ball` → `run_to_point(ball_position())`
   - `run to my zone` → `run_to_region(my_zone())`
   - `run to cell X` → `run_to_cell("X")`
   - `run to point (X, Z)` → `run_to_point(X, Z)`
   - `stop` → `stop()`

3. **Structure:**
   - Rules evaluated in order
   - First matching rule wins
   - `ALWAYS` = catch-all

**Implementation Plan:**

**Phase 1: Parser (1-2 days)**
```rust
// Using nom or pest
struct Rule {
    name: String,
    condition: Condition,
    action: Action,
}

enum Condition {
    Simple(SimpleCondition),
    And(Box<Condition>, Box<Condition>),
    Or(Box<Condition>, Box<Condition>),
    Not(Box<Condition>),
    Always,
}

enum SimpleCondition {
    ClosestToBall,
    RoleIs(String),
    BallInZone(String),
    TimeGreater(f32),
    // ... extensible
}

enum Action {
    RunToBall,
    RunToZone,
    RunToCell(String),
    RunToPoint(f32, f32),
    Stop,
}
```

**Phase 2: Code Generator (1-2 days)**
```rust
impl Transpiler {
    fn generate_lua(&self, rules: &[Rule]) -> String {
        let mut lua = String::from("function make_decision()\n");
        
        for rule in rules {
            lua.push_str(&format!("    -- RULE: {}\n", rule.name));
            lua.push_str("    if ");
            lua.push_str(&self.generate_condition(&rule.condition));
            lua.push_str(" then\n");
            lua.push_str(&self.generate_action(&rule.action));
            lua.push_str("    end\n\n");
        }
        
        lua.push_str("    return stop() -- fallback\n");
        lua.push_str("end\n");
        lua
    }
    
    fn generate_condition(&self, cond: &Condition) -> String {
        match cond {
            Condition::Simple(SimpleCondition::ClosestToBall) => 
                "am_i_closest_to_ball()".to_string(),
            Condition::Simple(SimpleCondition::RoleIs(role)) => 
                format!("my_role() == \"{}\"", role),
            Condition::And(a, b) => 
                format!("({} and {})", 
                    self.generate_condition(a), 
                    self.generate_condition(b)),
            Condition::Always => "true".to_string(),
            // ... etc
        }
    }
    
    fn generate_action(&self, action: &Action) -> String {
        match action {
            Action::RunToBall => 
                "        local pos = ball_position()\n\
                         return run_to_point(pos.x, pos.z)\n".to_string(),
            Action::Stop => 
                "        return stop()\n".to_string(),
            // ... etc
        }
    }
}
```

**Phase 3: Integration (1 day)**
```rust
// In ScriptedDecisionMaker or new DSLDecisionMaker

impl ScriptedDecisionMaker {
    pub fn new(game: &Game) -> Result<Self, DecisionError> {
        let mut engines = HashMap::new();
        
        for (idx, player) in game.config().players.iter().enumerate() {
            let lua_code = if player.script.starts_with("RULE") {
                // It's DSL - transpile it
                let transpiler = DslTranspiler::new();
                transpiler.transpile(&player.script)?
            } else {
                // It's already Lua
                player.script.clone()
            };
            
            // Rest is same - execute Lua
            let config = Self::build_config(game)?;
            let engine = DecisionEngine::new(&config)?;
            engines.insert(idx, engine);
        }
        
        Ok(Self { engines })
    }
}
```

---

## Detailed Comparison Matrix

| Criterion | Custom Interpreter | DSL→Lua Transpiler | Existing Lang | Hybrid |
|-----------|-------------------|-------------------|---------------|---------|
| **Development Time** | 2-3 weeks | 1-2 weeks | 1 week | 1-2 weeks |
| **Simplicity for Users** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Lua Compatibility** | ❌ | ✅ | Partial | ✅ |
| **Code Reuse** | ❌ | ✅ | ✅ | ✅ |
| **Extensibility** | ⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Debugging** | Hard | Easy | Medium | Easy |
| **Performance** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **Maintenance** | High | Low | Medium | Low |
| **Error Messages** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Mix DSL+Lua** | ❌ | ✅ | Partial | ✅ |

---

## Recommendation: Hybrid Approach ⭐⭐

**Why:**
1. **Reuses existing infrastructure** - no need to reinvent VM, sandbox, timeout
2. **Perfect compatibility** - DSL is just syntactic sugar
3. **Smooth learning curve** - start with DSL, graduate to Lua
4. **Minimal risk** - if DSL fails, users can still use Lua
5. **Future-proof** - can add visual editor that generates DSL

**What to build:**

### Phase 1: Minimal Viable DSL (MVP)
**Time: 1 week**

**Features:**
- Simple RULE syntax
- 5-6 basic conditions (closest to ball, role, time)
- 3-4 basic actions (run to ball/zone/cell, stop)
- Transpile to Lua
- Good error messages

**Test with:**
- Real users (non-programmers)
- Collect feedback
- Iterate on syntax

### Phase 2: Enhanced DSL
**Time: 1 week**

**Features:**
- More conditions (distance checks, teammates count, etc.)
- More actions (run to teammate, run to opponent)
- Variables and simple expressions: `distance to ball < 5`
- Comments and documentation

### Phase 3: Developer Experience
**Time: 1 week**

**Features:**
- Syntax highlighting (VS Code extension)
- DSL → Lua preview (show generated code)
- Better error messages with line numbers
- DSL debugger (step through rules)

### Phase 4: Visual Editor (Future)
**Time: 2-3 weeks**

**Features:**
- Drag-and-drop rule builder
- Visual condition composer
- Live preview with game simulation
- Export to DSL or Lua

---

## Alternative: Domain-Specific Language Ideas

If we go fully custom, here are some interesting syntax options:

### Option A: Behavior Tree Syntax
```
BEHAVIOR "player_ai"
  SEQUENCE
    CONDITION closest_to_ball
    ACTION run_to ball
  FALLBACK
    CONDITION in_defensive_zone
    ACTION run_to my_zone
  DEFAULT
    ACTION stop
```

**Pros:** Familiar to game developers
**Cons:** More complex structure

### Option B: State Machine Syntax
```
STATE chasing_ball
  ENTER: run to ball
  WHILE: I am closest to ball
  EXIT: go to defending

STATE defending
  ENTER: run to my zone
  WHILE: ball in opponent half
  EXIT: go to chasing_ball
```

**Pros:** Clear state modeling
**Cons:** Requires state management (complexity)

### Option C: Natural Language (Very Simple)
```
If I am closest to the ball, run to the ball.
If my role is defender and ball is in opponent half, run to my zone.
Otherwise, stop.
```

**Pros:** Easiest for non-programmers
**Cons:** Hard to parse reliably, ambiguous

---

## Implementation Checklist

### For DSL→Lua Transpiler:

- [ ] Define DSL grammar (EBNF)
- [ ] Choose parser library (pest recommended)
- [ ] Implement parser
- [ ] Implement code generator
- [ ] Write unit tests (DSL → Lua → parse valid)
- [ ] Integrate into ScriptedDecisionMaker
- [ ] Add error handling with line numbers
- [ ] Write documentation
- [ ] Create example scripts in DSL
- [ ] Add integration tests

### For Preamble Functions:

DSL relies on preambles, so need to implement:

**Core preamble** (`core.lua`):
- [ ] `my_position()` → context.me.position
- [ ] `ball_position()` → context.ball.position
- [ ] `game_time()` → context.game.elapsed_time
- [ ] `stop()` → {action = "stop"}
- [ ] `run_to_point(x, z)` → decision table
- [ ] `run_to_cell(cell)` → decision table

**Stdlib preamble** (`stdlib.lua`):
- [ ] `distance(pos1, pos2)` → Euclidean distance
- [ ] `am_i_closest_to_ball()` → boolean
- [ ] `nearest_teammate()` → teammate info
- [ ] `nearest_opponent()` → opponent info

**Team preamble** (example):
- [ ] `my_role()` → "goalkeeper"|"defender"|etc
- [ ] `my_zone()` → region definition
- [ ] `ball_in_opponent_half()` → boolean

---

## Risks and Mitigations

### Risk 1: DSL Too Limited
**Mitigation:** Users can always fall back to Lua. DSL is optional.

### Risk 2: Transpiler Bugs
**Mitigation:** 
- Extensive tests: DSL → Lua → validate Lua syntax
- Show generated Lua to users for verification

### Risk 3: Poor Error Messages
**Mitigation:**
- Track source locations during parsing
- Map Lua errors back to DSL line numbers
- Provide helpful suggestions

### Risk 4: User Confusion (Two Languages)
**Mitigation:**
- Clear documentation: "DSL is simplified Lua"
- Show transpiled Lua as learning tool
- Gradual migration path

---

## Conclusion

**Recommended approach: DSL→Lua Transpiler (Hybrid)**

**Key benefits:**
✅ Reuses all existing infrastructure  
✅ Perfect Lua compatibility  
✅ Simple for non-programmers  
✅ Flexible for power users  
✅ Low development and maintenance cost  
✅ Future-proof (can add visual editor)  

**Next steps:**
1. Design DSL syntax (get user feedback!)
2. Implement parser (pest crate)
3. Implement code generator
4. Integrate into decision system
5. Write preamble functions
6. Create documentation and examples

**Estimated timeline:** 2-3 weeks for complete MVP

This approach offers the best balance of simplicity, power, and maintainability.
