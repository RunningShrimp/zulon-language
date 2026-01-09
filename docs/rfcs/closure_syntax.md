# ZULON Closure Syntax - RFC (Request for Comments)

**RFC Number**: 0001
**Status**: Draft
**Date**: 2026-01-08
**Author**: ZULON Language Team
**Target Version**: 0.2.0-Alpha

---

## Executive Summary

This RFC proposes closure syntax for ZULON, inspired by Rust's closure design but simplified for consistency. Closures (lambda functions) are anonymous functions that can capture variables from their surrounding environment, enabling functional programming patterns like higher-order functions, callbacks, and iterators.

---

## Motivation

### Why Closures?

1. **Functional Programming**: Enable map, filter, fold, and other higher-order functions
2. **Concise Code**: Eliminate boilerplate for small callback functions
3. **Context Capture**: Allow functions to carry their environment with them
4. **Modern Language Expectation**: Closures are standard in modern languages (Rust, Swift, C++, JavaScript)

### Use Cases

```zulon
// Higher-order functions
vec.map(|x| x * 2)

// Event handlers
button.on_click(|event| handle_click(event))

// Thread spawning
thread::spawn(|| {
    println("Hello from thread")
})

// Iterators
(0..10).filter(|x| x % 2 == 0)
```

---

## Syntax Constraints

### Empty Closure Restriction ⚠️

**Important**: Empty closures `|| expr` are **not supported** due to ambiguity with the logical OR operator `||`.

**Why**: The lexer tokenizes `||` as `TokenKind::Or`, making it impossible to distinguish from logical OR in the parser.

**Workarounds**:
```zulon
// Option 1: Use block body (RECOMMENDED)
let get_zero = || { 0 };

// Option 2: Use a dummy parameter (discouraged)
let get_zero = |_dummy| 0;

// Option 3: Define a named function instead
fn get_zero() -> i32 { 0 }
```

**Rationale**: This is a known trade-off for keeping the lexer simple and maintaining compatibility with logical OR. Other languages (e.g., Rust) have similar restrictions in certain contexts.

**Future**: Phase 3 may introduce more sophisticated disambiguation using context-sensitive lexical analysis.

---

## Syntax Proposals

### Option A: Rust-Style Closure (RECOMMENDED) ✅

**Syntax**:
```zulon
// Basic closure
let add = |x, y| x + y;

// Closure with explicit types
let add = |x: i32, y: i32| -> i32 { x + y };

// Closure with block body
let complex = |x| {
    let y = x * 2;
    y + 10
};

// Closure capturing environment
let multiplier = 10;
let multiply = |x| x * multiplier;

// No-argument closure
let get_value = || 42;

// Immediate invocation
(|x, y| x + y)(10, 20)
```

**Pros**:
- ✅ Familiar to Rust developers
- ✅ Clean and concise
- ✅ Explicit when needed (types can be added)
- ✅ Consistent with function syntax

**Cons**:
- ⚠️ Pipe characters `|` may be unfamiliar to beginners
- ⚠️ Ambiguity with bitwise OR (though rare in practice)

---

### Option B: Simplified fn-style

**Syntax**:
```zulon
// Basic closure
let add = fn(x, y) { x + y };

// Closure with explicit types
let add = fn(x: i32, y: i32) -> i32 { x + y };

// Closure capturing environment
let multiplier = 10;
let multiply = fn(x) x * multiplier;
```

**Pros**:
- ✅ Familiar `fn` keyword
- ✅ No special characters
- ✅ Consistent with named functions

**Cons**:
- ❌ Ambiguous with named function syntax
- ❌ Harder to distinguish from functions at a glance
- ❌ Less visually distinct

---

### Option C: Arrow Syntax

**Syntax**:
```zulon
// Basic closure
let add = (x, y) => x + y;

// Closure with explicit types
let add = (x: i32, y: i32) => i32 { x + y };

// Closure capturing environment
let multiplier = 10;
let multiply = (x) => x * multiplier;
```

**Pros**:
- ✅ Familiar to JavaScript/C# developers
- ✅ Clear directionality (input → output)

**Cons**:
- ❌ Arrow `=>` conflicts with comparison operator `>=`
- ❌ Different from rest of ZULON syntax
- ❌ Less concise than pipe syntax

---

## Recommended Syntax: Option A (Rust-Style)

**Decision**: Use Rust-style pipe syntax with optional type annotations.

### Complete Grammar

```
closure_expression ::= '|' closure_params '|' closure_body
closure_params     ::= [ param (',' param)* ]
closure_body       ::= expression | block

param              ::= IDENTIFIER | IDENTIFIER ':' TYPE
```

### Examples

```zulon
// Example 1: Simple closure
let square = |x| x * x;

// Example 2: With type annotation
let square = |x: i32| -> i32 { x * x };

// Example 3: Multiple parameters
let add = |x, y| x + y;

// Example 4: Block body
let complex = |x| {
    let temp = x * 2;
    temp + 10
};

// Example 5: No parameters
let get_zero = || { 0 };  // Must use block body for empty closures

// Example 6: Capturing by reference
let value = 42;
let print_value = || { println(value) };

// Example 7: Capturing by mutable reference
let mut counter = 0;
let increment = || {
    counter = counter + 1;
    counter
};

// Example 8: Immediate invocation
let result = (|a, b| a + b)(10, 20);
```

---

## Type Inference Rules

### 1. Parameter Type Inference

Closures use **contextual type inference** for parameters:

```zulon
// Parameter types inferred from Vec::map signature
fn map<T, U>(vec: Vec<T>, f: |T| -> U) -> Vec<U>

// Usage
let doubled = vec.map(|x| x * 2);
// x inferred as i32 from vec type
```

**When to Annotate**:
- When context is ambiguous
- When you want explicit documentation
- When inference fails (rare)

### 2. Return Type Inference

Return type is always inferred from the body:

```zulon
// Return type inferred as i32
let add = |x, y| x + y;

// Return type inferred as bool
let is_positive = |x| x > 0;

// Block body: type of last expression
let complex = |x| {
    let y = x * 2;
    y + 10  // Return type: i32
};
```

### 3. Closure Type Representation

Closures have unique types, but implement trait objects:

```zulon
// Closure type: anonymous environment + function pointer
type Closure = struct {
    env: Environment,
    func: fn(Environment, Args) -> Return
}

// Traits for closures
trait Fn<Args> -> Return {
    fn call(&self, args: Args) -> Return;
}

trait FnMut<Args> -> Return {
    fn call_mut(&mut self, args: Args) -> Return;
}

trait FnOnce<Args> -> Return {
    fn call_once(self, args: Args) -> Return;
}
```

---

## Capture Semantics

### Capture Modes

1. **By Reference** (immutable borrow)
   - Closure reads captured variable
   - Variable must outlive closure
   - Multiple closures can capture same variable

2. **By Mutable Reference** (mutable borrow)
   - Closure can modify captured variable
   - Exclusive access required
   - Only one closure can capture mutably

3. **By Value** (move/Copy)
   - Closure owns captured value
   - Original variable invalidated
   - Happens for Copy types or explicit move

### Inference Rules

```zulon
// Rule 1: Default to immutable borrow
let x = 10;
let print = || println(x);  // &x

// Rule 2: Mutable borrow if modified
let mut x = 10;
let increment = || {
    x = x + 1  // &mut x
};

// Rule 3: Move if value consumed
let s = "hello";
let consumer = || {
    consume_string(s)  // s moved into closure
};
```

### Explicit Capture List (Phase 3)

For future consideration, allow explicit capture specification:

```zulon
// Explicit captures (optional syntax for Phase 3)
let closure = || [x, &mut y, &z] {
    // ...
};
```

---

## Closure Traits

### Hierarchy

```
FnOnce (can be called once)
  ↑
FnMut (can be called multiple times, modifies env)
  ↑
Fn (can be called multiple times, immutable)
```

### Trait Bounds

```zulon
// Fn: immutable closure
fn apply_fn<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32
{
    f(x)
}

// FnMut: mutable closure
fn apply_fn_mut<F>(f: F, x: i32) -> i32
where
    F: FnMut(i32) -> i32
{
    f(x)
}

// FnOnce: consuming closure
fn apply_once<F>(f: F, x: i32) -> i32
where
    F: FnOnce(i32) -> i32
{
    f(x)
}
```

### Implementation

- Compiler automatically implements appropriate traits based on capture semantics
- Immutable captures → Fn
- Mutable captures → FnMut
- Consuming captures → FnOnce

---

## Lowering Strategy

### Desugaring to Struct + Function

Closures will be desugared during HIR→MIR lowering:

**Original Code**:
```zulon
let multiplier = 10;
let multiply = |x| x * multiplier;
let result = multiply(5);
```

**Desugared MIR**:
```rust
// Closure environment struct
struct Closure_env {
    multiplier: i32,
}

// Closure body function
fn closure_body(env: &Closure_env, x: i32) -> i32 {
    env.multiplier * x
}

// Usage
let multiplier = 10;
let env = Closure_env { multiplier: multiplier };
let multiply = (closure_body, &env);
let result = multiply.0(multiply.1, 5);
```

### LLVM IR Generation

- Environment stored as struct
- Closure function takes environment pointer as first parameter
- Calls pass environment pointer implicitly

---

## Standard Library Integration

### Vec<T> Methods

```zulon
impl Vec<T> {
    // Apply function to each element
    fn map<U>(self: Vec<T>, f: |&T| -> U) -> Vec<U>

    // Filter elements
    fn filter(self: Vec<T>, f: |&T| -> bool) -> Vec<T>

    // Fold with accumulator
    fn fold<U>(self: Vec<T>, init: U, f: |&U, &T| -> U) -> U

    // For each (side effects)
    fn for_each(self: Vec<T>, f: |&T|)

    // Any element satisfies
    fn any(self: Vec<T>, f: |&T| -> bool) -> bool

    // All elements satisfy
    fn all(self: Vec<T>, f: |&T| -> bool) -> bool

    // Find first matching
    fn find(self: Vec<T>, f: |&T| -> bool) -> Option<&T>
}
```

### Option<T> Methods

```zulon
impl Option<T> {
    // Map if Some
    fn map<U>(self: Option<T>, f: |&T| -> U) -> Option<U>

    // Chain with another Option-returning function
    fn and_then<U>(self: Option<T>, f: |&T| -> Option<U>) -> Option<U>

    // Execute if Some
    fn and_then_some(self: Option<T>, f: |&T|) -> Option<T>
}
```

### Iterator Trait

```zulon
trait Iterator<T> {
    fn next(self: &mut Self) -> Option<T>

    fn map<U>(self: Self, f: |T| -> U) -> MapIterator<Self, T, U>
    fn filter(self: Self, f: |&T| -> bool) -> FilterIterator<Self, T>
    fn fold<U>(self: Self, init: U, f: |U, T| -> U) -> U
}
```

---

## Implementation Plan

### Phase 2 Week 1-2: Syntax and Type System

1. **Lexer Extension**
   - Add pipe `|` token (already exists as BitOr)
   - No new tokens needed

2. **Parser Extension**
   - Add closure expression parsing
   - Handle parameter lists
   - Support both expression and block bodies

3. **HIR Extension**
   - Add `HirExpression::Closure`
   - Add `HirCapture` enum
   - Add closure type representation

4. **Type Checker**
   - Closure type inference
   - Capture mode inference
   - Trait bound checking

### Phase 2 Week 3-4: MIR Lowering

5. **MIR Lowering**
   - Desugar closures to structs + functions
   - Generate environment structs
   - Closure body functions

6. **LIR Lowering**
   - Optimize closure calls
   - Inline small closures

### Phase 2 Week 5-6: Code Generation

7. **LLVM Codegen**
   - Environment struct layout
   - Closure function generation
   - Closure call generation

8. **Optimization**
   - Closure inlining
   - Environment elimination (when possible)

### Phase 2 Week 7-8: Standard Library

9. **Standard Library**
   - Implement Fn traits
   - Add Vec methods (map, filter, fold)
   - Add Option methods (map, and_then)

10. **Testing**
    - Unit tests for closure syntax
    - Integration tests for capture modes
    - Performance benchmarks

---

## Examples

### Example 1: Map

```zulon
fn main() -> i32 {
    let numbers = [1, 2, 3, 4, 5];
    let doubled = numbers.map(|x| x * 2);
    // doubled = [2, 4, 6, 8, 10]
    0
}
```

### Example 2: Filter

```zulon
fn main() -> i32 {
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let evens = numbers.filter(|x| x % 2 == 0);
    // evens = [2, 4, 6, 8, 10]
    0
}
```

### Example 3: Fold

```zulon
fn main() -> i32 {
    let numbers = [1, 2, 3, 4, 5];
    let sum = numbers.fold(0, |acc, x| acc + x);
    // sum = 15
    sum
}
```

### Example 4: Chaining

```zulon
fn main() -> i32 {
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let result = numbers
        .filter(|x| x % 2 == 0)
        .map(|x| x * x)
        .fold(0, |acc, x| acc + x);
    // 4 + 16 + 36 + 64 + 100 = 220
    result
}
```

### Example 5: Capture by Reference

```zulon
fn main() -> i32 {
    let threshold = 5;
    let numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Closure captures 'threshold' by reference
    let filtered = numbers.filter(|x| x > threshold);
    // filtered = [6, 7, 8, 9, 10]
    0
}
```

### Example 6: Capture by Mutable Reference

```zulon
fn main() -> i32 {
    let mut counter = 0;

    // Closure captures 'counter' by mutable reference
    let mut increment = || {
        counter = counter + 1;
        counter
    };

    increment();
    increment();
    increment();

    // counter = 3
    counter
}
```

### Example 7: Higher-Order Function

```zulon
fn apply_twice<F>(f: F, x: i32) -> i32
where
    F: Fn(i32) -> i32
{
    f(f(x))
}

fn main() -> i32 {
    let add_one = |x| x + 1;
    let result = apply_twice(add_one, 10);
    // result = 12
    result
}
```

### Example 8: Returning Closure

```zulon
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    |x| x + n
}

fn main() -> i32 {
    let add_10 = make_adder(10);
    let add_20 = make_adder(20);

    add_10(5) + add_20(5)  // 15 + 25 = 40
}
```

---

## Alternatives Considered

### Alternative 1: No Closures

**Rejected**: Would severely limit functional programming capabilities.

### Alternative 2: Function Pointers Only

**Rejected**: Cannot capture environment, severely limiting utility.

### Alternative 3: Blocks Instead of Closures

**Rejected**: Less ergonomic, harder to pass as arguments.

### Alternative 4: C++ Lambda Syntax

```zulon
// C++ style
let closure = [x](y) { return x + y; };
```

**Rejected**: More complex, less clean than pipe syntax.

---

## Open Questions

### Q1: Should we support explicit capture lists?

**Proposal**: Phase 3 (lifetimes needed first)
**Reasoning**: Inference works well for most cases, explicit captures add complexity

### Q2: Should closures support move semantics explicitly?

**Proposal**: Phase 3
**Reasoning**: Requires ownership system, borrow checker

### Q3: Should we support async closures?

**Proposal**: Phase 3 (async/await)
**Reasoning**: Requires async runtime

---

## Rationale

### Why Rust-Style?

1. **Proven**: Rust's closure design is battle-tested and well-liked
2. **Clean**: Pipe syntax is concise and readable
3. **Flexible**: Optional type annotations provide clarity when needed
4. **Consistent**: Matches function syntax where possible

### Why Not Other Options?

- **fn-style**: Too similar to named functions, loses visual distinction
- **Arrow style**: Conflicts with `>=` operator, less concise

---

## Impact

### Breaking Changes

None - this is a new feature.

### Compatibility

Fully backward compatible with Phase 1 code.

### Performance

- **Small overhead**: Environment allocation for closures with captures
- **Zero-cost**: Closure inlining eliminates overhead for small closures
- **Optimization**: Compiler can eliminate unused environment fields

---

## Unresolved Questions

1. **Closure representation**: Should we use trait objects or monomorphization?
   - **Proposal**: Monomorphize for performance, trait objects for dyn

2. **Recursive closures**: Should closures be able to call themselves?
   - **Proposal**: Yes, but requires explicit type annotation

3. **Default closure bounds**: Should Fn be the default?
   - **Proposal**: Yes, prefer immutable by default

---

## Future Enhancements

### Phase 3
- Explicit capture lists
- Move closures
- Async closures
- Closure lifetime annotations
- Coroutine support

---

## References

- [Rust Closure Documentation](https://doc.rust-lang.org/book/ch13-01-closures.html)
- [C++ Lambdas](https://en.cppreference.com/w/cpp/language/lambda)
- [JavaScript Arrow Functions](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Functions/Arrow_functions)
- [Swift Closures](https://docs.swift.org/swift-book/LanguageGuide/Closures.html)

---

## Conclusion

This RFC proposes Rust-style closures for ZULON, providing:
- ✅ Clean, concise syntax
- ✅ Powerful type inference
- ✅ Flexible capture semantics
- ✅ Integration with standard library
- ✅ Zero-cost abstractions (via inlining)

Closures will significantly enhance ZULON's expressiveness and enable functional programming patterns while maintaining the language's focus on performance and type safety.

---

**RFC Version**: 1.0
**Status**: Draft - Open for Comments
**Next Steps**: Community feedback, implementation planning
