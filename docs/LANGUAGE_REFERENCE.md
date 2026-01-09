# ZULON Language Features Reference

Complete reference of ZULON language features and their current implementation status.

## Table of Contents

1. [Types](#types)
2. [Expressions](#expressions)
3. [Statements](#statements)
4. [Functions](#functions)
5. [Control Flow](#control-flow)
6. [Pattern Matching](#pattern-matching)
7. [Defer](#defer)
8. [Template Strings](#template-strings)
9. [Extern Declarations](#extern-declarations)
10. [Implementation Status](#implementation-status)

---

## Types

### Primitive Types

#### Integer Types

```zulon
let x: i32 = 42;
let y: i64 = 1000000000;
let z: u32 = 255;
```

**Status**: ✅ Fully implemented

#### Boolean Type

```zulon
let is_true: bool = true;
let is_false: bool = false;
```

**Status**: ✅ Fully implemented

#### String Type

```zulon
let greeting: string = `Hello, World!`;
```

**Status**: ✅ Static strings fully implemented
**Note**: String interpolation not yet supported

### Composite Types

#### Tuple Types

```zulon
// Single-element tuple (requires comma)
let single: (i32,) = (42,);

// Pair
let pair: (i32, i32) = (1, 2);

// Triple
let triple: (i32, i32, i32) = (1, 2, 3);

// Field access with numeric indices
let first = pair.0;
let second = pair.1;
```

**Status**: ⚠️ 90% complete
- ✅ Parser: Full support
- ✅ Type checking: Works
- ✅ Field access: `tuple.0`, `tuple.1`, etc.
- ⚠️  Construction: Multi-element tuples have MIR limitations

#### Enum Types

```zulon
enum Option {
    Some(i32),
    None,
}

enum Result {
    Ok(i32),
    Err(string),
}
```

**Status**: ✅ Declaration implemented
**Note**: Variant construction has limitations in some contexts

---

## Expressions

### Literals

```zulon
// Integer literals
42
-10
0

// Boolean literals
true
false

// String literals (template strings)
`Hello, World!`
```

**Status**: ✅ Fully implemented

### Binary Operators

```zulon
// Arithmetic
+  -  *  /  %

// Comparison
==  !=  <  >  <=  >=

// Logical
&&  ||  !
```

**Status**: ✅ Fully implemented

### Block Expressions

```zulon
let result = {
    let x = 10;
    let y = 20;
    x + y  // Last expression is the value
};
```

**Status**: ✅ Fully implemented

### Path Expressions

```zulon
// Variable access
let x = value;

// Enum variant access
let opt = Option::Some(42);
```

**Status**: ⚠️ Partially implemented
- ✅ Variable access
- ⚠️  Enum variant paths (context-dependent)

### Field Access

```zulon
// Tuple field access
let first = tuple.0;
let second = tuple.1;
```

**Status**: ✅ Fully implemented for tuples

---

## Statements

### Let Statements

```zulon
let x = 42;
let name = `ZULON`;
let mutable = 10;  // Note: mut keyword not yet enforced
```

**Status**: ✅ Fully implemented

### Expression Statements

```zulon
println("Hello");
let x = y + z;
```

**Status**: ✅ Fully implemented

### Defer Statements

```zulon
defer println("cleanup")
```

**Status**: ⚠️ 85% complete
- ✅ Basic defer with single expression
- ❌ No early return handling
- ❌ No error path cleanup
- ⚠️  **Important**: Must NOT have semicolon at end

See [Defer](#defer) section for details.

---

## Functions

### Function Declaration

```zulon
fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

**Status**: ✅ Fully implemented

### Function Calls

```zulon
let result = add(10, 20);
println("Hello");
```

**Status**: ✅ Fully implemented

### Return Statements

```zulon
fn early_return(x: i32) -> i32 {
    if x < 0 {
        return 0;
    }
    x
}
```

**Status**: ✅ Fully implemented

### Extern Functions

```zulon
extern fn println(s: string);
extern fn print_int(n: i32);

fn main() -> i32 {
    println("Hello");
    print_int(42);
    0
}
```

**Status**: ✅ Fully implemented (C ABI)

---

## Control Flow

### If Expressions

```zulon
// As expression
let result = if x > 0 {
    "positive"
} else {
    "non-positive"
};

// As statement
if x > 5 {
    println("Greater than 5");
} else {
    println("5 or less");
}
```

**Status**: ✅ Fully implemented

### While Loops

```zulon
let i = 0;
while i < 10 {
    println(i);
    let i = i + 1;  // Variable shadowing in loop
}
```

**Status**: ✅ Fully implemented
**Note**: No `for` loops or `loop` keywords yet

---

## Pattern Matching

### Match Expressions

```zulon
let result = match value {
    1 => "one",
    2 => "two",
    _ => "other"
};
```

**Status**: ✅ Fully implemented for integer literals

### Match Arms with Blocks

```zulon
let y = match x {
    1 => {
        println("Got one");
        1
    },
    2 => {
        println("Got two");
        2
    },
    _ => {
        println("Something else");
        0
    }
};
```

**Status**: ✅ Fully implemented
**Note**: Block arms require commas (except last arm)

### Wildcard Patterns

```zulon
match value {
    _ => println("Catch-all")
}
```

**Status**: ✅ Fully implemented

### Variable Bindings

```zulon
enum Option {
    Some(i32),
    None,
}

match opt {
    Option::Some(v) => v,  // v is bound
    Option::None => 0,
}
```

**Status**: ⚠️ Partially implemented
- ✅ Works in some contexts
- ❌ Type checker issues with enum variants in others

---

## Defer

Defer statements execute cleanup code when a scope exits, in LIFO (Last In, First Out) order.

### Basic Usage

```zulon
fn process() -> i32 {
    defer println("cleanup")

    println("working");
    0
}
// Output: working, then cleanup
```

### Multiple Defers

```zulon
fn main() -> i32 {
    defer println("cleanup 3")
    defer println("cleanup 2")
    defer println("cleanup 1")

    println("main");
    0
}
// Output: main, cleanup 1, cleanup 2, cleanup 3
```

**Status**: ⚠️ 85% complete

### Limitations

1. **No semicolons**: Defer statements must NOT end with semicolon
   ```zulon
   // ✅ Correct
   defer println("cleanup")

   // ❌ Wrong - causes parse error
   defer println("cleanup");
   ```

2. **No early return handling**: Defer only executes at normal block exit
3. **No error path cleanup**: Defer doesn't execute on error paths
4. **No multi-statement blocks**: Only single expressions supported

---

## Template Strings

Template strings use backticks (`) instead of quotes.

### Static Strings

```zulon
let greeting = `Hello, World!`;
let message = `Static strings work perfectly`;
```

**Status**: ✅ Fully implemented

### String Interpolation (Not Yet Implemented)

```zulon
// ❌ This does NOT work yet
let name = "ZULON";
let greeting = `Hello, ${name}!`;  // Parse error
```

**Status**: ❌ Not implemented (60% complete)
- Parser infrastructure exists
- Token stream management needs refactoring
- Estimated 4-5 iterations to complete

---

## Extern Declarations

Extern functions declare functions with C ABI linkage.

### Basic Extern

```zulon
extern fn printf(format: string, ...) -> i32;
extern fn strlen(s: string) -> i32;
```

**Status**: ✅ Fully implemented

### Common External Functions

```zulon
extern fn println(s: string);
extern fn print_int(n: i32);
extern fn print_bool(b: bool);
```

**Status**: ✅ Fully implemented

### Linking

**Note**: The compiler generates LLVM IR but doesn't provide implementations. You must link with C libraries or provide your own implementations.

```bash
# Generate IR
cargo run -p zulon-compiler -- program.zl

# View IR
cat program.ll

# Compile and link (requires C implementations)
llc program.ll -o program.s
clang program.s your_implementations.o -o program
```

---

## Implementation Status

### Completed Features (100%)

- ✅ Functions (declaration, calls, returns)
- ✅ Basic types (i32, i64, bool)
- ✅ Control flow (if/else, while)
- ✅ Variable declarations
- ✅ Extern functions
- ✅ Static template strings
- ✅ Tuple field access (.0, .1, etc.)
- ✅ Match expressions (integer literals)
- ✅ Wildcard patterns (_)

### Partially Implemented Features

- ⚠️  **Tuples** (90%): Field access works, construction has MIR limitations
- ⚠️  **Defer** (85%): Basic support, no early return/error handling
- ⚠️  **Template strings** (60%): Static only, no interpolation
- ⚠️  **Enum variant construction** (varies by context)
- ⚠️  **Pattern matching** (95%): Integer patterns work, enum patterns context-dependent

### Not Yet Implemented

- ❌ Template string interpolation
- ❌ Generic type instantiation
- ❌ For loops
- ❌ Loop construct
- ❌ Break/continue
- ❌ Struct declarations
- ❌ Methods
- ❌ Closures
- ❌ Capture semantics
- ❌ Async/await
- ❌ Modules/imports
- ❌ Macros (except built-in)
- ❌ Type inference (limited)
- ❌ Mutability checking
- ❌ Borrow checking
- ❌ Error handling (Result, Error types)

---

## Syntax Quick Reference

### Function Declaration

```zulon
fn name(param1: Type1, param2: Type2) -> ReturnType {
    body
}
```

### Variable Declaration

```zulon
let name = value;
let name: Type = value;
```

### If Expression

```zulon
if condition {
    consequence
} else {
    alternative
}
```

### Match Expression

```zulon
match scrutinee {
    pattern1 => expression1,
    pattern2 => expression2,
    _ => default_expression
}
```

### Defer Statement

```zulon
defer expression  // NO semicolon!
```

### Tuple

```zulon
let tuple = (value1, value2, value3);
let first = tuple.0;
```

---

## Migration from Rust

ZULON is inspired by Rust. Here are key similarities and differences:

### Similarities

- `fn` for functions
- `let` for variables
- `if` as expression
- `match` for pattern matching
- `extern` for foreign functions
- C-style syntax

### Differences

| Rust | ZULON |
|------|-------|
| `String` or `&str` | `string` |
| `"string"` | `` `string` `` |
| `let mut` | `let` (mut not enforced yet) |
| `Drop` trait | `defer` statement |
| `Result<T, E>` | Not yet implemented |
| `Option<T>` | Enum only, no type parameter |
| `loop` | Not yet implemented |
| `for` | Not yet implemented |

---

## Future Roadmap

See [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) for the complete roadmap.

### Phase 2.1 (Current)

- ✅ Generic type parsing
- ✅ Tuple field access
- ✅ Basic defer
- ⚠️ Template strings (partial)

### Phase 2.2 (Planned)

- Template string interpolation
- Enhanced defer (early returns, error paths)
- Enum variant construction
- Struct declarations

### Phase 3+ (Future)

- Borrow checking
- Closures and capture
- Async/await
- Standard library
- Package manager

---

**Last Updated**: January 2026

**Compiler Version**: 0.1.0

**For the latest information, check the GitHub repository.**
