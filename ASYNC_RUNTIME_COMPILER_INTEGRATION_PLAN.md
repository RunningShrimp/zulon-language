# Async Runtime Compiler Integration Plan

**Date**: 2026-01-10
**Iteration**: 18
**Phase**: 2.2.1 - Compiler Integration Strategy
**Status**: 📋 Planning Document

---

## Executive Summary

This document outlines the strategy for integrating the async runtime (built in Iterations 16-17) with the ZULON compiler. The integration will enable ZULON code to use effect-based async operations like `Async::read()` without function coloring.

---

## Current State Analysis

### What We Have ✅

1. **ZULON Parser** (Complete):
   - Recognizes `effect Async { ... }` syntax ✅
   - Recognizes `try...with` blocks ✅
   - Recognizes `Async::operation()` calls ✅

2. **HIR Layer** (Complete):
   - Effect declarations exist ✅
   - Effect handler blocks exist ✅
   - Type checking for effects works ✅

3. **MIR Layer** (Complete):
   - `PerformEffect` instruction exists ✅
   - Effect handler blocks exist ✅
   - But: Effect lowering is stubbed (returns Unit)

4. **LIR Layer** (Partial):
   - `PerformEffect` lowering exists but stubbed ✅
   - Currently just returns placeholder value
   - Needs: Integration with async runtime

5. **LLVM Codegen** (Complete):
   - Can generate function calls ✅
   - Can generate external declarations ✅
   - Needs: Runtime function calls

6. **Async Runtime** (Complete):
   - Event loop works ✅
   - Sleep operation works ✅
   - Effect operations defined ✅
   - But: No compiler integration yet

### What's Missing ❌

1. **MIR → Async Runtime**: How to convert `PerformEffect` to `AsyncOperation`?
2. **Continuation Capture**: How to capture execution state at async points?
3. **Handler Dispatch**: How to find and call effect handlers?
4. **LLVM Integration**: How to generate runtime calls?

---

## Integration Architecture

### High-Level Flow

```
ZULON Code
    ↓
Parser (AST)
    ↓
HIR (Effect Declaration)
    ↓
MIR (PerformEffect Instruction)
    ↓
LIR (Runtime Function Call) ← **WE ARE HERE**
    ↓
LLVM IR (External Function Call)
    ↓
Native Code (Async Runtime)
```

### Design Strategy

**Key Insight**: Use external functions as the bridge between LIR and the async runtime.

**Approach**:
1. Define external functions in LLVM for async operations
2. LIR generates calls to these external functions
3. Runtime implements these functions in Rust
4. Compiler links with runtime at build time

---

## Detailed Integration Plan

### Phase 1: External Function Declarations (Week 1)

**Objective**: Declare async runtime functions that ZULON code can call

**LLVM External Declarations**:
```llvm
; File: async_runtime.ll

declare void @async_sleep(i64 %duration_ms)
declare i8* @async_file_read(i8* %path)
declare void @async_file_write(i8* %path, i8* %data)
declare i32 @async_tcp_connect(i8* %host, i16 %port)
declare i8* @async_tcp_read(i32 %fd, i64 %len)
declare void @async_tcp_write(i32 %fd, i8* %data)
```

**ZULON Usage**:
```zulon
effect Async {
    fn sleep(duration_ms: i64) -> ()
    fn read(path: string) -> string
    fn write(path: string, data: string) -> ()
}

fn main() -> i32 {
    try {
        Async::sleep(1000)  // Generates call to @async_sleep
        let data = Async::read("file.txt")  // Generates call to @async_file_read
    } with Async {
        // Event loop
    }
    0
}
```

**Implementation**:
1. Add external declarations to `zulon-tools-yan/src/build.rs`
2. Map ZULON effect operations to external functions
3. Inject declarations during compilation

**Deliverables**:
- External function declarations injected
- Mapping table: effect operation → external function

---

### Phase 2: LIR Lowering Enhancement (Week 2)

**Objective**: Generate external function calls from `PerformEffect`

**Current LIR Lowering** (Stub):
```rust
MirInstruction::PerformEffect { dest, effect_name, operation_name, args, return_type } => {
    // Currently just returns Unit
    Ok(vec![
        LirInstruction::Const {
            dest: dest_vreg,
            value: LirConstant::Unit,
            ty: return_type.into(),
        }
    ])
}
```

**Enhanced LIR Lowering** (Real Implementation):
```rust
MirInstruction::PerformEffect { dest, effect_name, operation_name, args, return_type } => {
    // Map effect operation to external function
    let extern_name = match (effect_name.as_str(), operation_name.as_str()) {
        ("Async", "sleep") => "async_sleep",
        ("Async", "read") => "async_file_read",
        ("Async", "write") => "async_file_write",
        _ => return Err(...),
    };

    // Generate external function call
    Ok(vec![
        LirInstruction::Call {
            dest: dest_vreg,
            func_name: extern_name.to_string(),
            args: args_to_vregs(args),
            return_type: return_type.into(),
        }
    ])
}
```

**Implementation Tasks**:
1. Create `effect_to_extern_map()` function
2. Generate `Call` instruction instead of `Const`
3. Handle arguments correctly (convert MirPlace to VReg)
4. Handle return types correctly

**Deliverables**:
- Enhanced `PerformEffect` lowering
- Mapping table implementation
- Tests for external call generation

---

### Phase 3: Runtime Function Implementation (Week 3)

**Objective**: Implement the external functions in the runtime

**Approach A: Simple (Recommended for MVP)**
```rust
// File: runtime/async_runtime_impl.c

void async_sleep(int64_t duration_ms) {
    // Direct implementation using thread sleep
    usleep(duration_ms * 1000);
}

char* async_file_read(const char* path) {
    // Direct file read (blocking for now)
    FILE* f = fopen(path, "r");
    // ... read file
    fclose(f);
    return buffer;
}
```

**Approach B: Integrated with Event Loop (Full Implementation)**
```rust
// File: crates/zulon-async-runtime/src/ffi.rs

use std::ffi::CString;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn async_sleep(duration_ms: i64) {
    // Submit to event loop
    let operation = AsyncOperation::Sleep { duration_ms };
    // Submit to global event loop...
}

#[no_mangle]
pub extern "C" fn async_file_read(path: *const c_char) -> *const c_char {
    // Convert path
    let path_str = unsafe { CString::from_raw(path as *mut i8) };
    let path_str = path_str.to_string_lossy().to_string();

    // Submit to event loop
    let operation = AsyncOperation::FileRead { path: path_str };
    // Submit and wait...
}
```

**Recommendation**: Start with Approach A (simple), migrate to Approach B later.

**Deliverables**:
- Runtime functions implemented (C or Rust)
- Linked into compiler output
- Tests passing

---

### Phase 4: Effect Handler Integration (Week 4)

**Objective**: Connect `try...with` blocks to the event loop

**ZULON Code**:
```zulon
effect Async {
    fn read(path: string) -> string
}

fn main() -> i32 {
    try {
        let data = Async::read("file.txt")
        printf("%s\n", data)
    } with Async {
        // This block needs to:
        // 1. Create event loop
        // 2. Run event loop until completion
        // 3. Return results
    }
    0
}
```

**Compiler Transformation** (MIR → LLVM):

**Current MIR**:
```
EffectCall("Async", "read", ["file.txt"], dest=t1)
  ↓
EffectHandlerBlock("Async")  // Currently does nothing
```

**Target LLVM IR**:
```llvm
; Call async function
call void @async_file_read(i8* %file_txt, i8** %result_ptr)

; Effect handler (try...with block)
call void @async_handler_enter()
; ... event loop setup ...
call void @async_run_event_loop()
call void @async_handler_exit()
```

**Implementation Tasks**:
1. Generate event loop setup code
2. Generate event loop run code
3. Pass continuation pointer to runtime
4. Handle results from async operations

**Deliverables**:
- Effect handler code generation
- Event loop integration
- End-to-end test passing

---

### Phase 5: Continuation Capture (Week 5)

**Objective**: Capture and restore execution state at async points

**Challenge**: When `Async::read()` is called, we need to:
1. Save the current execution state (stack, registers, instruction pointer)
2. Suspend execution
3. Resume when operation completes

**Approach**: Stackful Coroutines

**Simplified Strategy** (for MVP):
- Don't capture full continuation yet
- Use blocking calls that appear async to user
- Event loop runs in background
- Future: Implement full coroutines

**Full Strategy** (for later):
- Use stack switching (makecontext, fibers)
- Or use compiler-generated state machines
- Capture stack pointer, registers, IP
- Restore on completion

**Recommendation**: Start with simplified strategy, implement full coroutines later.

---

## Integration Timeline

### Week 1: External Declarations
- [ ] Declare async functions in LLVM
- [ ] Inject via build tool
- [ ] Test declarations appear in IR

### Week 2: LIR Lowering
- [ ] Map effects to extern functions
- [ ] Generate Call instructions
- [ ] Test external calls generated

### Week 3: Runtime Implementation
- [ ] Implement simple runtime functions
- [ ] Link with compiler output
- [ ] Test end-to-end (blocking)

### Week 4: Handler Integration
- [ ] Generate event loop setup code
- [ ] Generate event loop run code
- [ ] Test try...with blocks

### Week 5: Validation
- [ ] Write comprehensive tests
- [ ] Performance benchmarks
- [ ] Documentation
- [ ] Go/no-go decision

---

## Success Criteria

### Minimal Viable Integration (MVP)

**Must Have**:
- [ ] ZULON code can call `Async::sleep()`
- [ ] Sleep operation actually works (delays execution)
- [ ] No compiler errors
- [ ] Simple test passes

**Should Have**:
- [ ] `Async::read()` works (blocking file read)
- [ ] `try...with` blocks generate event loop code
- [ ] Runtime linked correctly

**Nice to Have**:
- [ ] Full continuation capture
- [ ] Non-blocking operations
- [ ] Multiple concurrent operations

---

## Risks & Mitigation

### Risk 1: LLVM Integration Complexity

**Risk**: Generating correct LLVM IR for async operations is complex

**Mitigation**: Start with simple blocking calls, add async later

### Risk 2: Continuation Capture

**Risk**: Capturing full execution state is very complex

**Mitigation**: Use simplified "appears async" approach first

### Risk 3: Event Loop Integration

**Risk**: Integrating event loop with compiler is tricky

**Mitigation**: Generate explicit event loop calls, hide complexity in runtime

### Risk 4: Performance

**Risk**: Async operations might be slow

**Mitigation**: Benchmark early, optimize hot paths

---

## Next Steps

### Immediate (Iteration 18)

1. ✅ Document integration plan (this document)
2. ✅ Understand current architecture
3. ✅ Identify integration points
4. [ ] Create external function declarations
5. [ ] Implement simple LIR lowering

### Next Phase (Iteration 19-20)

1. Implement Phase 1 (External Declarations)
2. Implement Phase 2 (LIR Lowering)
3. Implement Phase 3 (Runtime Functions)
4. Test end-to-end

---

## Conclusion

This integration plan provides a **clear, incremental path** from the current state (stubbed effect lowering) to a working async system. By starting with simple blocking calls and gradually adding sophistication, we can validate the approach at each step while making steady progress.

**Key Strategy**: Incremental implementation with continuous validation
**Timeline**: 5 weeks for full integration
**Risk Level**: Medium (well-understood problem, clear solution)

**Recommendation**: ✅ **Proceed with Phase 1 (External Declarations) in Iteration 19**

---

**Document Status**: ✅ Complete - Ready for implementation
**Next Action**: Begin Iteration 19 with Phase 1 implementation
