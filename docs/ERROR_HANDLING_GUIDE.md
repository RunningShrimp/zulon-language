# ZULON Error Handling Guide

**Last Updated**: 2026-01-10
**ZULON Version**: 0.1.0 (MVP)
**Status**: ✅ Error Handling 90% Complete

---

## Table of Contents

1. [Overview](#overview)
2. [Basic Error Handling](#basic-error-handling)
3. [The ? Operator](#the--operator)
4. [Best Practices](#best-practices)
5. [Common Patterns](#common-patterns)
6. [Known Limitations](#known-limitations)
7. [Examples](#examples)

---

## Overview

ZULON provides ergonomic error handling through:
- **Error types**: Define custom error types using enums
- **Throwing**: Return errors using `throw` keyword
- **Propagation**: Use `?` operator to automatically propagate errors
- **Union types**: Functions return `T | E` to indicate possible errors

### The Outcome Type

Error handling in ZULON is implemented using the `Outcome<T, E>` type, which is represented as a struct with:
- **discriminant**: `i32` value (0 = Ok, 1 = Err)
- **data**: The success value (T) or error value (E)

```rust
// Internally represented as:
struct Outcome<T, E> {
    discriminant: i32,  // 0 = Ok, 1 = Err
    data: /* T or E */,
}
```

---

## Basic Error Handling

### Defining Error Types

Error types are defined as enums:

```zig
enum MathError {
    Zero,        // Division by zero
    Negative,    // Negative input
    Overflow,    // Numeric overflow
}
```

### Throwing Errors

Use the `throw` keyword to return an error:

```zig
fn divide(a: i32, b: i32) -> i32 | MathError {
    if b == 0 {
        throw MathError::Zero;  // Returns error
    }
    if a < 0 || b < 0 {
        throw MathError::Negative;
    }
    a / b  // Returns success value
}
```

**Important**: The last expression in a function is automatically returned. No need for explicit `return` in most cases.

### Handling Errors

When calling a function that can fail, you get an `Outcome` value:

```zig
fn main() -> i32 {
    let result = divide(10, 2);  // Type: i32 | MathError

    // Store the Outcome for later inspection
    // (In the future, you'll use match expressions to handle this)
    0
}
```

---

## The ? Operator

The `?` operator provides ergonomic error propagation.

### Basic Usage

```zig
fn compute(x: i32) -> i32 | MathError {
    let result = divide(x, 2)?;  // Automatically returns error if divide fails
    result + 1
}
```

### How It Works

When you use `?`:
1. It checks the discriminant of the `Outcome`
2. If discriminant is 0 (Ok), it extracts the value and continues
3. If discriminant is 1 (Err), it returns the error immediately

**Generated Code** (simplified):
```zig
fn compute(x: i32) -> i32 | MathError {
    let outcome = divide(x, 2);

    // Check discriminant
    if outcome.discriminant == 0 {
        // Success path
        let result = outcome.data;
        result + 1
    } else {
        // Error path - return immediately
        outcome
    }
}
```

### Chaining ? Operators

```zig
fn multi_step(x: i32) -> i32 | MathError {
    let a = validate(x)?;
    let b = process(a)?;
    let c = transform(b)?;
    c + 1
}
```

Each `?` can potentially return early if an error occurs.

---

## Best Practices

### 1. Use Descriptive Error Types

✅ **Good**:
```zig
enum FileError {
    NotFound,
    PermissionDenied,
    Corrupted,
}
```

❌ **Avoid**:
```zig
// Too generic - doesn't help users understand what went wrong
enum Error {
    Fail,
}
```

### 2. Handle Errors at the Right Level

```zig
// Low-level function: just report errors
fn read_config(path: str) -> Config | IoError {
    // ...
}

// High-level function: provide context
fn load_app_config() -> Config | AppError {
    let cfg = read_config("config.zl")?;
    // Can add more specific error handling here
    cfg
}
```

### 3. Use ? for Propagation

✅ **Preferred**:
```zig
fn process(x: i32) -> i32 | MathError {
    let y = validate(x)?;
    calculate(y)
}
```

❌ **Verbose** (not recommended):
```zig
fn process(x: i32) -> i32 | MathError {
    let outcome = validate(x);
    // Manual checking (verbose - use ? instead)
    if outcome.discriminant == 0 {
        let y = outcome.data;
        calculate(y)
    } else {
        outcome
    }
}
```

### 4. Avoid Nested Function Calls as Last Expression

There's a known bug with tail call optimization. Use this pattern:

✅ **Works**:
```zig
fn middle() -> i32 | MathError {
    let result = inner();  // Assign first
    result  // Then return
}
```

⚠️ **Known Bug** (avoid for now):
```zig
fn middle() -> i32 | MathError {
    inner()  // TODO: This has a codegen bug
}
```

---

## Common Patterns

### Pattern 1: Validation Chain

```zig
fn validate_and_process(x: i32) -> i32 | ValidationError {
    // Step 1: Validate range
    let checked = check_range(x)?;

    // Step 2: Validate constraints
    let constrained = apply_constraints(checked)?;

    // Step 3: Process
    transform(constrained)
}
```

### Pattern 2: Default Values

```zig
fn safe_divide(a: i32, b: i32) -> i32 {
    let result = divide(a, b);

    // For now, store the result
    // TODO: Use match when implemented to handle errors properly
    0  // Placeholder
}
```

### Pattern 3: Error Conversion

```zig
enum AppError {
    Io(IoError),
    Parse(ParseError),
    Math(MathError),
}

fn read_and_parse(path: str) -> Data | AppError {
    // TODO: Implement error conversion when match is ready
    // For now, direct propagation works with ?
    let content = read_file(path)?;
    parse_data(content)?
}
```

---

## Known Limitations

### 1. Match Expressions Not Ready ⚠️

**Status**: Frontend complete, backend missing

You cannot yet elegantly handle `Outcome` values:

```zig
// This syntax parses but doesn't work yet:
fn handle_result() -> i32 {
    let result = divide(10, 2);

    match result {
        Ok(value) => value,
        Err(err) => 0,
    }
}
```

**Workaround**: For now, store `Outcome` values and inspect them manually or use `?` to propagate.

### 2. Tail Call Optimization Bug ⚠️

**Status**: Documented, not fixed

Functions that just return another function's result may have issues:

```zig
fn middle() -> i32 | MathError {
    inner()  // Can double-wrap Outcome
}
```

**Workaround**: Assign to a local variable first:
```zig
fn middle() -> i32 | MathError {
    let result = inner();
    result
}
```

### 3. Error Type Conversion ⚠️

**Status**: Not implemented

Converting between different error types requires manual work:

```zig
fn convert_errors() -> i32 | AppError {
    // This doesn't work yet:
    // let x = divide(10, 2)?;  // Returns MathError
    // Need manual conversion when match is ready
    0
}
```

---

## Examples

### Example 1: Simple Division

```zig
enum MathError {
    Zero,
}

fn divide(a: i32, b: i32) -> i32 | MathError {
    if b == 0 {
        throw MathError::Zero;
    }
    a / b
}

fn main() -> i32 {
    let result1 = divide(10, 2);   // Success
    let result2 = divide(10, 0);   // Would be error
    0
}
```

### Example 2: Multi-Step Validation

```zig
enum ValidationError {
    Empty,
    TooShort,
    InvalidChar,
}

fn check_empty(s: str) -> str | ValidationError {
    if s.len() == 0 {
        throw ValidationError::Empty;
    }
    s
}

fn check_length(s: str) -> str | ValidationError {
    if s.len() < 3 {
        throw ValidationError::TooShort;
    }
    s
}

fn validate_username(name: str) -> str | ValidationError {
    let non_empty = check_empty(name)?;
    let long_enough = check_length(non_empty)?;
    long_enough
}

fn main() -> i32 {
    let user1 = validate_username("alice");  // Success
    let user2 = validate_username("");      // Would be error
    0
}
```

### Example 3: Nested Calculations

```zig
enum CalcError {
    Zero,
    Negative,
}

fn square_root(x: i32) -> i32 | CalcError {
    if x < 0 {
        throw CalcError::Negative;
    }
    // Simplified - just return x
    x
}

fn divide_by_two(x: i32) -> i32 | CalcError {
    if x == 0 {
        throw CalcError::Zero;
    }
    x / 2
}

fn complex_calc(x: i32) -> i32 | CalcError {
    let half = divide_by_two(x)?;
    let root = square_root(half)?;
    root + 1
}

fn main() -> i32 {
    let result1 = complex_calc(100);  // Success chain
    let result2 = complex_calc(-5);   // Would error at square_root
    0
}
```

---

## Current Status

### ✅ Implemented (Working)

- [x] Error type definition (enums)
- [x] Throwing errors (`throw` keyword)
- [x] Union return types (`T | E`)
- [x] ? operator for error propagation
- [x] Multiple error types
- [x] Conditional throwing
- [x] End-to-end compilation

### ⚠️ Partially Implemented

- [ ] Match expressions (frontend done, backend missing)
- [ ] Error type conversion
- [ ] Comprehensive error messages

### ❌ Not Implemented

- [ ] Try-catch blocks
- [ ] Error trait/protocol
- [ ] Stack trace capture
- [ ] Error context preservation

---

## Testing Your Error Handling

### Compile Test

```bash
# Compile your ZULON program
cargo run --example test_error_compile -- path/to/your/file.zl

# This generates LLVM IR
```

### Run Test

```bash
# The compiler outputs LLVM IR which can be assembled and linked
# See examples/ directory for complete build scripts
```

---

## Next Steps

As ZULON matures, the error handling will improve with:

1. **Match Expressions** - Allow elegant error handling
2. **Error Traits** - Standard error interface
3. **Better Error Messages** - Contextual information
4. **Stack Traces** - Debugging support
5. **Error Composition** - Combining errors from different sources

---

## Resources

- **Implementation**: See `RALPH_LOOP_ITERATION_14_COMPLETE.md` for technical details
- **Test Suite**: `ERROR_HANDLING_TEST_SUITE.md` for comprehensive tests
- **Status**: `RALPH_LOOP_ITERATIONS_7-12_SUMMARY.md` for progress tracking

---

**Happy Coding!** 🎉

ZULON's error handling system is designed to be both ergonomic and safe. The `?` operator makes error propagation painless, while strong typing ensures errors are handled explicitly.

*Generated: 2026-01-10*
*Status: Error Handling 90% Complete* ✅
