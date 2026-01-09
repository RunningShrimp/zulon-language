# ZULON Compiler - Known Bugs and Limitations

This document tracks known bugs and limitations discovered during testing and development.

**Last Updated**: January 9, 2026
**Compiler Version**: 0.1.0

## Critical Bugs

None currently classified as critical.

## Major Bugs

### 1. Type Checker Doesn't Catch All Type Mismatches

**Severity**: Major
**Status**: Known limitation
**Component**: Type Checker

**Description**:
The type checker doesn't catch all type mismatches in function arguments and match arms.

**Example**:
```zulon
fn greet(name: string) -> i32 {
    0
}

fn main() -> i32 {
    greet(42)  // Should error but doesn't in some cases
}
```

**Impact**: Users may get confusing LLVM errors instead of clear type errors.

**Workaround**: None currently. Type annotations help in some cases.

**Estimated Fix**: 2-3 iterations - Enhance type checker to validate all function call arguments

---

## Minor Bugs

### 2. Block Expressions in Let Statements Don't Parse

**Severity**: Minor
**Status**: Parser limitation
**Component**: Parser

**Description**:
Block expressions (`{ ... }`) cannot be used in let statement assignments.

**Example**:
```zulon
// ❌ Doesn't parse
let result = {
    let x = 10;
    let y = 20;
    x + y
};

// ✅ Workaround
let x = 10;
let y = 20;
let result = x + y;
```

**Error**: `unexpected token in expression: Some(LeftBrace)`

**Impact**: Cannot use block expressions for variable initialization.

**Workaround**: Use separate statements instead of block expressions.

**Estimated Fix**: 1-2 iterations - Update parser to handle block expressions in more contexts

---

### 3. Bool Type Annotations Don't Parse

**Severity**: Minor
**Status**: Parser limitation
**Component**: Parser

**Description**:
Type annotations using `bool` don't parse. Parser expects type annotations to be paths.

**Example**:
```zulon
// ❌ Doesn't parse
fn example() -> bool {
    true
}

let x: bool = true;

// ✅ Workaround - rely on type inference
fn example() -> i32 {
    if true {
        1
    } else {
        0
    }
}

let x = true;
```

**Error**: `expected type, found Some(Bool)`

**Impact**: Cannot use explicit `bool` type annotations.

**Workaround**: Use type inference or `i32` for boolean-like values.

**Estimated Fix**: 1 iteration - Add built-in types to type annotation parser

---

### 4. Enum Variant Construction Fails in Some Contexts

**Severity**: Minor
**Status**: Type checker limitation
**Component**: Type Checker

**Description**:
Enum variant construction (`Option::Some(42)`) is treated as non-callable in some contexts.

**Example**:
```zulon
enum Option {
    Some(i32),
    None,
}

fn main() -> i32 {
    let value = Option::Some(42);  // May fail
    match value {
        Option::Some(v) => v,
        Option::None => 0
    }
}
```

**Error**: `cannot call non-function type` for the enum

**Impact**: Cannot use enum variants in all contexts.

**Workaround**: Use pattern matching on directly constructed values or use simpler patterns.

**Estimated Fix**: 2-3 iterations - Enhance type checker to handle enum variant construction

---

### 5. Template String Return Type Causes Inference Error

**Severity**: Minor
**Status**: Type inference limitation
**Component**: Type Checker

**Description**:
Functions with explicit `string` return type and template string body fail type checking.

**Example**:
```zulon
// ❌ Type checker error
fn get_greeting() -> string {
    `Hello, World!`
}

// ✅ Workaround
fn get_greeting() -> i32 {
    println(`Hello, World!`);
    0
}
```

**Error**: Type mismatch between inferred type and return type

**Impact**: Cannot return template strings from functions with explicit return types.

**Workaround**: Use implicit return types or print strings instead of returning them.

**Estimated Fix**: 1 iteration - Fix type inference for template strings

---

### 6. Match Arm Type Mismatches Not Caught

**Severity**: Minor
**Status**: Type checker limitation
**Component**: Type Checker

**Description**:
Type checker doesn't always catch type mismatches between match arms.

**Example**:
```zulon
let result = match x {
    1 => 10,
    2 => `string`,  // Should error but may not
    _ => 0
};
```

**Impact**: May get LLVM errors instead of clear type errors.

**Workaround**: Ensure all match arms have compatible types.

**Estimated Fix**: 1 iteration - Enhance match expression type checking

---

## Limitations (Not Bugs)

These are known limitations, not bugs, documented for clarity:

### 1. Template String Interpolation Not Implemented

**Status**: By Design (not yet implemented)
**Complexity**: High (4-5 iterations estimated)

**Description**:
Template strings only support static content. Interpolation like `` `Hello, ${name}` `` is not yet implemented.

**Workaround**: Use only static strings in template literals.

---

### 2. Defer Statements Have Limited Early Return Handling

**Status**: By Design (85% complete)
**Complexity**: Medium (2-3 iterations estimated)

**Description**:
Defer statements only execute at normal block exit, not on early returns or error paths.

**Workaround**: Manually call cleanup before returns if needed.

---

### 3. No For Loops

**Status**: By Design (not yet implemented)
**Complexity**: Medium (2-3 iterations estimated)

**Description**:
Only `while` loops are implemented. `for` loops and `loop` constructs are not available.

**Workaround**: Use `while` loops with manual index management.

---

### 4. Generic Type Instantiation Incomplete

**Status**: By Design (partial implementation)
**Complexity**: High (5-6 iterations estimated)

**Description**:
Parser supports generic type syntax (`Option<T>`), but type checker cannot fully instantiate generic types.

**Workaround**: Use concrete types or monomorphize manually.

---

### 5. Tuple Construction Has MIR Limitations

**Status**: By Design (90% complete)
**Complexity**: High (4-6 iterations estimated - architectural changes needed)

**Description**:
Multi-element tuple construction in MIR only returns the first element due to architectural limitations.

**Workaround**: Use individual variables or struct when available.

---

### 6. Type Inference Limited

**Status**: By Design (partial implementation)
**Complexity**: Medium (2-3 iterations estimated)

**Description**:
Type inference works for simple cases but not complex expressions or cross-function scenarios.

**Workaround**: Use explicit type annotations when inference fails.

---

## Parser Limitations

### 1. Defer in While Loop Bodies

**Description**: Defer statements in while loop bodies have parser constraints due to semicolon requirements.

**Workaround**: Avoid defer in while loops or use separate functions.

---

### 2. Match Arms Require Commas

**Description**: Match arms with block expressions require commas after them (except the last arm).

**Example**:
```zulon
match x {
    1 => {
        println("one");
        1
    },  // ← Comma required
    2 => {
        println("two");
        2
    }   // ← No comma on last arm
}
```

**Workaround**: Remember to add commas after match arms (except last).

---

### 3. Defer Statements Cannot Have Semicolons

**Description**: Defer statements must NOT end with semicolons, unlike other statements.

**Example**:
```zulon
// ✅ Correct
defer println("cleanup")

// ❌ Wrong
defer println("cleanup");
```

**Workaround**: Remember not to add semicolons after defer statements.

---

## Testing Gaps

The following areas have limited test coverage:

1. **Error Recovery**: Limited testing of error recovery mechanisms
2. **Performance**: No performance or regression tests
3. **Stress Testing**: No tests with very large programs or deeply nested structures
4. **Edge Cases**: Some edge cases in type system not fully tested

---

## Prioritization for Future Work

### High Priority (Next Phase)

1. Fix block expression parsing (1-2 iterations)
2. Fix bool type annotations (1 iteration)
3. Enhance type checker for function calls (2-3 iterations)
4. Improve match expression type checking (1 iteration)

### Medium Priority (Phase 2.2+)

1. Template string interpolation (4-5 iterations)
2. Enhanced defer with early return handling (2-3 iterations)
3. Enum variant construction fixes (2-3 iterations)

### Low Priority (Phase 3+)

1. For loops (2-3 iterations)
2. Generic type instantiation (5-6 iterations)
3. Tuple MIR architectural fixes (4-6 iterations)

---

## Reporting Bugs

Found a bug? Please report it with:

1. **Minimal reproduction code**
2. **Expected behavior**
3. **Actual behavior/error message**
4. **Compiler version**: `cargo run -p zulon-compiler -- --version`
5. **Platform**: OS and architecture

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on bug reports.

---

**Note**: This document is updated as bugs are discovered and fixed. Last comprehensive review: Iteration 37
