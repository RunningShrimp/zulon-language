# Contributing to ZULON

Thank you for your interest in contributing to ZULON! This document provides guidelines and instructions for contributors.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Development Workflow](#development-workflow)
3. [Code Style](#code-style)
4. [Testing](#testing)
5. [Documentation](#documentation)
6. [Submitting Changes](#submitting-changes)
7. [Getting Help](#getting-help)

---

## Quick Start

### Prerequisites

- Rust 1.70 or later
- Git
- LLVM 15+ (for testing code generation)
- Basic understanding of compiler design

### Setup

```bash
# Fork the repository on GitHub
# Clone your fork
git clone https://github.com/YOUR_USERNAME/zulon.git
cd zulon

# Add upstream remote
git remote add upstream https://github.com/zulon-lang/zulon.git

# Install development tools
cargo install cargo-watch
cargo install cargo-edit

# Build the project
cargo build

# Run tests
cargo test --workspace
```

---

## Development Workflow

### 1. Choose an Issue

Look for issues labeled:
- `good first issue` - Good for newcomers
- `help wanted` - Community contributions welcome
- `enhancement` - Feature requests

Comment on the issue to let others know you're working on it.

### 2. Create a Branch

```bash
# Update your main branch
git checkout main
git pull upstream main

# Create a feature branch
git checkout -b feature/your-feature-name

# Or a bugfix branch
git checkout -b fix/issue-number-description
```

### 3. Make Changes

#### Code Organization

The compiler is organized into crates:

```
crates/
├── zulon-compiler/         # Compiler driver (CLI)
├── zulon-parser/           # Lexer and parser
├── zulon-hir/              # High-level IR
├── zulon-typeck/           # Type checker
├── zulon-mir/              # Mid-level IR
├── zulon-lir/              # Low-level IR
├── zulon-codegen-llvm/     # LLVM code generation
└── ...
```

#### Adding a Feature

When adding a new language feature, you typically need to modify:

1. **Parser** (`zulon-parser`): Add token kinds and AST nodes
2. **HIR** (`zulon-hir`): Add HIR representation
3. **Type Checker** (`zulon-typeck`): Add type checking rules
4. **MIR** (`zulon-mir`): Add MIR lowering
5. **LIR** (`zulon-lir`): Add LIR lowering
6. **Code Gen** (`zulon-codegen-llvm`): Add code generation

See [ARCHITECTURE.md](ARCHITECTURE.md) for details.

### 4. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run tests for a specific crate
cargo test -p zulon-parser

# Run with output
cargo test --workspace -- --nocapture

# Run a specific test
cargo test test_parse_function
```

### 5. Commit Your Changes

```bash
# Stage changes
git add .

# Commit with a clear message
git commit -m "Add support for while loops

- Implement while loop parsing
- Add MIR lowering for while loops
- Add tests for while loops

Fixes #123"
```

#### Commit Message Format

```
<type>: <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Example**:
```
feat: Add tuple field access syntax

Implement tuple.field syntax for accessing tuple elements
by numeric index (tuple.0, tuple.1, etc.).

This distinguishes tuple field access from struct field access
in the parser.

Closes #456
```

### 6. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

#### Pull Request Checklist

- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] Commit messages are clear
- [ ] PR description explains changes

---

## Code Style

### Rust Conventions

Follow standard Rust conventions:
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Prefer `self` over `&self` when ownership is needed
- Use `Result<T, E>` for error handling
- Use `Arc` for shared ownership, not `Rc`

### Naming Conventions

```rust
// Types: PascalCase
struct MyStruct { }
enum MyEnum { }
type MyType = i32;

// Functions and methods: snake_case
fn my_function() { }
impl MyStruct {
    fn my_method(&self) { }
}

// Constants: SCREAMING_SNAKE_CASE
const MAX_SIZE: usize = 100;

// Variables: snake_case
let my_variable = 42;

// Acronyms: keep uppercase
fn parse_ast() { }  // not parse_Ast
fn generate_llvm_ir() { }  // not generate_llvm_ir
```

### Error Handling

```rust
// Use Result for fallible operations
pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
    // ...
}

// Use ? for error propagation
pub fn compile(&mut self) -> Result<()> {
    let ast = self.parse()?;
    let hir = self.lower_to_hir(ast)?;
    // ...
}

// Use thiserror for error types
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("type error: {0}")]
    Type(#[from] TypeError),
}
```

### Documentation

```rust
/// Parses a ZULON expression from the token stream.
///
/// # Arguments
///
/// * `tokens` - The tokens to parse
///
/// # Returns
///
/// * `Ok(Expression)` - Successfully parsed expression
/// * `Err(ParseError)` - Parse error with location information
///
/// # Examples
///
/// ```
/// use zulon_parser::Parser;
///
/// let mut parser = Parser::new("let x = 42");
/// let expr = parser.parse_expression().unwrap();
/// ```
pub fn parse_expression(&mut self) -> Result<Expression, ParseError> {
    // ...
}
```

---

## Testing

### Unit Tests

Test individual functions and modules:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_integer_literal() {
        let source = "42";
        let mut parser = Parser::new(source);
        let expr = parser.parse_expression().unwrap();
        assert!(matches!(expr, Expression::Literal(42)));
    }

    #[test]
    fn test_parse_tuple_field_access() {
        let source = "tuple.0";
        let mut parser = Parser::new(source);
        let expr = parser.parse_expression().unwrap();
        // ... assertions
    }
}
```

### Integration Tests

Test the full compilation pipeline:

```rust
// tests/integration_test.rs
#[test]
fn test_compile_hello_world() {
    let source = r#"
        extern fn println(s: string);

        fn main() -> i32 {
            println("Hello, World!");
            0
        }
    "#;

    let result = compile_to_llvm_ir(source);
    assert!(result.is_ok());
}
```

### Golden Tests

Test against expected outputs:

```rust
#[test]
fn test_golden_example() {
    let input = std::fs::read_to_string("test_data/input/test.zl").unwrap();
    let expected = std::fs::read_to_string("test_data/expected/test.ll").unwrap();

    let actual = compile_to_llvm_ir(&input).unwrap();
    assert_eq!(expected, actual);
}
```

### Running Tests

```bash
# All tests
cargo test --workspace

# Specific test
cargo test test_parse_function

# Watch mode (re-run on changes)
cargo watch -x test

# Test with output
cargo test -- --nocapture

# Test specific crate
cargo test -p zulon-parser
```

---

## Documentation

### Code Documentation

- Document all public APIs
- Include examples for non-trivial functions
- Use `///` for documentation comments

### User Documentation

Update user-facing docs when adding features:
- [GETTING_STARTED.md](GETTING_STARTED.md) - Getting started guide
- [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) - Language reference
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Troubleshooting guide

### Examples

Add examples demonstrating new features:

```bash
# Add example file
examples/your_feature_demo.zl

# Update examples README
examples/README_EN.md
```

### Internal Documentation

Update design docs for significant changes:
- [ARCHITECTURE.md](ARCHITECTURE.md) - Compiler architecture
- [IMPLEMENTATION_PLAN.md](../IMPLEMENTATION_PLAN.md) - Implementation roadmap

---

## Submitting Changes

### Pull Request Process

1. **Description**: Clearly describe what you changed and why
2. **Link Issues**: Reference related issues (e.g., "Fixes #123")
3. **Screenshots**: Include screenshots for UI changes (if applicable)
4. **Tests**: Show that tests pass
5. **Docs**: List documentation changes

### Review Process

1. **Automated Checks**: CI runs tests and linting
2. **Code Review**: Maintainers review your code
3. **Address Feedback**: Make requested changes
4. **Approval**: Get approval from maintainers
5. **Merge**: Maintainers merge your PR

### After Merge

- Celebrate! 🎉
- Your contribution will be credited in the next release
- Consider helping review other PRs

---

## Getting Help

### Communication Channels

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Questions and ideas
- **Discord**: Real-time chat (link in README)

### Resources

- [ARCHITECTURE.md](ARCHITECTURE.md) - Compiler architecture
- [GETTING_STARTED.md](GETTING_STARTED.md) - User guide
- [LANGUAGE_REFERENCE.md](LANGUAGE_REFERENCE.md) - Language reference
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md) - Troubleshooting

### Asking Good Questions

When asking for help:

1. **Search first**: Check if your question was already answered
2. **Be specific**: Include code snippets and error messages
3. **Provide context**: What are you trying to do?
4. **Show effort**: What have you tried?

**Example**:
```
Hi, I'm trying to add tuple field access to the parser.
I've added token recognition in the lexer, but I'm getting
a parse error. Here's my code:

[paste code]

Error message:
[paste error]

What am I missing?
```

---

## Areas Where We Need Help

### Good First Issues

- Improve error messages
- Add more test cases
- Write documentation
- Fix typos in comments

### Larger Projects

- Template string interpolation
- Generic type instantiation
- For loop implementation
- Struct declarations
- Closures and capture

### Research Projects

- Lifetime inference
- Effect system
- Dependent types
- Compiler optimizations

---

## Code of Conduct

Be respectful, inclusive, and constructive. We're all here to build something great together.

If you experience or witness inappropriate behavior, please contact the maintainers.

---

## License

By contributing, you agree that your contributions will be licensed under the same license as the project (Apache-2.0 OR MIT).

---

**Thank you for contributing to ZULON!** 🚀

Every contribution, no matter how small, helps make ZULON better. We appreciate your time and effort.

**Questions?** Open an issue or start a discussion. We're here to help!
