# Ralph Loop - Next Steps Quick Reference

**Last Updated**: 2026-01-09
**Iteration**: 1 Complete → 2 Starting
**Focus**: MVP Completion (Phase 1)

---

## Immediate Action Items (Next 2-3 Weeks)

### 1. Complete Test Framework ⭐ HIGH PRIORITY

**Status**: Partial - Test discovery works, execution needs implementation

**Tasks**:
- [ ] Implement `#[test]` macro expansion to test functions
- [ ] Implement assertion macros: `assert!`, `assert_eq!`, `assert_ne!`
- [ ] Build test runner that:
  - Discovers all `#[test]` functions
  - Executes each test
  - Reports pass/fail status
  - Captures test output
- [ ] Add test for the test framework itself

**Files to Modify**:
- `crates/zulon-macros/src/lib.rs` - Test macro
- `crates/zulon-compiler/src/lib.rs` - Test runner
- `crates/zulon-tests-integration/` - Test suite

**Success Criteria**:
```bash
yan test
# Running 3 tests...
# test test_addition ... ok
# test test_multiply ... ok
# test test_division ... ok
#
# Test result: ok. 3 passed; 0 failed
```

**Estimated Time**: 1-2 weeks

---

### 2. Complete Error Handling Features ⭐ HIGH PRIORITY

**Status**: Partial - Syntax supported, codegen needs completion

**Tasks**:
- [ ] Verify `throw` statement code generation
- [ ] Verify `?` operator code generation
- [ ] Verify `|` type separator (T|E → Outcome<T,E>)
- [ ] Test error propagation through call stack
- [ ] Ensure Outcome<T,E> type is complete

**Test Program**:
```zulon
enum MathError {
    DivisionByZero,
}

fn divide(a: i32, b: i32) -> i32 | MathError {
    if b == 0 {
        throw MathError::DivisionByZero;
    }
    a / b
}

fn main() -> i32 {
    let result = divide(10, 2)?;
    match result {
        Ok(value) => println("Result: {}", value),
        Err(MathError::DivisionByZero) => println("Error: Division by zero"),
    }
    0
}
```

**Files to Modify**:
- `crates/zulon-parser/src/parser/mod.rs` - Parse throw/?/|
- `crates/zulon-hir/src/hir.rs` - HIR nodes
- `crates/zulon-mir/src/mir.rs` - MIR instructions
- `crates/zulon-codegen-llvm/src/codegen.rs` - LLVM codegen

**Success Criteria**: Above program compiles and runs correctly

**Estimated Time**: 1 week

---

### 3. Complete Advanced Control Flow ⭐ HIGH PRIORITY

**Status**: Basic if/else works, advanced patterns pending

**Tasks**:
- [ ] Complete `loop { ... }` expression
- [ ] Complete `while cond { ... }` loop
- [ ] Complete `for pat in iter { ... }` loop
- [ ] Complete `match expr { pat => expr, ... }` with patterns:
  - Literals
  - Wildcard (`_`)
  - Struct patterns
  - Enum patterns
  - Tuple patterns
  - Guards (`if`)

**Test Program**:
```zulon
fn main() -> i32 {
    // Loop
    let mut x = 0;
    loop {
        x = x + 1;
        if x == 10 {
            break;
        }
    }

    // While
    let mut y = 0;
    while y < 5 {
        y = y + 1;
    }

    // For
    let sum = 0;
    for i in 0..10 {
        sum = sum + i;
    }

    // Match
    let result = match x {
        0 => "zero",
        1 | 2 => "small",
        n if n < 10 => "medium",
        _ => "large",
    };

    0
}
```

**Files to Modify**:
- `crates/zulon-parser/src/parser/mod.rs` - Parse control flow
- `crates/zulon-hir/src/hir.rs` - HIR representation
- `crates/zulon-mir/src/mir.rs` - MIR control flow
- `crates/zulon-lir/src/lir.rs` - LIR terminators

**Success Criteria**: Above program compiles and runs correctly

**Estimated Time**: 1-2 weeks

---

## Secondary Priorities (After MVP Core)

### 4. Closure Support (2 weeks)

**Tasks**:
- [ ] Parse closure syntax: `fn|args| body` or `|args| body`
- [ ] Implement capture analysis (what variables are captured)
- [ ] Type inference for closures
- [ ] Code generation for closure calls

**Example**:
```zulon
fn main() -> i32 {
    let x = 10;
    let add = fn|y: i32| -> i32 { x + y };
    add(5)
}
```

---

### 5. String Interpolation (1 week)

**Tasks**:
- [ ] Parse `${expr}` inside strings
- [ ] Implement template string syntax (backticks)
- [ ] Code generation for string concatenation
- [ ] Add to_string() method for types

**Example**:
```zulon
fn main() -> i32 {
    let name = "ZULON";
    let version = 1;
    println("Welcome to ${name} v${version}!");
    0
}
```

---

### 6. Improve Error Messages (1 week)

**Tasks**:
- [ ] Add span information to all error types
- [ ] Implement error highlighting in output
- [ ] Add suggestions for common mistakes
- [ ] Colorize terminal output

**Example Output**:
```
error: type mismatch
  --> examples/test.zl:5:10
   |
5  |     let x: i32 = "hello";
   |          ^^^^^^^   ------- expected i32, found &str
   |
   = note: expected type `i32`
              found type `&str`
   = help: consider removing the type annotation
   = help: or change the value to an integer: `42`
```

---

## Testing Strategy

### Unit Tests
- Each crate should have comprehensive unit tests
- Target: 80%+ code coverage

### Integration Tests
- Test end-to-end compilation pipeline
- Test standard library functionality
- Test error handling scenarios

### Example Programs
- Maintain working examples in `examples/working/`
- Each example should compile and run correctly
- Test with: `yan run examples/working/program.zl`

### Test Framework Tests
- Once test framework is complete, add tests for:
  - Type system edge cases
  - Error handling
  - Control flow
  - Standard library functions

---

## Daily Workflow

### Morning Routine
1. Pull latest changes
2. Run `cargo check --workspace`
3. Run `cargo test --workspace`
4. Review yesterday's work

### Development Workflow
1. Pick a task from this list
2. Create a feature branch: `git checkout -b feature/feature-name`
3. Implement the feature
4. Write tests
5. Run full test suite
6. Commit changes
7. Document in iteration report

### Evening Routine
1. Update todo list
2. Run full test suite
3. Push changes to branch
4. Update status report
5. Plan tomorrow's work

---

## Progress Tracking

### MVP Completion Checklist

**Compiler Pipeline**:
- [x] Lexer
- [x] Parser
- [x] Type Checker
- [x] HIR
- [x] MIR
- [x] LIR
- [x] LLVM Codegen
- [x] Linking

**Language Features**:
- [x] Functions
- [x] Variables (let, let mut)
- [x] Basic types (i32, f64, bool, str)
- [x] Binary/unary operations
- [x] If/else expressions
- [ ] Loops (loop, while, for)
- [ ] Match expressions
- [ ] Closures
- [ ] String interpolation

**Error Handling**:
- [ ] throw statement
- [ ] ? operator
- [ ] | type separator
- [x] Outcome<T,E> type

**Standard Library**:
- [x] Vec<T>
- [x] HashMap<K,V>
- [x] HashSet<T>
- [x] VecDeque<T>
- [ ] LinkedList<T>
- [ ] BTreeMap<K,V>
- [ ] BTreeSet<T>

**Tools**:
- [x] yan build
- [x] yan run
- [x] yan new
- [x] yan clean
- [ ] yan test
- [ ] yan fmt
- [ ] yan doc

**Testing**:
- [ ] Test framework
- [ ] Assertion macros
- [ ] Test runner
- [ ] Test discovery

---

## Reference Links

- **Implementation Plan**: `IMPLEMENTATION_PLAN.md`
- **TODO List**: `TODOLIST.md`
- **README**: `README.md`
- **Iteration Reports**: `RALPH_LOOP_ITERATION_*.md`

---

## Contact and Support

**Questions**: Refer to implementation plan first
**Blockers**: Document in iteration report
**Ideas**: Add to TODO list with priority tag

**Remember**: The goal is MVP completion. Focus on core features first.
