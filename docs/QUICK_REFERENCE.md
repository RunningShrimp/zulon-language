# ZULON Compiler - Developer Quick Reference

Quick reference for common development tasks.

## Development Commands

### Building

```bash
# Build all crates
cargo build --workspace

# Build with optimizations
cargo build --workspace --release

# Build specific crate
cargo build -p zulon-parser

# Clean build artifacts
cargo clean
```

### Testing

```bash
# Run all tests
cargo test --workspace

# Run tests with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_name

# Run tests in watch mode
cargo install cargo-watch
cargo watch -x test

# Show test output
cargo test -- --show-output
```

### Linting and Formatting

```bash
# Format code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace -- -D warnings

# Fix clippy warnings
cargo clippy --workspace --fix
```

### Running the Compiler

```bash
# Compile a ZULON file
cargo run -p zulon-compiler -- example.zl

# With output file
cargo run -p zulon-compiler -- example.zl -o output

# Keep intermediate files
cargo run -p zulon-compiler -- example.zl --keep-intermediates

# Show compiler version
cargo run -p zulon-compiler -- --version
```

---

## Code Organization

### Crates

| Crate | Purpose |
|-------|---------|
| `zulon-compiler` | Compiler driver and CLI |
| `zulon-parser` | Lexer and parser |
| `zulon-hir` | High-level IR |
| `zulon-typeck` | Type checker |
| `zulon-mir` | Mid-level IR |
| `zulon-lir` | Low-level IR |
| `zulon-codegen-llvm` | LLVM code generation |

### Key Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace configuration |
| `IMPLEMENTATION_PLAN.md` | Roadmap |
| `TODOLIST.md` | Task list |
| `.claude/ralph-loop.local.md` | Development session status |

---

## Common Patterns

### Adding a Token

1. **Lexer** (`crates/zulon-lexer/src/lexer/token.rs`):
```rust
pub enum TokenKind {
    // ...
    YourNewToken,
}
```

2. **Lexer implementation** (`crates/zulon-lexer/src/lexer/mod.rs`):
```rust
"your_keyword" => TokenKind::YourNewToken,
```

### Adding AST Node

1. **AST** (`crates/zulon-parser/src/ast/mod.rs`):
```rust
pub enum ExpressionKind {
    // ...
    YourNewExpression,
}
```

2. **Parser** (`crates/zulon-parser/src/parser/mod.rs`):
```rust
fn parse_your_expression(&mut self) -> ParseResult<Expression> {
    // ...
}
```

### Adding Type

1. **Type definition** (`crates/zulon-typeck/src/ty.rs`):
```rust
pub enum TyKind {
    // ...
    YourNewType,
}
```

2. **Type checking** (`crates/zulon-typeck/src/checker.rs`):
```rust
fn check_your_expression(&mut self, expr: &Expression) -> Result<Ty> {
    // ...
}
```

---

## Debugging

### Enable Debug Logging

```bash
# Set environment variable
export RUST_LOG=debug

# Run with logging
RUST_LOG=debug cargo run -p zulon-compiler -- example.zl
```

### Print AST

```rust
// In parser tests
println!("{:#?}", ast);
```

### Print MIR

```rust
// In MIR lowering
println!("{:#?}", mir_body);
```

### Print LLVM IR

```bash
# Generate and view
cargo run -p zulon-compiler -- example.zl
cat example.ll
```

---

## Git Workflow

### Branch Naming

```
feature/feature-name
fix/issue-number-description
docs/documentation-update
refactor/component-name
test/test-description
```

### Commit Message Format

```
type: subject

body

footer
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Example:
```
feat: Add tuple field access

Implement tuple.0, tuple.1 syntax for accessing tuple
elements by numeric index.

Closes #123
```

---

## Common Issues

### "borrow checker error"

**Problem**: Borrow doesn't live long enough

**Solution**: Use `clone()` or restructure ownership
```rust
// Instead of
let stmt = self.parse_statement()?;
self.do_something(&stmt);

// Use
let stmt = self.parse_statement()?;
let stmt_clone = stmt.clone();
self.do_something(&stmt_clone);
```

### "type inference failed"

**Problem**: Can't infer type

**Solution**: Add type annotation
```rust
// Instead of
let x = self.get_value();

// Use
let x: i32 = self.get_value();
```

### "recursion limit exceeded"

**Problem**: Recursive type or function

**Solution**: Increase limit or use indirection
```rust
#[recursion_limit = "256"]
fn recursive_function() { }
```

---

## Testing Tips

### Test Organization

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_case() { }

    #[test]
    fn test_edge_case() { }

    #[test]
    fn test_error_case() { }
}
```

### Test Naming

```
test_<function>_<scenario>_<expected>

Examples:
test_parse_function_valid_input_success
test_parse_function_missing_name_error
test_type_check_mismatch_error
```

### Assert Macros

```rust
// Equality
assert_eq!(left, right);

// Pattern matching
assert!(matches!(value, Pattern::Variant));

// Errors
assert!(result.is_err());
assert_eq!(result.unwrap_err(), expected_error);

// Panics
assert!(std::panic::catch_unwind(|| {
    should_panic();
}).is_err());
```

---

## Performance

### Profiling

```bash
# Flame graph
cargo install flamegraph
cargo flamegraph --example example_name

# Time compilation
time cargo build --workspace

# Profile tests
cargo test --workspace -- --test-threads=1 --nocapture
```

### Optimization Tips

1. **Use `Arc` instead of `Rc`** for thread-safe sharing
2. **Arena allocation** for AST nodes
3. **String interning** for identifiers
4. **Incremental compilation** for faster builds

---

## Documentation

### Generate Docs

```bash
# Build documentation
cargo doc --workspace --no-deps --open

# Document private items
cargo doc --workspace --document-private-items
```

### Doc Tests

```rust
/// Example that runs as a test
///
/// ```
/// let result = add(1, 2);
/// assert_eq!(result, 3);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Run doc tests:
```bash
cargo test --workspace --doc
```

---

## CI/CD

### GitHub Actions

Workflows in `.github/workflows/`:
- `ci.yml` - Continuous integration
- `release.yml` - Release automation
- `fmt.yml` - Code formatting check

### Local CI Testing

```bash
# Run all checks
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --all -- --check
```

---

## Useful Crates

### Development

- `cargo-watch` - Watch for changes and run commands
- `cargo-edit` - Add dependencies from CLI
- `cargo-expand` - Macro expansion

### Testing

- `criterion` - Benchmarking
- `proptest` - Property-based testing
- `insta` - Snapshot testing

### Debugging

- `env_logger` - Logging
- `pretty_assertions` - Better assert output
- `debugify` - Debug information

---

## Keyboard Shortcuts (VS Code)

| Command | Shortcut |
|---------|----------|
| Go to Definition | F12 |
| Find References | Shift+F12 |
| Rename Symbol | F2 |
| Format Document | Shift+Alt+F |
| Toggle Terminal | Ctrl+` |
| Problems | Ctrl+Shift+M |

---

## Quick Help

### Find a function

```bash
# Grep codebase
grep -r "function_name" crates/

# Use ripgrep (faster)
rg "function_name" crates/
```

### Find a type

```bash
# Find type definition
rg "struct YourType" crates/
rg "enum YourType" crates/
```

### Find tests

```bash
# Find all tests
rg "#\[test\]" crates/

# Find specific test
rg "test_function_name" crates/
```

---

**Last Updated**: January 2026

**For detailed information, see:**
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [GETTING_STARTED.md](GETTING_STARTED.md)
