# Effect System Implementation Plan

**Date**: 2026-01-08
**Phase**: 2.1 - Effect System (3 weeks)
**Status**: 📋 Planning
**Reference**: POPL 2025 Distinguished Paper on Effect Handlers

---

## 🎯 Objectives

Implement ZULON's **Algebraic Effect System** based on POPL 2025 research:

1. **Effect Definition** - Declare effect operations
2. **Effect Functions** - Functions with effect annotations
3. **Effect Handlers** - Runtime effect resolution
4. **Effect Checking** - Compile-time effect validation
5. **Effect Lowering** - Transform to IR with explicit effect operations

---

## 📋 Design Overview

### Syntax Examples

```go
// 1. Define Effect
effect IO {
    fn read_line() -> str
    fn print_line(line: str)
}

effect State<T> {
    fn get() -> T
    fn set(value: T)
}

// 2. Effect Function (uses ! syntax)
fn greet_user() -> str ! IO {
    print_line("Enter your name:")
    let name = read_line()
    return format!("Hello, {}!", name)
}

// 3. Effect Handler (with block)
fn handle_io() {
    let result = try {
        greet_user()
    } with IO {
        fn read_line() -> str {
            return std::io::stdin().read_line()
        }

        fn print_line(line: str) {
            println!("{}", line)
        }
    }

    println!("Result: {}", result)
}

// 4. Mock for Testing
fn test_greet() {
    let result = try {
        greet_user()
    } with IO {
        fn read_line() -> str {
            return "Alice"
        }

        fn print_line(line: str) {
            // Ignore
        }
    }

    assert_eq!(result, "Hello, Alice!")
}
```

---

## 🏗️ Architecture

### Compiler Pipeline

```
Parser → AST → HIR → Type Check → MIR → LIR → LLVM IR
                ↓                     ↓
           Effect Annotations    Effect Operations
                ↓                     ↓
           Effect Check         Effect Lowering
```

### Components

#### 1. Parser (Lexer → AST)
- **Token**: `effect`, `!`, `with`, `try`
- **AST Nodes**:
  - `EffectDeclaration`
  - `EffectFunction` (function with `!` annotation)
  - `EffectHandler` (try-with block)

#### 2. HIR (High-Level IR)
- **Effect Annotations**:
  - `HirFunction.effects: Vec<Effect>`
  - `HirType::Effect { name, operations }`
  - Effect operations in function body

#### 3. Type Checker
- **Effect Checking**:
  - Verify effect operations are declared
  - Check effect handlers provide all operations
  - Infer effects (optional, like type inference)
  - Effect set operations (union, subset)

#### 4. MIR (Mid-Level IR)
- **Effect Operations**:
  - `MirInstruction::PerformOp { effect, operation, args }`
  - `MirInstruction::HandlerEnter { effect }`
  - `MirInstruction::HandlerExit`
- **Control Flow**: Explicit effect operation calls

#### 5. LIR → LLVM IR
- **Code Generation**:
  - Effect operations become function calls
  - Handlers become callback registration
  - State management via handler context

---

## 📅 Implementation Phases (3 weeks)

### Week 1: Foundation (Days 1-7)

#### Phase 1.1: Syntax & Parser (2 days)
**Goal**: Parse effect syntax into AST

**Tasks**:
1. Add tokens: `effect`, `!`, `with`, `try`
2. AST nodes:
   ```rust
   enum AstStmt {
       EffectDecl {
           name: String,
           operations: Vec<EffectOp>,
       },
       EffectHandler {
           effect_name: String,
           handlers: Vec<Handler>,
           body: Box<AstExpr>,
       },
   }

   struct AstFunction {
       effects: Vec<String>,  // ! IO syntax
       // ... existing fields
   }
   ```
3. Parser rules for:
   - Effect declarations
   - Effect function signatures (`fn foo() -> T ! E`)
   - Try-with blocks

**Success Criteria**:
- ✅ Parse `effect IO { ... }`
- ✅ Parse `fn foo() -> str ! IO`
- ✅ Parse `try { expr } with IO { ... }`
- ✅ Zero parser errors on effect examples

**Estimated Time**: 2 days

---

#### Phase 1.2: HIR Integration (1 day)
**Goal**: Add effect annotations to HIR

**Tasks**:
1. Extend `HirFunction`:
   ```rust
   struct HirFunction {
       name: String,
       params: Vec<HirParam>,
       return_type: HirTy,
       effects: Vec<HirEffect>,  // NEW
       body: HirBlock,
   }

   struct HirEffect {
       name: String,
       operations: Vec<HirEffectOp>,
   }
   ```

2. AST → HIR lowering for effects

**Success Criteria**:
- ✅ Effect declarations lower to HIR
- ✅ Effect functions track effects
- ✅ Try-with blocks preserve handler info

**Estimated Time**: 1 day

---

#### Phase 1.3: Type System (2 days)
**Goal**: Type checking for effects

**Tasks**:
1. Extend type checker:
   ```rust
   fn check_effect_decl(&mut self, decl: &HirEffect) -> Result<()>;
   fn check_effect_fn(&mut self, fn_decl: &HirFunction) -> Result<()>;
   fn check_effect_handler(&mut self, handler: &HirEffectHandler) -> Result<()>;
   fn infer_effects(&mut self, expr: &HirExpr) -> Result<HashSet<HirEffect>>;
   ```

2. Effect rules:
   - Effect operations must be declared
   - Handlers must provide all operations
   - Effect sets propagate through function calls
   - Subeffecting: if `fn foo() ! A + B`, calls `fn bar() ! A`, valid

**Success Criteria**:
- ✅ Detect undeclared effect operations
- ✅ Verify handler completeness
- ✅ Infer effects in expressions
- ✅ Valid type errors for effect violations

**Estimated Time**: 2 days

---

#### Phase 1.4: Testing & Documentation (2 days)
**Goal**: Validate foundation

**Tasks**:
1. Test cases:
   - Effect declaration parsing
   - Effect function type checking
   - Handler validation
   - Effect inference

2. Documentation:
   - Effect system design doc
   - User guide for effects
   - Implementation notes

**Success Criteria**:
- ✅ 20+ unit tests passing
- ✅ Comprehensive documentation

**Estimated Time**: 2 days

---

### Week 2: MIR & Lowering (Days 8-14)

#### Phase 2.1: MIR Effect Operations (3 days)
**Goal**: Lower HIR effects to MIR operations

**Tasks**:
1. Add MIR instructions:
   ```rust
   enum MirInstruction {
       PerformOp {
           effect: String,
           operation: String,
           args: Vec<MirPlace>,
           dest: MirPlace,
       },
       HandlerEnter {
           effect: String,
           handlers: Vec<Handler>,
       },
       HandlerExit,
       // ... existing
   }
   ```

2. HIR → MIR lowering:
   - Effect function calls → `PerformOp`
   - Try-with blocks → handler setup

**Success Criteria**:
- ✅ Effect operations become explicit MIR
- ✅ Handlers generate enter/exit blocks
- ✅ No information loss

**Estimated Time**: 3 days

---

#### Phase 2.2: LIR Integration (2 days)
**Goal**: Lower MIR effects to LIR

**Tasks**:
1. LIR instructions for effects:
   ```rust
   enum LirInstruction {
       EffectCall {
           handler_func: String,
           args: Vec<LirOperand>,
           dest: VReg,
       },
       // ... existing
   }
   ```

2. MIR → LIR lowering for effects

**Success Criteria**:
- ✅ Effect operations become LIR calls
- ✅ Handlers compile correctly

**Estimated Time**: 2 days

---

#### Phase 2.3: LLVM Code Generation (2 days)
**Goal**: Generate LLVM IR for effects

**Tasks**:
1. Effect operation codegen:
   - Convert to function calls
   - Handler context management
   - Callback registration

2. Handler codegen:
   - Handler function generation
   - Dynamic dispatch (or monomorphization)

**Success Criteria**:
- ✅ Effects compile to valid LLVM IR
- ✅ Handlers generate executable code
- ✅ Zero LLVM errors

**Estimated Time**: 2 days

---

#### Phase 2.4: Integration Testing (1 day)
**Goal**: End-to-end testing

**Tasks**:
1. Test effect examples compile
2. Verify generated LLVM IR
3. Basic runtime testing (if possible)

**Success Criteria**:
- ✅ Effect programs compile
- ✅ LLVM IR looks correct
- ✅ No regressions

**Estimated Time**: 1 day

---

### Week 3: Advanced Features & Polish (Days 15-21)

#### Phase 3.1: Effect Inference (2 days)
**Goal**: Automatic effect detection

**Tasks**:
1. Implement effect inference algorithm
2. Type-based effect propagation
3. Optional effect annotations (like types)

**Success Criteria**:
- ✅ Infer effects in simple functions
- ✅ Propagate effects through calls
- ✅ Type checking with inference

**Estimated Time**: 2 days

---

#### Phase 3.2: Standard Library Effects (2 days)
**Goal**: Built-in effects

**Tasks**:
1. Implement `IO` effect in std
2. Implement `State<T>` effect
3. Implement `Error<E>` effect (connects to error handling)

**Success Criteria**:
- ✅ `std::effect::io` module
- ✅ `std::effect::state` module
- ✅ Documentation for standard effects

**Estimated Time**: 2 days

---

#### Phase 3.3: Performance Optimization (2 days)
**Goal**: Zero-cost abstractions

**Tasks**:
1. Handler inlining (static handlers)
2. Effect operation specialization
3. Eliminate runtime overhead where possible

**Success Criteria**:
- ✅ Benchmarks show minimal overhead
- ✅ Static handlers inline
- ✅ No runtime cost for inferred effects

**Estimated Time**: 2 days

---

#### Phase 3.4: Testing & Documentation (2 days)
**Goal**: Production-ready

**Tasks**:
1. Comprehensive test suite:
   - 50+ unit tests
   - Integration tests
   - Performance benchmarks

2. Documentation:
   - User guide
   - Effect system tutorial
   - Best practices
   - Migration guide from error handling

**Success Criteria**:
- ✅ All tests passing
- ✅ Complete documentation
- ✅ Example programs

**Estimated Time**: 2 days

---

## 🎯 Success Criteria

### Must Have (P0)
- ✅ Parse effect syntax
- ✅ Type check effects
- ✅ Lower effects to MIR/LIR
- ✅ Generate LLVM IR
- ✅ Zero compiler warnings/errors

### Should Have (P1)
- ✅ Effect inference
- ✅ Standard library effects (IO, State)
- ✅ Comprehensive testing
- ✅ Good documentation

### Nice to Have (P2)
- ✅ Performance optimization
- ✅ Handler inlining
- ✅ Advanced effect features

---

## 📊 Progress Tracking

| Phase | Status | Progress |
|-------|--------|----------|
| 1.1 Syntax & Parser | ⏳ Pending | 0% |
| 1.2 HIR Integration | ⏳ Pending | 0% |
| 1.3 Type System | ⏳ Pending | 0% |
| 1.4 Testing & Docs | ⏳ Pending | 0% |
| 2.1 MIR Operations | ⏳ Pending | 0% |
| 2.2 LIR Integration | ⏳ Pending | 0% |
| 2.3 LLVM Codegen | ⏳ Pending | 0% |
| 2.4 Integration Tests | ⏳ Pending | 0% |
| 3.1 Effect Inference | ⏳ Pending | 0% |
| 3.2 Stdlib Effects | ⏳ Pending | 0% |
| 3.3 Optimization | ⏳ Pending | 0% |
| 3.4 Testing & Docs | ⏳ Pending | 0% |

**Overall**: 0% complete

---

## 💡 Key Insights

`★ Insight ─────────────────────────────────────`

**1. Effect Handlers vs Monads**:
Unlike Haskell's monads, effect handlers allow:
- Separate effect definition from handling
- Multiple handlers for same effect
- Dynamic handler composition
- Zero-cost abstraction (compiles to state machine)

**2. Gradual Adoption**:
Effects can be adopted incrementally:
- Start: No effects (pure functions)
- Add: Explicit effect annotations
- Advance: Effect inference
- Master: Custom effects and handlers

**3. Testing Superpower**:
Effects make testing trivial:
- Mock IO, database, network by providing handlers
- No dependency injection needed
- Pure functions + mock handlers = easy tests

**4. Performance**:
POPL 2025 shows effect handlers can be:
- Faster than monads (no heap allocation)
- Same performance as handwritten code (with inlining)
- Zero runtime cost for static handlers

`─────────────────────────────────────────────────`

---

## 🚀 Next Steps

### Immediate (Today)
1. Start Phase 1.1: Add effect tokens to lexer
2. Implement AST nodes for effects
3. Add parser rules for effect syntax

### This Week
1. Complete Phase 1: Parser + HIR + Type Checker
2. Write initial tests
3. Document effect system design

### Next 2 Weeks
1. Complete MIR/LIR/LLVM lowering
2. Implement standard effects
3. Performance optimization

---

**Document Version**: 1.0
**Date**: 2026-01-08
**Status**: 📋 Ready to start implementation
**Next**: Phase 1.1 - Syntax & Parser
