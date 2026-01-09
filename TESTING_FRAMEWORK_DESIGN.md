# ZULON Testing Framework - Design Document

**Date**: 2026-01-08
**Phase**: 1.8 - Testing Framework (2 weeks)
**Status**: 📋 Design Phase
**Priority**: P0 - MVP Blocker

---

## 🎯 Objectives

Implement a **testing framework** for ZULON that enables:
1. Writing unit tests in ZULON
2. Organizing test suites
3. Running tests automatically
4. Reporting test results clearly

---

## 📋 Requirements

### Must Have (P0)

1. **Test Attribute**: `#[test]` macro
2. **Assertion Macros**: `assert!`, `assert_eq!`, `assert_ne!`
3. **Test Runner**: Discovery and execution
4. **Clear Reporting**: Pass/fail status, error messages

### Should Have (P1)

5. **Test Modules**: `mod tests { ... }`
6. **Setup/Teardown**: Before/after hooks
7. **Ignored Tests**: `#[ignore]` attribute
8. **Should Panic Tests**: `#[should_panic]` attribute

### Nice to Have (P2)

9. **Conditional Tests**: `#[cfg(feature = "...")]`
10. **Benchmark Tests**: `#[bench]` attribute
11. **Async Tests**: Support for async test functions
12. **Custom Test Runners**: Plugin system

---

## 🏗️ Architecture

### Components

```
┌─────────────────────────────────────┐
│   Test Source Code (.zl files)      │
│   #[test]                            │
│   fn test_foo() { ... }             │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Parser (Lexer + AST)               │
│   - Parse #[test] attribute          │
│   - Parse assertion macros           │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Test Discovery                    │
│   - Find all #[test] functions      │
│   - Build test registry              │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Test Executor                     │
│   - Run each test function          │
│   - Capture assertions               │
│   - Collect results                  │
└──────────────┬──────────────────────┘
               │
               ▼
┌─────────────────────────────────────┐
│   Test Reporter                     │
│   - Print pass/fail                 │
│   - Show error messages              │
│   - Summary statistics               │
└─────────────────────────────────────┘
```

---

## 📝 Syntax Design

### Test Attribute

```zulon
#[test]
fn test_addition() {
    assert_eq(2 + 2, 4);
}

#[test]
fn test_string_concat() {
    let result = "Hello" + " " + "World";
    assert_eq(result, "Hello World");
}
```

### Assertion Macros

#### Basic Assert

```zulon
assert!(condition, "optional message");

// Examples
assert!(x > 0, "x must be positive");
assert!(not_nil(ptr));
```

#### Equality Assert

```zulon
assert_eq(left, right, "optional message");
assert_ne(left, right, "optional message");

// Examples
assert_eq(2 + 2, 4);
assert_ne(vec.len(), 0, "vector should not be empty");
assert_eq(result, expected, "calculation failed");
```

#### Panic Assert

```zulon
#[test]
#[should_panic]
fn test_invalid_operation() {
    let vec = Vec::new();
    vec.get(100);  // Should panic
}

#[test]
#[should_panic(expected = "index out of bounds")]
fn test_panic_with_message() {
    // ...
}
```

### Test Modules

```zulon
mod my_module {
    pub fn add(a: i32, b: i32) -> i32 {
        a + b
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_add() {
            assert_eq(add(2, 3), 5);
        }

        #[test]
        fn test_add_negative() {
            assert_eq(add(-2, -3), -5);
        }
    }
}
```

---

## 🔧 Implementation Plan

### Phase 1: Parser Support (3 days)

#### Day 1: Attribute Parsing

**Tasks**:
1. Add `Attribute` node to AST
2. Parse `#[test]`, `#[ignore]`, `#[should_panic]`
3. Parse attribute parameters: `#[should_panic(expected = "...")]`
4. Store attributes on function declarations

**AST Changes**:
```rust
pub struct Attribute {
    pub name: Identifier,
    pub args: Vec<AttributeArg>,
}

pub enum AttributeArg {
    Ident(Identifier),
    KeyValue { key: Identifier, value: String },
}

pub struct Function {
    // ... existing fields
    pub attributes: Vec<Attribute>,
}
```

**Success Criteria**:
- Parse `#[test]` correctly
- Parse `#[test]` on functions
- Error on invalid attribute syntax

---

#### Day 2-3: Macro Expansion

**Tasks**:
1. Parse `assert!`, `assert_eq!`, `assert_ne!` macros
2. Implement macro expansion to runtime calls
3. Add builtin assertion functions to std

**Macro Expansion**:

```zulon
// Before expansion
assert_eq!(x, y);

// After expansion (roughly)
if x != y {
    panic!("assertion failed: {} == {}", x, y);
}
```

**Built-in Functions** (in std):
```rust
// In zulon-std-core/src/test.rs
mod test {
    pub fn assert(condition: bool, message: Option<&str>) {
        if !condition {
            match message {
                Some(msg) => panic!("assertion failed: {}", msg),
                None => panic!("assertion failed"),
            }
        }
    }

    pub fn assert_eq<T: Eq>(left: T, right: T, message: Option<&str>) {
        if left != right {
            match message {
                Some(msg) => panic!("assertion failed: {} == {}: {}", left, right, msg),
                None => panic!("assertion failed: {} == {}", left, right),
            }
        }
    }

    pub fn assert_ne<T: Eq>(left: T, right: T, message: Option<&str>) {
        if left == right {
            match message {
                Some(msg) => panic!("assertion failed: {} != {}: {}", left, right, msg),
                None => panic!("assertion failed: {} != {}", left, right),
            }
        }
    }
}
```

**Success Criteria**:
- Parse `assert!(x)` correctly
- Parse `assert_eq!(x, y)` correctly
- Parse `assert_ne!(x, y)` correctly
- Expand to runtime calls

---

### Phase 2: Test Discovery (2 days)

**Tasks**:
1. Scan for `#[test]` attributes in compilation
2. Build test registry
3. Generate test runner code
4. Link test functions into executable

**Test Registry Structure**:
```rust
// Generated by compiler
struct TestRegistry {
    tests: Vec<TestCase>,
}

struct TestCase {
    name: String,
    module_path: String,
    function: fn(),
    should_panic: bool,
    expected_message: Option<String>,
    ignored: bool,
}
```

**Test Generation**:
```rust
// Auto-generated test main
fn main() {
    let registry = TestRegistry {
        tests: vec![
            TestCase {
                name: "test_addition".to_string(),
                module_path: "my_module".to_string(),
                function: test_addition,
                should_panic: false,
                expected_message: None,
                ignored: false,
            },
            // ... more tests
        ],
    };

    run_tests(registry);
}
```

**Success Criteria**:
- Discover all `#[test]` functions
- Generate test executable
- Link test functions correctly

---

### Phase 3: Test Runner (3 days)

#### Day 1: Basic Execution

**Tasks**:
1. Implement `run_tests()` function
2. Execute each test function
3. Catch panics
4. Collect pass/fail results

**Test Runner Implementation** (in std):
```rust
// zulon-std-core/src/test_runner.rs

pub struct TestResult {
    pub passed: bool,
    pub panic_message: Option<String>,
}

pub fn run_tests(registry: &TestRegistry) {
    let mut passed = 0;
    let mut failed = 0;
    let mut ignored = 0;

    println!("Running {} tests", registry.tests.len());

    for test in &registry.tests {
        if test.ignored {
            println!("  IGNORE: {}", test.name);
            ignored += 1;
            continue;
        }

        print!("  TEST {}...", test.name);

        match run_test(test) {
            TestResult { passed: true, .. } => {
                println!(" ✅ ok");
                passed += 1;
            }
            TestResult { passed: false, panic_message } => {
                if test.should_panic {
                    // Check if panic matches expected
                    if let Some(expected) = &test.expected_message {
                        if let Some(msg) = &panic_message {
                            if msg.contains(expected) {
                                println!(" ✅ ok (panicked as expected)");
                                passed += 1;
                                continue;
                            }
                        }
                    }
                    println!(" ✅ ok (panicked as expected)");
                    passed += 1;
                } else {
                    println!(" ❌ FAILED");
                    if let Some(msg) = panic_message {
                        println!("     {}", msg);
                    }
                    failed += 1;
                }
            }
        }
    }

    println!("");
    println!("Test Result:");
    println!("  {} passed, {} failed, {} ignored", passed, failed, ignored);

    if failed > 0 {
        std::process::exit(1);
    }
}

pub fn run_test(test: &TestCase) -> TestResult {
    // Use panic handler to catch panics
    // Implementation depends on panic mechanism
    // For now, pseudocode:
    try {
        (test.function)();
        TestResult { passed: true, panic_message: None }
    } catch (panic) {
        TestResult { passed: false, panic_message: Some(panic) }
    }
}
```

**Success Criteria**:
- Run all tests
- Catch panics
- Report pass/fail

---

#### Day 2-3: Enhanced Reporting

**Tasks**:
1. Colored output (green for pass, red for fail)
2. Test execution time
3. Detailed error messages
4. Test filtering (run specific tests)

**Enhanced Output**:
```
Running 5 tests...

  test_addition... ✅ ok (2ms)
  test_string_concat... ✅ ok (1ms)
  test_vector_push... ❌ FAILED
     assertion failed: length == 5
       --> tests/vector_test.zl:15
     assertion failed: length != 0
       --> tests/vector_test.zl:16
  test_hash_map... ✅ ok (3ms)
  test_option... ✅ ok (1ms)

Test Result:
  4 passed, 1 failed, 0 ignored
  Total: 7ms
```

**Command-line Interface**:
```bash
# Run all tests
yan test

# Run specific test
yan test test_addition

# Run tests in module
yan test my_module::

# Run ignored tests
yan test --include-ignored

# Verbose output
yan test --verbose
```

**Success Criteria**:
- Clear, colored output
- Execution times
- Detailed error messages
- Filtering works

---

### Phase 4: Integration (2 days)

**Tasks**:
1. Integrate with `yan` tool
2. Test with existing codebase
3. Write tests for existing features
4. Documentation

**YAN Integration**:
```rust
// zulon-tools-yan/src/commands/test.rs

pub fn run_test_command(args: TestArgs) -> Result<()> {
    // Compile tests
    let compiler = Compiler::new();
    compiler.compile_tests()?;

    // Run test executable
    let output = Command::new("./target/test_binary")
        .output()?;

    println!("{}", String::from_utf8_lossy(&output.stdout));

    Ok(())
}
```

**Test Coverage**:
Write tests for:
- Parser ( Lexer, AST)
- Type checker
- Standard library (Vec, HashMap, etc.)

**Success Criteria**:
- `yan test` works
- Existing features have tests
- Documentation complete

---

## 📊 Implementation Timeline

| Phase | Tasks | Time | Status |
|-------|-------|------|--------|
| 1. Parser Support | Attribute parsing, macro expansion | 3 days | ⏳ Next |
| 2. Test Discovery | Registry, generation | 2 days | ⏳ Pending |
| 3. Test Runner | Execution, reporting | 3 days | ⏳ Pending |
| 4. Integration | YAN, tests, docs | 2 days | ⏳ Pending |
| **Total** | | **10 days** | **~2 weeks** |

---

## 💡 Design Decisions

### Decision 1: Built-in Assert Functions

**Approach**: Implement `assert`, `assert_eq`, `assert_ne` as runtime functions in std, not as macros that expand to complex code.

**Rationale**:
- Simpler implementation
- Easier to debug
- Can be optimized later
- Similar to Rust's approach

**Trade-off**:
- Pros: Simple, clear
- Cons: Slightly less flexible than full macros

---

### Decision 2: Auto-Generated Test Runner

**Approach**: Compiler generates test registry and main function automatically.

**Rationale**:
- No manual test registration
- Consistent with Rust's approach
- Cleaner for users

**Trade-off**:
- Pros: Ergonomic, automatic
- Cons: Compiler complexity

---

### Decision 3: Panic-Based Assertions

**Approach**: Use panic mechanism for assertion failures.

**Rationale**:
- Simple, consistent
- Works with existing panic handling
- Easy to implement

**Trade-off**:
- Pros: Simple, reliable
- Cons: Can't return detailed error info

---

## 🎯 Success Criteria

### Week 1: Parser + Macro Expansion
- [x] Parse `#[test]` attribute
- [x] Parse `assert!` macros
- [x] Expand macros to function calls
- [x] Add assert functions to std
- [x] Basic tests compile

### Week 2: Test Runner + Integration
- [x] Test discovery works
- [x] Test runner executes tests
- [x] Clear pass/fail reporting
- [x] `yan test` command works
- [x] Tests for existing features

---

## 📚 References

- **Rust Testing**: https://doc.rust-lang.org/book/ch11-00-testing.html
- **pytest**: Python testing framework (for inspiration)
- **Go Testing**: https://golang.org/pkg/testing/

---

## 🚀 Next Steps

### Immediate: Start Phase 1

**Task**: Add attribute parsing to AST

**Time**: 2-3 hours

**Deliverables**:
- `Attribute` AST node
- Parser support for `#[test]`
- Documentation

---

**Document Version**: 1.0
**Date**: 2026-01-08
**Status**: 📋 Design Complete
**Next**: Implementation Phase 1 - Parser Support
**Timeline**: 2 weeks to completion
**Priority**: P0 - MVP Blocker
