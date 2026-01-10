# Phase 1.8 - Testing Framework Design

**Date**: 2026-01-10
**Status**: 🎯 **Design Phase**
**Target**: 2 weeks (2026-01-10 ~ 2026-01-24)

---

## Executive Summary

This document outlines the design and implementation plan for ZULON's testing framework. The framework will enable developers to write unit tests directly in `.zl` files using a `#[test]` attribute and assertion macros, similar to Rust's testing approach.

**Goals**:
- Enable in-language unit testing
- Provide clear assertion macros
- Integrate with YAN toolchain
- Support test discovery and execution

**Non-Goals** (deferred to Phase 2.7):
- Parameterized tests
- Async test support
- Coverage reporting
- Benchmarks

---

## Architecture Overview

### Components

```
┌─────────────────────────────────────────────────┐
│  User Code (.zl files)                           │
│  ┌───────────────────────────────────────────┐  │
│  │ #[test]                                     │  │
│  │ fn test_addition() { ... }                  │  │
│  └───────────────────────────────────────────┘  │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  Parser Extension                               │
│  - Recognize #[test] attribute                   │
│  - Add to AST node                              │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  HIR/MIR Lowering                               │
│  - Mark test functions                           │
│  - Generate special metadata                     │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  LLVM Code Generation                           │
│  - Emit test function code                       │
│  - Register in test section                      │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  Test Runtime (NEW)                             │
│  - Test discovery                                │
│  - Test execution                                │
│  - Result reporting                              │
└─────────────────────────────────────────────────┘
           ↓
┌─────────────────────────────────────────────────┐
│  YAN Integration                                │
│  - yan test command                              │
│  - Collect and run tests                         │
└─────────────────────────────────────────────────┘
```

---

## 1. Syntax Design

### 1.1 Test Attribute

**Syntax**:
```zulon
#[test]
fn test_function_name() {
    // Test code here
}
```

**Rules**:
- Test functions must be marked with `#[test]` attribute
- Test functions take no parameters
- Test functions should return `()` (unit type)
- Functions NOT marked with `#[test]` are normal code

**Example**:
```zulon
#[test]
fn test_addition() {
    let result = 2 + 2;
    assert!(result == 4);
}

#[test]
fn test_multiplication() {
    assert_eq!(6, 2 * 3);
}

fn main() -> i32 {
    // Normal code
    0
}
```

### 1.2 Assertion Macros

#### assert!(condition)

**Purpose**: Assert that a boolean condition is true

**Syntax**:
```zulon
assert!(condition);
assert!(condition, "error message");
```

**Expansion** (simplified):
```zulon
if !condition {
    panic!("assertion failed");
}
```

**Examples**:
```zulon
#[test]
fn test_comparison() {
    let x = 10;
    let y = 20;
    assert!(x < y);
    assert!(x + y == 30);
}
```

#### assert_eq!(left, right)

**Purpose**: Assert that two values are equal

**Syntax**:
```zulon
assert_eq!(left, right);
assert_eq!(left, right, "error message");
```

**Expansion** (simplified):
```zulon
if left != right {
    panic!("assertion failed: left != right");
}
```

**Examples**:
```zulon
#[test]
fn test_equality() {
    assert_eq!(42, 40 + 2);
    assert_eq!("hello", "hello");
}
```

#### assert_ne!(left, right)

**Purpose**: Assert that two values are NOT equal

**Syntax**:
```zulon
assert_ne!(left, right);
assert_ne!(left, right, "error message");
```

**Expansion** (simplified):
```zulon
if left == right {
    panic!("assertion failed: left == right");
}
```

**Examples**:
```zulon
#[test]
fn test_inequality() {
    assert_ne!(10, 20);
    assert_ne!("foo", "bar");
}
```

### 1.3 Panic Macro

**Purpose**: Immediately fail a test with a message

**Syntax**:
```zulon
panic!();
panic!("message");
panic!("format: {}", value);
```

**Examples**:
```zulon
#[test]
fn test_unreachable() {
    if some_condition {
        panic!("This should never happen!");
    }
}
```

---

## 2. Parser Implementation

### 2.1 AST Extensions

**File**: `crates/zulon-parser/src/ast/mod.rs`

**Add Attribute Node**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: String,
    pub args: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    // ... existing fields ...
    pub attrs: Vec<Attribute>,  // NEW: List of attributes
}
```

### 2.2 Parser Extensions

**File**: `crates/zulon-parser/src/parser/mod.rs`

**Parse Attributes**:
```rust
// Before parsing function
fn parse_attributes(&mut self) -> Result<Vec<Attribute>, ParseError> {
    let mut attrs = Vec::new();

    while self.peek_token() == Some(&Token::Hash) {
        self.consume(Token::Hash)?;
        self.consume(Token::LeftBracket)?;
        let name = self.parse_identifier()?;
        let args = if self.peek_token() == Some(&Token::LeftParen) {
            self.parse_paren_args()?  // e.g., #[test(arg1, arg2)]
        } else {
            Vec::new()
        };
        self.consume(Token::RightBracket)?;

        attrs.push(Attribute {
            name,
            args,
            span: self.span(),
        });
    }

    Ok(attrs)
}
```

**Recognize #[test]**:
```rust
fn parse_function(&mut self) -> Result<Function, ParseError> {
    let attrs = self.parse_attributes()?;  // NEW

    // ... existing function parsing ...

    Ok(Function {
        // ... existing fields ...
        attrs,  // NEW
    })
}
```

### 2.3 Test Cases

**File**: `crates/zulon-parser/tests/test_attribute_tests.rs`

```rust
#[test]
fn parse_test_attribute() {
    let input = r#"
        #[test]
        fn test_add() { }
    "#;
    let mut parser = Parser::from_source(input);
    let func = parser.parse_function().unwrap();
    assert_eq!(func.attrs.len(), 1);
    assert_eq!(func.attrs[0].name, "test");
}

#[test]
fn parse_multiple_attributes() {
    let input = r#"
        #[test]
        #[ignore]
        fn test_something() { }
    "#;
    let mut parser = Parser::from_source(input);
    let func = parser.parse_function().unwrap();
    assert_eq!(func.attrs.len(), 2);
}
```

---

## 3. HIR/MIR Lowering

### 3.1 HIR Extensions

**File**: `crates/zulon-hir/src/hir.rs`

**Add is_test Field**:
```rust
pub struct HirFunction {
    // ... existing fields ...
    pub is_test: bool,  // NEW: true if marked with #[test]
}
```

### 3.2 AST → HIR Lowering

**File**: `crates/zulon-hir/src/simple_lower.rs`

```rust
fn lower_function(&mut self, func: &ast::Function) -> HirFunction {
    let is_test = func.attrs.iter().any(|a| a.name == "test");  // NEW

    HirFunction {
        // ... existing fields ...
        is_test,  // NEW
    }
}
```

### 3.3 MIR Treatment

**File**: `crates/zulon-mir/src/lower.rs`

**No special treatment needed** - test functions are just regular functions from MIR's perspective. The metadata is preserved for the test runtime.

---

## 4. LLVM Code Generation

### 4.1 Test Metadata

**Strategy**: Generate a special section that lists all test functions

**Approach 1: Symbol-based (Simpler)**
- Convention: Test functions are exported with a special name prefix
- Test runtime discovers them by symbol name pattern

**Approach 2: Metadata Section (Better)**
- Emit a custom LLVM section with test function metadata
- Includes: function name, file location, should_ignore flag

**Recommendation**: Start with **Approach 1** (simpler), upgrade to Approach 2 in Phase 2.7

### 4.2 Code Generation

**File**: `crates/zulon-codegen-llvm/src/codegen.rs`

```rust
impl<'a> CodeGenerator<'a> {
    pub fn generate_function(&mut self, func: &LirFunction) {
        // ... existing code generation ...

        // If it's a test function, we might want to:
        // - Mark it with a special attribute
        // - Ensure it's exported (not internal)
        if func.is_test {
            self.emit_test_metadata(func);
        }
    }

    fn emit_test_metadata(&mut self, func: &LirFunction) {
        // For Approach 1: Just ensure symbol is exported
        // For Approach 2: Emit metadata section
    }
}
```

---

## 5. Test Runtime

### 5.1 Design Options

**Option A: In-Language Test Runner**
- Write test runner in ZULON
- Uses reflection or metadata discovery
- Pro: Tests run in same environment as code
- Con: Requires reflection/metadata support

**Option B: External Test Runner (Recommended)**
- Test runner is external (Rust)
- Reads LLVM IR metadata or symbols
- Pro: Simpler, more control
- Con: Requires external tool

**Recommendation**: **Option B** for Phase 1.8

### 5.2 Test Runtime Implementation

**File**: `crates/zulon-test-runtime/src/lib.rs` (NEW CRATE)

```rust
pub struct Test {
    pub name: String,
    pub function_name: String,
    pub file: String,
    pub line: usize,
    pub should_ignore: bool,
}

pub struct TestResult {
    pub test: Test,
    pub passed: bool,
    pub error_message: Option<String>,
    pub duration: Duration,
}

pub trait TestRunner {
    fn discover_tests(&self, module: &CompiledModule) -> Vec<Test>;
    fn run_test(&self, test: &Test) -> TestResult;
    fn run_all(&self, tests: &[Test]) -> Vec<TestResult>;
}
```

### 5.3 Integration with LLVM

**Using LLVM JIT or Process-based**:

**Process-based Approach** (Simpler):
1. Compile test module to executable
2. Each test is a function in the executable
3. Test runner calls each test function via command-line flags
4. Test returns exit code: 0 = pass, 1 = fail

```rust
// main() with test support
fn main() -> i32 {
    if env::args().contains("--run-test") {
        let test_name = env::args().get("--test-name").unwrap();
        run_single_test(test_name)
    } else {
        // Normal main
        run_tests()  // Run all tests and report
    }
}
```

---

## 6. YAN Integration

### 6.1 yan test Command

**File**: `crates/zulon-tools-yan/src/commands/test.rs` (NEW)

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct TestCommand {
    /// Run only tests matching this pattern
    #[arg(short, long)]
    filter: Option<String>,

    /// Run tests in parallel (not implemented in Phase 1)
    #[arg(short, long)]
    parallel: bool,

    /// Show test output
    #[arg(short, long)]
    verbose: bool,

    /// Number of times to repeat each test (for flaky tests)
    #[arg(long, default_value = "1")]
    repeat: usize,
}

pub fn run_test_command(cmd: TestCommand) -> Result<()> {
    println!("🧪 Running tests...");

    // 1. Compile the project
    let module = compile_project()?;

    // 2. Discover tests
    let runner = NativeTestRunner::new();
    let tests = runner.discover_tests(&module)?;

    // 3. Filter tests if requested
    let tests = filter_tests(tests, cmd.filter)?;

    // 4. Run tests
    let results = runner.run_all(&tests)?;

    // 5. Print summary
    print_test_summary(&results);

    // 6. Exit with appropriate code
    let failed_count = results.iter().filter(|r| !r.passed).count();
    if failed_count > 0 {
        std::process::exit(1);
    }

    Ok(())
}
```

### 6.2 CLI Integration

**File**: `crates/zulon-tools-yan/src/main.rs`

```rust
#[derive(Subcommand)]
enum Commands {
    Build(BuildCommand),
    Run(RunCommand),
    New(NewCommand),
    Clean(CleanCommand),
    Test(TestCommand),  // NEW
}

fn main() -> Result<()> {
    match cli.command {
        Commands::Test(cmd) => run_test_command(cmd),
        // ... other commands ...
    }
}
```

---

## 7. Assertion Macro Implementation

### 7.1 Macro System Design

**Phase 1.8 Approach**: **Macro Expansion at Parse Time**

Since we don't have a full macro system yet, we'll implement assertion macros as special syntax that the parser recognizes and expands.

### 7.2 assert! Implementation

**Parser Recognition**:
```rust
// In parser, detect assert!(...)
fn parse_macro_invocation(&mut self) -> Result<Expr, ParseError> {
    match self.peek_token() {
        Some(Token::Identifier(name)) => {
            match name.as_str() {
                "assert" => self.parse_assert_macro(),
                "assert_eq" => self.parse_assert_eq_macro(),
                "assert_ne" => self.parse_assert_ne_macro(),
                "panic" => self.parse_panic_macro(),
                _ => self.parse_call_expr(),
            }
        }
        _ => self.parse_call_expr(),
    }
}
```

**AST Expansion**:
```rust
fn parse_assert_macro(&mut self) -> Result<Expr, ParseError> {
    self.consume_identifier("assert")?;
    self.consume(Token::Bang)?;
    self.consume(Token::LeftParen)?;

    let condition = self.parse_expr()?;

    let message = if self.peek_token() == Some(&Token::Comma) {
        self.consume(Token::Comma)?;
        Some(self.parse_string_literal()?)
    } else {
        None
    };

    self.consume(Token::RightParen)?;

    // Expand to: if !condition { panic!(message); }
    Ok(Expr::If {
        condition: Box::new(Expr::UnaryOp {
            op: UnaryOp::Not,
            expr: Box::new(condition),
        }),
        then_block: Block {
            statements: vec![Statement::Expr(Expr::MacroCall {
                macro_name: "panic".to_string(),
                args: message.map(|m| vec![Expr::String(m)]).unwrap_or_default(),
            })],
            trailing_expr: None,
        },
        else_block: None,
    })
}
```

### 7.3 assert_eq! Implementation

```rust
fn parse_assert_eq_macro(&mut self) -> Result<Expr, ParseError> {
    self.consume_identifier("assert_eq")?;
    self.consume(Token::Bang)?;
    self.consume(Token::LeftParen)?;

    let left = self.parse_expr()?;
    self.consume(Token::Comma)?;
    let right = self.parse_expr()?;

    let message = if self.peek_token() == Some(&Token::Comma) {
        self.consume(Token::Comma)?;
        Some(self.parse_string_literal()?)
    } else {
        None
    };

    self.consume(Token::RightParen)?;

    // Expand to: if left != right { panic!(...); }
    let error_msg = format!(
        "assertion failed: `(left == right)`\n  left: `{:?}`,\n  right: `{:?}`",
        /* left, right - in real impl */
    );

    Ok(Expr::If {
        condition: Box::new(Expr::BinaryOp {
            left: Box::new(left.clone()),
            op: BinaryOp::NotEq,
            right: Box::new(right.clone()),
        }),
        then_block: Block {
            statements: vec![Statement::Expr(Expr::MacroCall {
                macro_name: "panic".to_string(),
                args: vec![Expr::String(error_msg)],
            })],
            trailing_expr: None,
        },
        else_block: None,
    })
}
```

### 7.4 panic! Implementation

**For Phase 1.8**: Use runtime panic

```rust
fn parse_panic_macro(&mut self) -> Result<Expr, ParseError> {
    self.consume_identifier("panic")?;
    self.consume(Token::Bang)?;
    self.consume(Token::LeftParen)?;

    let message = if self.peek_token() != Some(&Token::RightParen) {
        Some(self.parse_expr()?)
    } else {
        None
    };

    self.consume(Token::RightParen)?;

    // For now, expand to a call to external panic function
    // In Phase 2, we'll have proper panic support
    Ok(Expr::Call {
        function: "builtin_panic".to_string(),
        args: message.map(|m| vec![m]).unwrap_or_default(),
    })
}
```

**LLVM Code Gen**:
```llvm
declare void @builtin_panic(ptr)

; panic!("message")
call void @builtin_panic(ptr @.panic_message)
```

---

## 8. Example Test File

**File**: `examples/test_example.zl`

```zulon
// Simple addition tests
#[test]
fn test_add_integers() {
    let result = 2 + 2;
    assert_eq!(result, 4);
}

#[test]
fn test_add_negative() {
    let result = -5 + 3;
    assert_eq!(result, -2);
}

// Comparison tests
#[test]
fn test_comparison() {
    assert!(10 < 20);
    assert!(20 > 10);
    assert!(10 <= 10);
    assert!(10 >= 10);
}

// String tests (when strings work)
#[test]
fn test_string_equality() {
    assert_eq!("hello", "hello");
    assert_ne!("hello", "world");
}

// Function tests
fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[test]
fn test_add_function() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, 1), 0);
}

// Ignored test (future)
#[test]
#[ignore]
fn test_not_implemented_yet() {
    panic!("This test is not implemented yet");
}

// Main function (not a test)
fn main() -> i32 {
    println!("Running example program");
    0
}
```

**Expected Output**:
```bash
$ yan test

running 5 tests
test test_add_integers ... ok
test test_add_negative ... ok
test test_comparison ... ok
test test_string_equality ... ok
test test_add_function ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

---

## 9. Implementation Timeline

### Week 1 (2026-01-10 ~ 2026-01-17)

**Day 1-2**: Parser Extensions
- [ ] Add Attribute to AST
- [ ] Implement #[test] parsing
- [ ] Add tests for attribute parsing
- [ ] Update AST → HIR lowering

**Day 3-4**: Assertion Macros
- [ ] Implement assert! macro expansion
- [ ] Implement assert_eq! macro expansion
- [ ] Implement assert_ne! macro expansion
- [ ] Implement panic! macro expansion
- [ ] Add macro tests

**Day 5**: HIR/MIR Updates
- [ ] Add is_test field to HirFunction
- [ ] Update AST → HIR lowering for test functions
- [ ] Verify MIR handles test functions correctly

### Week 2 (2026-01-18 ~ 2026-01-24)

**Day 6-7**: Test Runtime
- [ ] Create zulon-test-runtime crate
- [ ] Implement test discovery
- [ ] Implement test execution
- [ ] Implement result reporting

**Day 8-9**: YAN Integration
- [ ] Add yan test command
- [ ] Integrate with test runtime
- [ ] Implement test filtering
- [ ] Add verbose output

**Day 10**: Polish and Testing
- [ ] Write comprehensive tests
- [ ] Create example test files
- [ ] Write documentation
- [ ] Integration testing

---

## 10. Testing Strategy

### 10.1 Unit Tests

**Parser Tests**:
```rust
#[test]
fn test_parse_simple_test_attribute() {
    let input = "#[test] fn foo() { }";
    // Should parse correctly
}

#[test]
fn test_parse_assert_macro() {
    let input = "assert!(x > 0);";
    // Should expand to if statement
}
```

**HIR Tests**:
```rust
#[test]
fn test_hir_function_is_test_flag() {
    // Verify is_test field is set correctly
}
```

### 10.2 Integration Tests

**End-to-End Test**:
1. Create `.zl` file with tests
2. Run `yan test`
3. Verify correct output
4. Verify exit codes

**Example Test Suite**:
```bash
$ cd examples
$ yan test test_example.zl

running 5 tests
test test_add_integers ... ok
test test_add_negative ... ok
...

test result: ok. 5 passed; 0 failed
```

### 10.3 Test Scenarios

**Passing Test**:
```zulon
#[test]
fn test_passing() {
    assert_eq!(2 + 2, 4);
}
```
Expected: ✅ Pass

**Failing Test**:
```zulon
#[test]
fn test_failing() {
    assert_eq!(2 + 2, 5);  // Wrong!
}
```
Expected: ❌ Fail with message

**Panicking Test**:
```zulon
#[test]
fn test_panic() {
    panic!("Intentional panic");
}
```
Expected: ❌ Fail with panic message

---

## 11. Documentation

### 11.1 User Guide

**Section**: `docs/TESTING_GUIDE.md`

```markdown
# Testing in ZULON

## Writing Tests

Mark test functions with the `#[test]` attribute:

\`\`\`zulon
#[test]
fn test_feature() {
    assert!(condition);
}
\`\`\`

## Running Tests

Run all tests:
\`\`\`bash
yan test
\`\`\`

Run specific tests:
\`\`\`bash
yan test --filter "test_addition"
\`\`\`

## Assertions

- \`assert!(condition)\` - Assert condition is true
- \`assert_eq!(left, right)\` - Assert equality
- \`assert_ne!(left, right)\` - Assert inequality
```

### 11.2 API Documentation

**Document**:
- `#[test]` attribute behavior
- Assertion macro signatures
- Test output format
- Exit codes

---

## 12. Future Enhancements (Phase 2.7)

### 12.1 Advanced Features

**Parameterized Tests**:
```zulon
#[test]
#[data(1, 2, 3)]
#[data(10, 20, 30)]
fn test_add(a: i32, b: i32, expected: i32) {
    assert_eq!(a + b, expected);
}
```

**Async Tests**:
```zulon
#[test]
async fn test_async_operation() {
    let result = async_fetch().await;
    assert!(result.is_ok());
}
```

**Timeout Support**:
```zulon
#[test]
#[timeout(1000)]  // 1 second
fn test_slow_operation() {
    // ...
}
```

**Test Isolation**:
- Each test runs in isolated environment
- Resources cleaned up after test
- No state leakage between tests

### 12.2 Coverage

```bash
$ yan test --coverage

Filename: src/lib.zl
Lines: 85% (120/140)
Functions: 90% (18/20)
Branches: 75% (30/40)
```

---

## 13. Success Criteria

Phase 1.8 is complete when:

1. ✅ Parser recognizes `#[test]` attribute
2. ✅ Test functions compile correctly
3. ✅ Assertion macros expand and execute
4. ✅ `yan test` command runs tests
5. ✅ Test output shows pass/fail status
6. ✅ Exit code reflects test results
7. ✅ At least 10 example tests written
8. ✅ Documentation complete

---

## 14. Open Questions

### Q1: Should we support `#[should_panic]`?

**Proposal**: Yes, but defer to Phase 2.7

```zulon
#[test]
#[should_panic]
fn test_panics() {
    panic!();
}
```

### Q2: How to handle test setup/teardown?

**Proposal**: Defer to Phase 2.7

Potential syntax:
```zulon
#[test]
#[setup(setup_function)]
#[teardown(teardown_function)]
fn test_with_setup() {
    // ...
}
```

### Q3: Should we support ignored tests?

**Proposal**: Yes, simple version

```zulon
#[test]
#[ignore]
fn test_not_ready() {
    // This test won't run
}
```

Run with: `yan test --include-ignored`

---

## 15. References

- Rust Testing Guide: https://doc.rust-lang.org/book/ch11-00-testing.html
- Cargo Test: https://doc.rust-lang.org/cargo/commands/cargo-test.html
- LLVM Metadata: https://llvm.org/docs/LangRef.html#metadata

---

**Document Version**: 1.0
**Status**: Ready for Implementation
**Next Step**: Begin Parser Extensions (Day 1-2)
