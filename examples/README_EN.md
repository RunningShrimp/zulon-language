# ZULON Language Examples

This directory contains example programs demonstrating the ZULON compiler's capabilities.

## Quick Start

```bash
# Compile an example (generates LLVM IR)
cargo run -p zulon-compiler -- examples/hello_world.zl

# View the generated LLVM IR
cat examples/hello_world.ll

# Manually compile LLVM IR to executable (requires clang)
llc examples/hello_world.ll -o examples/hello_world.s
clang examples/hello_world.s -o examples/hello_world
./hello_world
```

**Note**: The compiler generates LLVM IR but linking requires external C functions (println, print_int, etc.) to be provided.

## Example Programs

### 1. hello_world.zl
**Basic Hello World program**

Demonstrates:
- Function declaration with `fn main() -> i32`
- External function declarations with `extern`
- Static template strings
- Return statements

**Status**: ✅ Fully working

**Run it**:
```bash
cargo run -- examples/hello_world.zl
```

---

### 2. template_strings.zl
**Template string demonstration**

Demonstrates:
- Static template strings (no interpolation)
- String literals with backticks
- Multiple string assignments

**Status**: ✅ Static strings work
**Note**: Template string interpolation (`` `Hello, ${name}` ``) is not yet implemented.

**Run it**:
```bash
cargo run -- examples/template_strings.zl
```

---

### 3. defer_demo.zl
**Defer statement demonstration**

Demonstrates:
- Defer with single expression
- Multiple defer statements
- LIFO execution order (Last In, First Out)
- Cleanup operations

**Status**: ✅ Fully working
**Note**: Defer statements must not have semicolons at the end (parser limitation).

**Expected output**:
```
Starting main function
Main code executing
About to return
Cleanup 3: Last registered, first to execute
Cleanup 2: Second registered
Cleanup 1: First registered, last to execute
```

**Run it**:
```bash
cargo run -p zulon-compiler -- examples/defer_demo.zl
```

---

### 4. tuples_demo.zl
**Tuple creation and field access**

Demonstrates:
- Single-element tuple creation: `(42,)`
- Pair tuple creation: `(10, 20)`
- Triple tuple creation: `(1, 2, 3)`
- Numeric field access: `tuple.0`, `tuple.1`, etc.

**Status**: ✅ Field access works perfectly
**Note**: Multi-element tuple construction has MIR limitations (returns first element). Field access works for all tuple sizes.

**Run it**:
```bash
cargo run -p zulon-compiler -- examples/tuples_demo.zl
```

---

### 5. pattern_matching.zl
**Match expression demonstration**

Demonstrates:
- Match expressions with integer literals
- Match arms with block expressions
- Pattern matching with wildcards (`_`)
- Match arm result binding

**Status**: ✅ Fully working
**Note**: Match arms with block expressions need commas (except last arm).

**Run it**:
```bash
cargo run -p zulon-compiler -- examples/pattern_matching.zl
```

---

### 6. control_flow.zl
**Control flow demonstration**

Demonstrates:
- If/else statements
- While loops
- Variable shadowing in loops
- Early returns
- Expression-based return values

**Status**: ✅ Fully working

**Expected output**:
```
x is greater than 5
0
1
2
Loop finished
Early return
```

**Run it**:
```bash
cargo run -p zulon-compiler -- examples/control_flow.zl
```

---

## Known Limitations

### Template Strings
- ✅ Static strings work: `` `Hello, World!` ``
- ❌ Interpolation doesn't work yet: `` `Hello, ${name}` ``
- Status: 60% complete

### Tuples
- ✅ Field access works: `t.0`, `t.1`, etc.
- ⚠️ Multi-element tuple construction has MIR limitations
- Status: 90% complete

### Defer
- ✅ Single expression: `defer println("cleanup")`
- ✅ LIFO execution order
- ⚠️ No semicolon allowed at end of defer statement
- Status: 85% complete

### Match Expressions
- ✅ Match with integer literals
- ✅ Wildcard patterns (`_`)
- ✅ Match arms with blocks
- ⚠️ Block match arms need commas
- Status: 95% complete

## Learning Path

We recommend exploring the examples in this order:

1. **hello_world.zl** - Basic syntax and structure
2. **control_flow.zl** - Control flow constructs
3. **template_strings.zl** - String handling
4. **defer_demo.zl** - Cleanup and resource management
5. **tuples_demo.zl** - Composite data types
6. **pattern_matching.zl** - Pattern matching and enums

## Troubleshooting

### Parse Errors
If you encounter a parse error, check:
- All statements end with semicolons
- Blocks use proper brace matching `{ }`
- Template strings use backticks `` ` ``
- Numeric tuple access uses dots: `t.0` not `t[0]`

### Type Errors
The compiler requires explicit type annotations:
- Function parameters: `fn foo(x: i32) -> i32`
- Variable types can often be inferred, but explicit types help

### Runtime Errors
If your program compiles but doesn't run correctly:
- Check for uninitialized variables
- Verify extern function signatures match expected C ABI
- Ensure all code paths return appropriate values

## Contributing

When adding new examples:
1. Make sure they compile successfully
2. Add clear comments explaining the features
3. Update this README with a description
4. Note any known limitations
5. Test on the latest compiler version

## See Also

- **README.md** - Chinese documentation with detailed explanations
- **../IMPLEMENTATION_PLAN.md** - Compiler implementation roadmap
- **../TODOLIST.md** - Current development tasks and status
