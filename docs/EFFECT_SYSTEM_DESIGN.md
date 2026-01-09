# ZULON Effect System Design

**Version**: 1.0
**Date**: 2026-01-08
**Status**: Design Phase

## Overview

The ZULON effect system provides a structured way to handle side effects and computational effects. Effects make implicit operations (IO, state, exceptions) explicit while maintaining code readability.

## Motivation

### Problems with Traditional Approaches

1. **Exceptions**: Hard to track what functions can throw, non-local control flow
2. **Monads**: Require deep nesting, cognitive overhead
3. **Implicity**: Hidden side effects make reasoning difficult

### Effect System Benefits

1. **Explicit Effects**: Function signatures declare what effects they use
2. **Composable**: Effects can be combined and layered
3. **Flexible Handlers**: Same operation can have different implementations
4. **Type-Safe**: Compiler enforces effect handling

## Syntax Design

### Effect Definition

```zulon
// Define an effect with operations
effect Log {
    fn log(message: &str);
}

effect State<T> {
    fn get() -> T;
    fn set(value: T);
}

effect IO {
    fn read(path: &str) -> &str;
    fn write(path: &str, content: &str);
}
```

### Effectful Functions

```zulon
// Functions declare effects with | separator
fn process_data() -> i32 | Log, IO {
    log("Starting processing");  // perform log operation
    let data = read("data.txt"); // perform IO operation
    log("Finished processing");
    data.len() as i32
}
```

### Effect Operations

```zulon
// Using effect operations
fn example() -> i32 | Log {
    perform log("Hello, world!");  // Explicit perform
    42
}

// Shorthand: perform is optional for unambiguous operations
fn example2() -> i32 | Log {
    log("Hello, world!");  // Implicit perform
    42
}
```

### Effect Handlers (try...with)

```zulon
// Handle effects in a block
fn main() -> i32 {
    try {
        // Code that uses Log effect
        process_data();
    } with Log {
        // Handle log operations
        fn log(message: &str) {
            println!("{}", message);
        }
    }
}
```

### Effect Composition

```zulon
// Handle multiple effects
fn main() -> i32 {
    try {
        process_data();
    } with {
        Log {
            fn log(message: &str) {
                println!("{}", message);
            }
        }
        IO {
            fn read(path: &str) -> &str {
                std::fs::read_to_string(path).unwrap()
            }
            fn write(path: &str, content: &str) {
                std::fs::write(path, content).unwrap()
            }
        }
    }
}
```

### Nested Handlers

```zulon
// Handlers can be nested
fn main() -> i32 {
    try {
        try {
            process_data();
        } with Log {
            fn log(message: &str) {
                println!("LOG: {}", message);
            }
        }
    } with IO {
        fn read(path: &str) -> &str {
            std::fs::read_to_string(path).unwrap()
        }
        fn write(path: &str, content: &str) {
            std::fs::write(path, content).unwrap()
        }
    }
}
```

### Deep Handlers (Resume)

```zulon
// Handlers can resume with different values
effect Ask {
    fn ask(prompt: &str) -> &str;
}

fn interactive() -> i32 | Ask {
    let name = ask("What's your name? ");
    let age = ask("How old are you? ").parse::<i32>().unwrap();
    age
}

fn main() -> i32 {
    try {
        interactive();
    } with Ask {
        fn ask(prompt: &str) -> &str {
            // Resume with default value
            if cfg!(test) {
                "test_user"
            } else {
                // Actually prompt user
                println!("{}", prompt);
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                input.trim()
            }
        }
    }
}
```

## Type System

### Effect Signatures

```zulon
// Pure function (no effects)
fn add(a: i32, b: i32) -> i32 {
    a + b
}

// Function with Log effect
fn log_and_add(a: i32, b: i32) -> i32 | Log {
    log("Adding numbers");
    a + b
}

// Function with multiple effects
fn complex() -> i32 | Log, IO, State<i32> {
    log("Complex operation");
    let data = read("data.txt");
    let state = get();
    set(state + 1);
    data.len() as i32 + state
}
```

### Effect Polymorphism (Future)

```zulon
// Generic over effects (not in initial implementation)
fn foreach<T>(items: Vec<T>, f: fn(T) | E) -> () | E {
    for item in items {
        f(item);
    }
}
```

## Implementation Strategy

### Phase 1: Syntax and Parsing (Week 1)

**Tasks**:
1. Add `effect` keyword to lexer
2. Add `perform` keyword to lexer (optional)
3. Add `try...with` block parsing
4. Add effect type syntax to parser

**AST Extensions**:
```rust
enum ItemKind {
    Effect(Effect),  // NEW
    // ...
}

struct Effect {
    name: Identifier,
    operations: Vec<EffectOperation>,
}

struct EffectOperation {
    name: Identifier,
    params: Vec<Param>,
    return_type: Option<Type>,
}

enum StatementKind {
    TryWith(TryWithBlock),  // NEW
    Perform(PerformExpr),  // NEW
    // ...
}

struct TryWithBlock {
    try_block: Block,
    handlers: Vec<EffectHandler>,
}

struct EffectHandler {
    effect_name: Identifier,
    operations: Vec<HandlerOperation>,
}

struct HandlerOperation {
    name: Identifier,
    params: Vec<Param>,
    body: Block,
}
```

### Phase 2: Type Checking (Week 1-2)

**Tasks**:
1. Add effect checking to type checker
2. Verify effects are declared in function signatures
3. Verify effect operations are valid
4. Check that all used effects are handled

**Type Environment Extensions**:
```rust
struct TypeEnv {
    // Add effect tracking
    effects: HashMap<String, EffectDefinition>,
}

struct FunctionSignature {
    params: Vec<Param>,
    return_type: Type,
    effects: Vec<Effect>,  // NEW
}
```

### Phase 3: MIR Lowering (Week 2)

**Tasks**:
1. Add `Perform` instruction to MIR
2. Add effect handling to MIR
3. Implement effect stack/tracking
4. Generate code for try...with blocks

**MIR Extensions**:
```rust
enum MirInstruction {
    Perform {
        operation: String,
        args: Vec<MirPlace>,
        dest: TempVar,
        resume_block: MirNodeId,  // Where to resume after handling
    },
    EffectHandler {
        effect_name: String,
        operations: Vec<HandlerImpl>,
        body_block: MirNodeId,
    },
    // ...
}

struct HandlerImpl {
    operation_name: String,
    params: Vec<Param>,
    implementation_block: MirNodeId,
}
```

### Phase 4: Code Generation (Week 2-3)

**Tasks**:
1. Implement effect dispatch mechanism
2. Generate code for effect handlers
3. Implement effect stack (runtime)
4. Test with simple examples

**LLVM IR Generation**:
```llvm
; Effect operations become function calls with special linkage
define void @effect_log(i8* %msg) {
  ; Look up current handler for Log effect
  ; Call handler implementation
  ; Return to resume point
  ret void
}

; Effect handlers capture the continuation
define void @handle_log(i8* %msg, void (%resume)* %resume) {
  ; Handler implementation
  call void @puts(i8* %msg)
  ; Call resume to continue
  call void %resume()
  ret void
}
```

### Phase 5: Built-in Effects (Week 3)

**Standard Library Effects**:

1. **Log Effect**
```zulon
effect Log {
    fn log(message: &str);
}
```

2. **IO Effect**
```zulon
effect IO {
    fn read(path: &str) -> &str;
    fn write(path: &str, content: &str);
    fn print(msg: &str);
    fn println(msg: &str);
}
```

3. **State Effect**
```zulon
effect State<T> {
    fn get() -> T;
    fn set(value: T);
    fn modify(f: fn(T) -> T);
}
```

## Examples

### Example 1: Simple Logging

```zulon
effect Log {
    fn log(msg: &str);
}

fn process() -> i32 | Log {
    log("Starting");
    let result = 42;
    log("Finished");
    result
}

fn main() -> i32 {
    try {
        process()
    } with Log {
        fn log(msg: &str) {
            println!("LOG: {}", msg);
        }
    }
}
```

### Example 2: File IO with Error Handling

```zulon
effect IO {
    fn read(path: &str) -> &str;
}

fn load_config() -> &str | IO {
    read("config.toml")
}

fn main() -> i32 {
    try {
        let config = load_config();
        println!("Config: {}", config);
        0
    } with IO {
        fn read(path: &str) -> &str {
            std::fs::read_to_string(path).unwrap_or_else(|_| {
                "{}"
            })
        }
    }
}
```

### Example 3: State Effect

```zulon
effect State<T> {
    fn get() -> T;
    fn set(value: T);
}

fn counter() -> i32 | State<i32> {
    let count = get();
    set(count + 1);
    get()
}

fn main() -> i32 {
    try {
        set(0);
        counter();  // Returns 1
        counter();  // Returns 2
        counter()   // Returns 3
    } with State<i32> {
        fn get() -> i32 {
            // Use local variable or heap storage
            __state__
        }
        fn set(value: i32) {
            __state__ = value;
        }
    }
}
```

## Implementation Timeline

**Week 1**: Syntax and Parsing
- Day 1-2: Lexer extensions, AST definitions
- Day 3-4: Parser implementation
- Day 5: Testing and validation

**Week 2**: Type Checking and MIR Lowering
- Day 1-3: Type checker extensions
- Day 4-5: MIR lowering and effect tracking

**Week 3**: Code Generation and Built-in Effects
- Day 1-3: LLVM code generation
- Day 4-5: Built-in effects (Log, IO)

## Testing Strategy

1. **Unit Tests**: Parser, type checker, MIR lowering
2. **Integration Tests**: End-to-end effect handling
3. **Examples**: Realistic use cases
4. **Performance**: Effect dispatch overhead

## Future Enhancements

1. **Effect Polymorphism**: Generic over effects
2. **Effect Composition**: Combine effects automatically
3. **Effect Inference**: Infer effects from function body
4. **Async Effects**: Integrate with async/await
5. **Effect Constraints**: Require certain effects for generics

## References

- Koka: https://koka-lang.org/
- Eff: https://www.eff-lang.org/
- Rust Effects RFC: https://github.com/rust-lang/rfcs/pull/2721
- Algebraic Effects for the Masses: https://www.youtube.com/watch?v=8zEHWtrHCDs

---

**Next Steps**: Implement effect keyword and parsing (Phase 1)
