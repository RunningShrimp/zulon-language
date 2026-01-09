# Ralph Loop Iteration 36+ - Strategic Plan

**Date**: 2026-01-09
**Based on**: IMPLEMENTATION_PLAN.md and TODOLIST.md analysis

---

## Current Status Assessment

### ✅ Completed (from TODOLIST.md)
- Type system基础 (Week 1)
- Type inference (Week 2-3)
- Collections (Vec, HashMap, HashSet, VecDeque)
- YAN tool基础 (build, run, new, clean)
- Error handling增强 (90% - throw, ?, | syntax)
- Testing framework基础 (test discovery, assertions)

### 🔧 Recently Fixed (Iteration 35)
- Compilation blocking errors resolved
- Effect system stubs added
- All core crates build successfully

### 📊 Progress Estimates
```
Phase 1 (MVP): ~40% complete
  ├─ Lexer/Parser: ~80% (needs testing/error recovery)
  ├─ Type System: ~70% (inference done, checking partial)
  ├─ IR Pipeline: ~50% (HIR/MIR/LIR partially done)
  ├─ Code Gen: ~40% (LLVM basic support)
  ├─ Runtime: ~30% (basic IO, no ARC yet)
  └─ Std Library: ~60% (core types, needs API completion)
```

---

## Next Iterations Priority Queue

### Iteration 36: Parser Error Recovery (P0 - Blocking)
**Goal**: Enable robust parsing with helpful error messages

**Tasks**:
1. Implement error recovery in parser
2. Add better syntax error messages
3. Test with malformed input
4. Document error format

**Impact**: Unlocks better user experience for development

**Estimated Time**: 1-2 iterations

---

### Iteration 37-38: HIR Type Checking (P0 - Blocking)
**Goal**: Complete type checking in HIR layer

**Tasks**:
1. Integrate type inference with HIR
2. Add type checking for all HIR constructs
3. Implement trait constraints checking
4. Add type error reporting

**Impact**: Enables semantic validation before codegen

**Estimated Time**: 2-3 iterations

---

### Iteration 39-40: MIR Borrow Checking (P0 - Blocking)
**Goal**: Implement basic Tree Borrows model

**Tasks**:
1. Design simplified borrow checker
2. Implement lifetime analysis (basic)
3. Add borrow checking rules
4. Test with ownership examples

**Impact**: Memory safety foundation

**Estimated Time**: 2-3 iterations

---

### Iteration 41-42: Test Runner (P1 - Important)
**Goal**: Complete testing framework

**Tasks**:
1. Implement `#[test]` attribute parsing
2. Build test discovery system
3. Create test execution engine
4. Add test reporting

**Impact**: Enables TDD and regression testing

**Estimated Time**: 2 iterations

---

### Iteration 43-44: Code Generation Completion (P0 - Blocking)
**Goal**: Full LLVM IR generation

**Tasks**:
1. Complete all construct codegen (structs, enums, closures)
2. Implement proper calling convention
3. Add error handling codegen (Outcome type)
4. Test generated code

**Impact**: Enables running real programs

**Estimated Time**: 2-3 iterations

---

### Iteration 45-46: Runtime ARC (P0 - Blocking)
**Goal**: Basic reference counting

**Tasks**:
1. Implement Arc<T> smart pointer
2. Add reference counting operations
3. Integrate with codegen
4. Test memory management

**Impact**: Memory safety without GC

**Estimated Time**: 2-3 iterations

---

### Iteration 47-48: Standard Library API Completion (P1)
**Goal**: Fill gaps in std library

**Tasks**:
1. Complete Vec API
2. Complete HashMap API
3. Add String operations
4. Documentation and examples

**Impact**: Usable library for developers

**Estimated Time**: 2 iterations

---

## Iteration Selection Strategy

### Criteria for P0 (Blocking)
1. Prevents compilation/execution of programs
2. Required for MVP
3. Blocks other work

### Criteria for P1 (Important)
1. Significantly improves developer experience
2. Required for MVP but not blocking
3. Enables testing/validation

### Criteria for P2 (Enhancement)
1. Nice to have
2. Can be deferred to Phase 2
3. Doesn't block MVP

---

## Recommended Next 5 Iterations

```
Iteration 36: Parser Error Recovery        [P0] ─┐
Iteration 37: HIR Type Checking (Part 1)   [P0]  │→ Compiler Frontend
Iteration 38: HIR Type Checking (Part 2)   [P0] ─┘

Iteration 39: Test Runner Implementation    [P1] ─→ Testing Infrastructure

Iteration 40: Code Generation - Control Flow [P0] ─→ Codegen Foundation
```

**Rationale**:
1. **Frontend First**: Complete parsing and type checking before moving to backend
2. **Testing Early**: Test runner enables validation of all subsequent work
3. **Incremental Codegen**: Start with control flow, build up to complex features

---

## Success Metrics

### Iteration 36-38 (Frontend)
- [ ] Parser recovers from 90% of syntax errors
- [ ] Type checker validates all core language features
- [ ] Error messages are clear and actionable

### Iteration 39 (Testing)
- [ ] Can run `yan test` successfully
- [ ] Test discovery works for all functions
- [ ] Test reports show pass/fail clearly

### Iteration 40 (Codegen)
- [ ] All control flow constructs compile
- [ ] Generated LLVM IR is verifiable
- [ ] Simple programs execute correctly

---

## Risk Mitigation

### Risk: Effect System Complexity
**Mitigation**: Keep stubs until Phase 2, focus on MVP features first

### Risk: Borrow Checker Difficulty
**Mitigation**: Start with simplified model, add sophistication in Phase 2

### Risk: Test Scope Creep
**Mitigation**: Implement minimal viable test runner, enhance in Phase 2

---

## Dependencies

```
Parser Error Recovery
  └─> HIR Type Checking
       └─> Test Runner
            └─> Code Generation
                 └─> Runtime ARC
```

**Critical Path**: Parser → Type Checking → Codegen → Runtime

**Parallel Opportunities**:
- Test Runner can be developed alongside type checking
- Standard Library API work can happen in parallel
- Documentation can be written incrementally

---

## Resource Allocation

**Frontend Work (Iterations 36-38)**: 60% effort
- Parser error recovery is complex but localized
- Type checking integration requires careful design
- High impact on user experience

**Testing (Iteration 39)**: 20% effort
- Test runner is straightforward
- Can leverage existing test discovery work
- Enables validation of all other work

**Codegen (Iteration 40)**: 20% effort
- Control flow is foundational
- Simpler than data structure codegen
- Enables early end-to-end testing

---

## Handoff Checklist

At the end of each iteration:
1. [ ] All tests pass
2. [ ] Documentation updated
3. [ ] Examples verified
4. [ ] Git commit with clear message
5. [ ] Summary document created

---

**Next Action**: Start Iteration 36 - Parser Error Recovery
**Focus**: Make the parser robust and user-friendly
**Success Criteria**: Parser handles malformed input gracefully with helpful error messages
