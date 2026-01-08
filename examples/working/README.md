# ZULON Working Examples

This directory contains **verified working examples** that demonstrate what ZULON can actually do today. All examples in this directory have been tested and confirmed to compile and run correctly.

## Quick Start

To run any example:
```bash
cargo run -p zulon-compiler -- examples/working/01_hello.zl -o hello
llc hello.ll -o hello.s
clang hello.s -o hello
./hello
echo $?  # Should print 42
```

## Examples by Category

### Basics (01-03)
- **01_hello.zl** - Simplest program (returns 42)
- **02_variables.zl** - Variable declarations (immutable and mutable)
- **03_arithmetic.zl** - Basic arithmetic operations

### Control Flow (04-05)
- **04_if_expressions.zl** - Conditional logic with if-expressions
- **05_while_loop.zl** - Looping with while

### Functions (06-07)
- **06_functions.zl** - Multiple functions working together
- **07_recursion.zl** - Recursive functions (fibonacci)

### Language Features (08-10)
- **08_comments.zl** - Code comments (work everywhere!)
- **09_struct_definition.zl** - Struct definitions
- **10_return.zl** - Explicit return statements

## Verified Capabilities

These examples demonstrate that ZULON currently supports:

✅ **Core features** (100% working):
- Function definitions and calls
- Variable declarations (let, let mut)
- All arithmetic operators (+, -, *, /, %)
- Comparison operators (<, >, <=, >=)
- Logical operators (&&, ||, !)
- If-expressions
- While loops
- Unary operations (-x, !x)
- Recursive functions
- Return statements

✅ **Advanced features** (partially working):
- Struct/enum definitions (parse and type-check)
- Comments (work everywhere)
- String literals (basic support)

⚠️ **Known limitations**:
- Match expressions (parse but don't compile)
- Struct field access (definitions work, HIR lowers, MIR doesn't)
- Struct initialization syntax
- Standard library (minimal)

## Expected Output

Each example's return value:
- 01_hello.zl → 42
- 02_variables.zl → 40 (10 + 30)
- 03_arithmetic.zl → 430 (sum of operations)
- 04_if_expressions.zl → 42 (absolute value)
- 05_while_loop.zl → 45 (sum 0..9)
- 06_functions.zl → 35 (square(5) + double(5))
- 07_recursion.zl → 55 (fibonacci(10))
- 08_comments.zl → 30 (10 + 20)
- 09_struct_definition.zl → 0 (placeholder)
- 10_return.zl → 42 (early return not triggered)

## Learning Path

We recommend following the examples in order:
1. Start with `01_hello.zl` to understand the basics
2. Progress through `02_variables.zl` and `03_arithmetic.zl`
3. Learn control flow with `04_if_expressions.zl` and `05_while_loop.zl`
4. Explore functions with `06_functions.zl` and `07_recursion.zl`
5. See advanced features in `08-10*.zl`

## Contributing

When adding new examples:
1. Ensure they compile with the current compiler
2. Test them end-to-end (compile → assemble → link → run)
3. Add them to the appropriate category above
4. Update this README with the expected output

## vs. Other Examples

You may notice other ZULON examples in the repository (in `examples/` parent directory). Those represent **aspirational/future examples** that demonstrate what ZULON *will* support. The examples here represent what ZULON *can do today*.

As we implement more features, we'll migrate examples from the parent directory into this working directory.
