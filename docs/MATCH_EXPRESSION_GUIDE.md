# Match Expression Guide

Complete guide to match expressions in ZULON, including current capabilities, limitations, and practical examples.

## Overview

Match expressions in ZULON provide pattern matching functionality similar to Rust's `match` or Rust's `match` expressions. They allow you to compare a value against multiple patterns and execute different code based on which pattern matches.

**Current Status**: Match expressions are **implemented and functional** for basic patterns (integers, wildcards, multiple arms). Enum destructuring patterns are **not yet supported**.

## What Works

ZULON match expressions currently support:

### 1. Simple Integer Matching

```zig
fn test_integer_match(x: i32) -> i32 {
    match x {
        0 => 10,
        1 => 20,
        2 => 30,
        _ => 40,
    }
}
```

**Generated LLVM**: Uses `switch` instruction for efficient jump table implementation.

### 2. Wildcard Pattern

```zig
fn test_wildcard(x: i32) -> i32 {
    match x {
        5 => 100,
        _ => 999,
    }
}
```

The `_` pattern matches anything, similar to Rust's wildcard pattern.

### 3. Multiple Arms

```zig
fn test_multiple_arms(x: i32) -> i32 {
    match x {
        1 => 10,
        2 => 20,
        3 => 30,
        4 => 40,
        5 => 50,
        _ => 0,
    }
}
```

Match expressions support any number of arms.

### 4. Complex Expressions in Arms

```zig
fn test_complex_expressions(x: i32) -> i32 {
    match x {
        1 => 2 * 5,
        2 => 10 + 20,
        3 => 100 / 2,
        _ => 0,
    }
}
```

Each arm can contain any expression, including arithmetic operations.

### 5. Match with Computation Before

```zig
fn test_match_with_compute(a: i32, b: i32) -> i32 {
    let sum = a + b;
    match sum {
        0 => 100,
        10 => 200,
        20 => 300,
        _ => 400,
    }
}
```

You can perform computations before the match expression.

### 6. Nested Match

```zig
fn test_nested_match(x: i32) -> i32 {
    match x {
        1 => match x {
            1 => 100,
            _ => 101,
        },
        2 => 200,
        _ => 300,
    }
}
```

Match expressions can be nested within other match arms.

### 7. Match with Function Call in Value

```zig
fn add_five(x: i32) -> i32 {
    x + 5
}

fn test_match_with_function(x: i32) -> i32 {
    match add_five(x) {
        5 => 100,
        6 => 200,
        _ => 300,
    }
}
```

The value being matched can be the result of a function call.

### 8. Match with Block Expressions

```zig
fn test_match_block(x: i32) -> i32 {
    match x {
        1 => {
            let a = 10;
            let b = 20;
            a + b
        },
        2 => {
            let c = 5;
            c * 2
        },
        _ => 0,
    }
}
```

Each arm can contain a block with multiple statements.

## What Doesn't Work (Yet)

### Enum Destructuring

```zig
enum Option {
    Some(i32),
    None,
}

fn test_option(opt: Option) -> i32 {
    match opt {
        Option::Some(x) => x,  // ❌ NOT SUPPORTED
        Option::None => 0,
    }
}
```

**Error**: `UnsupportedFeature { feature: "pattern: Struct(...)" }`

**Impact**: Cannot match on enum variants with field binding.

**Workaround**: Use discriminant checking and manual field access:

```zig
fn test_option_workaround(opt: Option) -> i32 {
    if opt.discriminant() == 0 {
        // It's Some, extract field manually
        opt.field_0()
    } else {
        0
    }
}
```

Note: This workaround requires implementing `discriminant()` and field access methods for enums, which may not be available yet.

## Practical Examples

### Example 1: State Machine

```zig
fn handle_state(state: i32) -> i32 {
    match state {
        0 => {
            // Initialize
            1
        },
        1 => {
            // Process
            2
        },
        2 => {
            // Cleanup
            0
        },
        _ => {
            // Error state
            -1
        },
    }
}

fn main() -> i32 {
    let state = 0;
    let state = handle_state(state);
    let state = handle_state(state);
    let state = handle_state(state);
    state  // Should be 0
}
```

### Example 2: Command Dispatch

```zig
fn execute_command(cmd: i32, arg: i32) -> i32 {
    let result = match cmd {
        0 => arg * 2,
        1 => arg + 10,
        2 => arg - 5,
        3 => arg / 2,
        _ => 0,
    };
    result
}

fn main() -> i32 {
    execute_command(1, 5)  // Returns 15
}
```

### Example 3: Error Handling (Without Enum Destructuring)

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

fn handle_division(x: i32) -> i32 {
    let result = divide(100, x);

    // Can't match on Outcome enum yet
    // Use conditional check instead
    let discriminant = result.discriminant();
    match discriminant {
        0 => {
            // Success case - extract value
            result.value_field()
        },
        1 => {
            // Error case
            -1
        },
        _ => -2,
    }
}

fn main() -> i32 {
    handle_division(5)  // Returns 20
}
```

**Note**: The `discriminant()` and `value_field()` methods are illustrative - actual implementation depends on available enum methods.

## Compiler Implementation Details

### LLVM IR Generation

Match expressions are compiled to LLVM's `switch` instruction:

```llvm
define i32 @test_match(i32 %x) {
  switch i32 %x, label %default [
    i32 0, label %case0
    i32 1, label %case1
    i32 2, label %case2
  ]

case0:
  ret i32 10

case1:
  ret i32 20

case2:
  ret i32 30

default:
  ret i32 40
}
```

This generates efficient jump tables for fast pattern matching.

### Compilation Pipeline

1. **Parsing**: Parse `match` expression syntax into AST
2. **HIR**: Lower AST to HIR, validate patterns
3. **MIR**: Lower HIR to MIR, generate control flow
4. **LIR**: Lower MIR to LIR, allocate registers
5. **LLVM IR**: Generate `switch` instruction
6. **Assembly**: LLVM generates machine code

### Current Limitations

1. **Pattern Types**: Only integer literals and wildcards supported
2. **Enum Destructuring**: Not implemented (requires pattern matching on enum variants)
3. **Struct Patterns**: Not implemented
4. **Range Patterns**: Not implemented (e.g., `1..=10`)
5. **Guards**: Not implemented (e.g., `x if x > 0 => ...`)
6. **Or Patterns**: Not implemented (e.g., `1 | 2 => ...`)

## Future Enhancements

### Priority 1: Enum Destructuring

**Value**: High - enables elegant error handling

**Example**:
```zig
fn handle_result(result: i32 | MathError) -> i32 {
    match result {
        Ok(v) => v,
        Err(e) => 0,
    }
}
```

**Complexity**: High (4-6 hours estimated)

**Requires**:
- Pattern matching in HIR lowering
- Enum variant representation
- Field binding and extraction
- Discriminant checking

### Priority 2: Range Patterns

**Value**: Medium - useful for validation

**Example**:
```zig
match x {
    0..=10 => "small",
    11..=100 => "medium",
    _ => "large",
}
```

**Complexity**: Medium

### Priority 3: Or Patterns

**Value**: Low - can use multiple arms

**Example**:
```zig
match x {
    1 | 3 | 5 => "odd",
    2 | 4 | 6 => "even",
    _ => "other",
}
```

**Complexity**: Low

### Priority 4: Guards

**Value**: Medium - expressive patterns

**Example**:
```zig
match x {
    n if n > 0 => "positive",
    n if n < 0 => "negative",
    _ => "zero",
}
```

**Complexity**: High

## Testing Match Expressions

### Test File: Comprehensive Test

Create a test file to verify all match features:

```zig
// comprehensive_match_test.zl

fn test_integer(x: i32) -> i32 {
    match x {
        0 => 10,
        1 => 20,
        _ => 30,
    }
}

fn test_wildcard(x: i32) -> i32 {
    match x {
        5 => 100,
        _ => 999,
    }
}

fn test_multiple(x: i32) -> i32 {
    match x {
        1 => 10,
        2 => 20,
        3 => 30,
        _ => 0,
    }
}

fn test_nested(x: i32) -> i32 {
    match x {
        1 => match x {
            1 => 100,
            _ => 101,
        },
        _ => 300,
    }
}

fn main() -> i32 {
    let t1 = test_integer(1);    // 20
    let t2 = test_wildcard(10);  // 999
    let t3 = test_multiple(3);   // 30
    let t4 = test_nested(1);     // 100

    t1 + t2 + t3 + t4  // 1149
}
```

**Compile and Test**:
```bash
./scripts/quick_build.sh comprehensive_match_test.zl
# Expected exit code: 1149
```

## Best Practices

### 1. Always Include a Wildcard Arm

```zig
// Good
match x {
    1 => 10,
    2 => 20,
    _ => 0,  // Default case
}

// Bad - may fail to compile or panic at runtime
match x {
    1 => 10,
    2 => 20,
}
```

### 2. Use Match for Readability

```zig
// Clear and readable
fn classify(x: i32) -> i32 {
    match x {
        0 => 1,
        1 => 2,
        _ => 3,
    }
}

// Less readable
fn classify(x: i32) -> i32 {
    if x == 0 {
        1
    } else {
        if x == 1 {
            2
        } else {
            3
        }
    }
}
```

### 3. Use Block Expressions for Complex Arms

```zig
fn process(x: i32) -> i32 {
    match x {
        1 => {
            let temp = x * 2;
            temp + 10
        },
        2 => {
            let temp = x + 5;
            temp * 3
        },
        _ => 0,
    }
}
```

## Troubleshooting

### Issue: "UnsupportedFeature: pattern: Struct(...)"

**Cause**: Trying to use enum destructuring patterns

**Solution**: Use integer matching on discriminants instead

**Example**:
```zig
// Instead of:
match result {
    Ok(v) => v,
    Err(e) => 0,
}

// Use (if discriminant methods available):
match result.discriminant() {
    0 => result.value(),
    1 => 0,
    _ => -1,
}
```

### Issue: Match Not Generating Switch Instruction

**Cause**: Pattern too complex or not supported

**Solution**: Use simpler integer patterns

**Example**:
```zig
// Works - generates switch
match x {
    1 => 10,
    2 => 20,
    _ => 30,
}

// May not work - enum destructuring
match opt {
    Some(x) => x,
    None => 0,
}
```

### Issue: Type Mismatch in Match Arms

**Cause**: Arms return different types

**Solution**: Ensure all arms return the same type

**Example**:
```zig
// Bad - mixed types
match x {
    1 => 10,
    2 => "hello",  // Type mismatch!
    _ => 0,
}

// Good - all i32
match x {
    1 => 10,
    2 => 20,
    _ => 0,
}
```

## Performance Considerations

### Switch Instruction Efficiency

Match expressions compile to LLVM's `switch` instruction, which:
- Generates efficient jump tables for dense integer patterns
- Uses binary search for sparse patterns
- Has O(1) or O(log n) lookup time
- Is generally faster than equivalent if-else chains

### Optimization Tips

1. **Use dense integer values** when possible:
```zig
// Good - dense values (0, 1, 2)
match x {
    0 => ...,
    1 => ...,
    2 => ...,
    _ => ...,
}

// Less optimal - sparse values (10, 100, 1000)
match x {
    10 => ...,
    100 => ...,
    1000 => ...,
    _ => ...,
}
```

2. **Put common cases first** (may help with branch prediction):
```zig
match x {
    1 => ...,  // Most common
    2 => ...,  // Second most common
    _ => ...,  // Least common
}
```

## Related Features

- **If-Else Expressions**: Alternative to match for simple cases
- **Union Types**: `T | E` for error handling
- **Throw Statement**: For returning errors
- **? Operator**: For automatic error propagation
- **Enums**: For defining custom data types (destructuring not yet supported)

## Further Reading

- **Error Handling Guide**: `docs/ERROR_HANDLING_GUIDE.md`
- **Compilation Guide**: `docs/COMPILATION_GUIDE.md`
- **Implementation Plan**: `IMPLEMENTATION_PLAN.md`
- **TODOLIST**: `TODOLIST.md`

## Summary

Match expressions in ZULON are **functional and efficient** for basic pattern matching on integers. They compile to LLVM `switch` instructions and support multiple arms, wildcards, nested matches, and complex expressions.

The main limitation is **enum destructuring**, which prevents elegant error handling patterns like `match result { Ok(v) => v, Err(e) => 0 }`. This is a known limitation that can be worked around using discriminant checking (when available).

**Status**: Match expressions are **production-ready** for integer-based pattern matching. Enum destructuring is planned for future enhancement.

**Last Updated**: Iteration 17 - Ralph Loop Cycle
