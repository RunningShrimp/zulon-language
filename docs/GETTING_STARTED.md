# ZULON Language - Getting Started Guide

Welcome to ZULON, a systems programming language inspired by Rust, focusing on safety, concurrency, and performance.

## Table of Contents

1. [Installation](#installation)
2. [Your First Program](#your-first-program)
3. [Basic Syntax](#basic-syntax)
4. [Compiling and Running](#compiling-and-running)
5. [Language Features](#language-features)
6. [Next Steps](#next-steps)

## Installation

### Prerequisites

- Rust toolchain (1.70+)
- LLVM 15+ (for code generation)
- Git

### Building from Source

```bash
# Clone the repository
git clone https://github.com/zulon-lang/zulon.git
cd zulon

# Build the compiler
cargo build --release

# The compiler binary will be at:
# ./target/release/zulon-compiler
```

### Verifying Installation

```bash
cargo run -p zulon-compiler -- --version
```

## Your First Program

Create a file named `hello.zl`:

```zulon
extern fn println(s: string);

fn main() -> i32 {
    println("Hello, World!");
    0
}
```

## Basic Syntax

### Functions

Functions are declared with the `fn` keyword:

```zulon
fn add(a: i32, b: i32) -> i32 {
    let result = a + b;
    result
}
```

### Variables

Variables are declared with `let`:

```zulon
let x = 42;
let name = `ZULON`;
let is_ready = true;
```

### Control Flow

#### If/Else

```zulon
if x > 0 {
    println("Positive");
} else {
    println("Non-positive");
}
```

#### Loops

```zulon
let i = 0;
while i < 10 {
    println(i);
    let i = i + 1;
}
```

#### Match Expressions

```zulon
let result = match x {
    1 => "one",
    2 => "two",
    _ => "other"
};
```

**Note**: Match arms with block expressions require commas:

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
        println("Got something else");
        0
    }
};
```

### Data Types

#### Primitive Types

- **Integers**: `i32`, `i64`, `u32`, `u64`
- **Booleans**: `bool`
- **Strings**: `string` (static template strings)

#### Tuples

Tuples are fixed-size collections with numeric field access:

```zulon
// Create tuples
let single = (42,);
let pair = (1, 2);
let triple = (1, 2, 3);

// Access fields
let first = pair.0;
let second = pair.1;
```

#### Enums

```zulonenum Option {
    Some(i32),
    None,
}

fn main() -> i32 {
    let value = Option::Some(42);

    match value {
        Option::Some(v) => {
            println("Has value");
            v
        },
        Option::None => {
            println("No value");
            0
        }
    }
}
```

### Defer Statements

Defer statements run cleanup code when a scope exits (LIFO order):

```zulon
fn main() -> i32 {
    println("Starting");

    // Defer statements execute in reverse order
    defer println("Cleanup 3")
    defer println("Cleanup 2")
    defer println("Cleanup 1")

    println("Working");
    0
}

// Output:
// Starting
// Working
// Cleanup 1
// Cleanup 2
// Cleanup 3
```

**Important**: Defer statements must NOT have semicolons:

```zulon
// ✅ Correct
defer println("cleanup")

// ❌ Wrong - will cause parse error
defer println("cleanup");
```

### Template Strings

Template strings use backticks:

```zulon
let greeting = `Hello, World!`;
let message = `Static strings work`;
```

**Note**: Template string interpolation (e.g., `` `Hello, ${name}` ``) is not yet implemented. Only static strings are supported.

## Compiling and Running

### Basic Compilation

```bash
cargo run -p zulon-compiler -- your_program.zl
```

This generates:
- `your_program.ll` - LLVM IR
- `your_program.s` - Assembly code
- Attempted executable linking (may fail without extern implementations)

### Viewing LLVM IR

```bash
# The compiler generates LLVM IR
cargo run -p zulon-compiler -- example.zl

# View the generated IR
cat example.ll
```

### Manual Compilation to Executable

```bash
# Generate LLVM IR
cargo run -p zulon-compiler -- example.zl

# Compile to assembly
llc example.ll -o example.s

# Assemble and link (requires providing extern functions)
clang example.s -o example

# Run
./example
```

## Language Features

### Current Status (As of January 2026)

#### ✅ Fully Working

- **Functions**: Declaration, calls, returns
- **Basic types**: i32, i64, bool, string
- **Control flow**: if/else, while loops, early returns
- **Match expressions**: Integer literals, wildcards, block arms
- **Tuples**: Creation and numeric field access
- **Defer statements**: Single expression defer
- **Static template strings**: Backtick string literals
- **Extern functions**: C ABI function declarations

#### ⚠️ Partially Working

- **Tuple construction** (90%): Multi-element tuples have MIR limitations
- **Defer statements** (85%): No early return or error path handling
- **Template strings** (60%): Only static strings, no interpolation

#### ❌ Not Yet Implemented

- Template string interpolation
- Generic type instantiation
- Enum variant construction (in some contexts)
- Closures and capture
- Async/await
- Modules and imports
- Macros

## Examples

The `examples/` directory contains working example programs:

- `hello_world.zl` - Basic Hello World
- `control_flow.zl` - If/else, loops, returns
- `template_strings.zl` - Static template strings
- `defer_demo.zl` - Defer statement demonstration
- `tuples_demo.zl` - Tuple creation and field access
- `pattern_matching.zl` - Match expressions

See [examples/README_EN.md](../examples/README_EN.md) for details.

## Common Patterns

### Error Handling (Basic)

```zulon
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        println("Error: Division by zero");
        return 0;
    }
    a / b
}
```

### Loop with Break

```zulon
fn find_value(target: i32) -> i32 {
    let i = 0;
    while i < 100 {
        if i == target {
            return i;
        }
        let i = i + 1;
    }
    -1  // Not found
}
```

### Resource Cleanup

```zulon
fn process_file() -> i32 {
    println("Opening file");

    defer println("Closing file")

    println("Processing file");
    0
}
```

## Limitations and Known Issues

### Parser Limitations

1. **Defer statements**: Must not end with semicolon
2. **Match arms**: Block expressions require commas (except last arm)
3. **Template interpolation**: Not supported, use static strings only

### Type System Limitations

1. **Generic types**: Parser support exists but type checker incomplete
2. **Enum variants**: Some construction patterns don't work
3. **Type inference**: Limited to simple cases

### Runtime Limitations

1. **No standard library**: Must declare all extern functions manually
2. **No garbage collection**: Manual memory management required
3. **Limited error handling**: No Result type or error propagation

## Troubleshooting

### Parse Errors

**"unexpected token in expression"**
- Check for missing commas in match expressions
- Ensure defer statements don't have semicolons
- Verify template strings don't use interpolation

### Type Errors

**"cannot find value in this scope"**
- Ensure all variables are declared before use
- Check that extern functions are declared
- Verify enum variant syntax

### Link Errors

**"Undefined symbols for architecture"**
- Extern functions (println, print_int) must be provided
- Compiler generates LLVM IR but linking requires C implementations
- Use `--keep-intermediates` flag to inspect generated code

## Next Steps

1. **Explore Examples**: Check out the `examples/` directory
2. **Read Language Features**: See [LANGUAGE_FEATURES.md](LANGUAGE_FEATURES.md)
3. **Contribute**: See [CONTRIBUTING.md](CONTRIBUTING.md) in the repository
4. **Report Issues**: Open an issue on GitHub for bugs or feature requests

## Additional Resources

- [Implementation Plan](../IMPLEMENTATION_PLAN.md) - Development roadmap
- [Todo List](../TODOLIST.md) - Current tasks and status
- [Ralph Loop Documentation](../.claude/ralph-loop.local.md) - Development methodology
- [Examples](../examples/README_EN.md) - Working example programs

## Support

- **GitHub Issues**: https://github.com/zulon-lang/zulon/issues
- **Documentation**: https://docs.zulon-lang.org
- **Community**: Join our Discord server

---

**Welcome to ZULON!** 🚀

We're excited to have you aboard. The compiler is under active development, and we welcome contributions and feedback.
