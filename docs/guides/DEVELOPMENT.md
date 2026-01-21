# ZULON Development Guide

## 📚 Overview

This guide helps new contributors understand the ZULON codebase, set up their development environment, and start contributing effectively.

## 🚀 Quick Start

### Prerequisites

```bash
# Required
- Rust 1.70+ (stable)
- LLVM 15.0+
- Git

# Recommended
- VS Code with ZULON extension
- clang for native compilation
- Node.js (for running tests)
```

### First Steps

1. **Clone Repository**
   ```bash
   git clone https://github.com/zulon-lang/zulon.git
   cd zulon
   ```

2. **Build Project**
   ```bash
   cargo build --workspace
   ```

3. **Run Tests**
   ```bash
   cargo test --workspace
   ```

4. **Install Tools**
   ```bash
   cargo install --path zulon-tools-yan
   ```

## 🏗️ Project Structure

### Crate Organization

```
zulon/
├── crates/                 # All workspace crates
│   ├── zulon-parser/       # Lexer and parser
│   ├── zulon-typeck/       # Type checker
│   ├── zulon-hir/         # High-level IR
│   ├── zulon-mir/         # Mid-level IR
│   ├── zulon-lir/         # Low-level IR
│   ├── zulon-codegen-llvm/ # LLVM backend
│   ├── zulon-runtime-actor/   # Actor model runtime
│   ├── zulon-async-runtime/  # Async runtime
│   ├── zulon-runtime-net/     # Network stack
│   ├── zulon-std-core/     # Standard library core
│   └── zulon-std-std/     # Standard library
├── examples/              # Example programs
└── docs/                 # Documentation
    ├── guides/          # User guides (Quick Start, etc.)
    ├── design/          # Design documents
    ├── history/         # Historical implementation reports
    ├── reports/         # Historical session reports
    └── api/            # API documentation
```

### Module Organization within Crates

```
crates/<crate-name>/src/
├── lib.rs              # Public API exports
├── mod.rs              # Internal module organization (if needed)
└── modules/            # Implementation modules
    ├── *.rs             # Module files
    └── tests/          # Unit tests
```

## 🔧 Development Workflow

### Building the Project

```bash
# Build entire workspace
cargo build --workspace

# Build specific crate
cargo build -p zulon-parser
cargo build -p zulon-typeck

# Build with release optimizations
cargo build --workspace --release
```

### Running Tests

```bash
# Run all tests
cargo test --workspace

# Run specific crate tests
cargo test -p zulon-typeck
cargo test -p zulon-runtime-actor

# Run tests with output
cargo test --workspace -- --nocapture

# Run tests in release mode
cargo test --workspace --release
```

### Using the YAN Tool

```bash
# Build and run a ZULON file
yan run examples/hello.zl

# Build only
yan build examples/hello.zl

# Test a file
yan test examples/hello.zl

# Create new project
yan new my_project
```

### Code Quality

```bash
# Run clippy linter
cargo clippy --workspace -- -D warnings

# Format code
cargo fmt --workspace

# Check formatting
cargo fmt --workspace -- --check

# Fix issues
cargo clippy --workspace --fix
```

## 📖 Key Components

### Compiler Pipeline

1. **Parser** (`zulon-parser`)
   - Tokenizes source code
   - Builds AST from tokens
   - Handles syntax errors

2. **Type Checker** (`zulon-typeck`)
   - Validates types
   - Type inference
   - Trait system

3. **IR Pipeline**
   - HIR (High-level IR) - `zulon-hir`
   - MIR (Mid-level IR) - `zulon-mir`
   - LIR (Low-level IR) - `zulon-lir`

4. **Code Generation** (`zulon-codegen-llvm`)
   - Generates LLVM IR from LIR
   - Produces object files
   - Creates executables

### Runtime System

1. **Core Runtime** (`zulon-runtime-core`)
   - Memory management (ARC, Weak)
   - I/O operations
   - Effect system

2. **Async Runtime** (`zulon-async-runtime`)
   - Task scheduling
   - Thread pool
   - Event loop abstraction

3. **Actor Runtime** (`zulon-runtime-actor`)
   - Actor management
   - Message passing
   - Error handling

4. **Network Stack** (`zulon-runtime-net`)
   - TCP/UDP sockets
   - DNS resolution
   - Async I/O support

### Standard Library

1. **Core** (`zulon-std-core`)
   - Primitives (types, traits)
   - Collections (Vec, HashMap, HashSet, VecDeque)
   - Memory management (Arc, Weak)
   - Result type

2. **Standard** (`zulon-std-std`)
   - I/O operations
   - String utilities
   - File operations

## 🧪 Adding New Features

### Feature Development Process

1. **Design Phase**
   - Write specification in design document
   - Update architecture documentation if needed
   - Discuss with team

2. **Implementation Phase**
   - Follow existing code patterns
   - Write tests alongside implementation
   - Ensure zero compiler warnings

3. **Testing Phase**
   - Unit tests for new functionality
   - Integration tests if needed
   - Update examples

4. **Documentation Phase**
   - Update user-facing guides
   - Add API documentation if public API changed
   - Update examples

### Example: Adding a New Function

```rust
// 1. Implement the function
pub fn new_feature() -> i32 {
    42
}

// 2. Add tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_feature() {
        assert_eq!(new_feature(), 42);
    }
}

// 3. Update lib.rs exports
pub use self::new_feature;
```

## 📝 Coding Standards

### Code Style

- **Naming Conventions**
  - Types: `PascalCase` (e.g., `ActorId`, `TaskState`)
  - Functions: `snake_case` (e.g., `spawn_actor`, `process_messages`)
  - Constants: `SCREAMING_SNAKE_CASE` (e.g., `MAX_BUFFER_SIZE`)
  - Modules: `snake_case` (e.g., `mod message_handling`)

- **Formatting**
  - Use `cargo fmt` to format code
  - Maximum line length: 100 characters
  - Indent with 4 spaces

- **Comments**
  - Public API: Use `///` for module-level documentation
  - Functions: Use `///` for public functions
  - Complex logic: Add explanatory comments
  - Keep comments concise and relevant

### Error Handling

- Use `Result<T, E>` for fallible operations
- Use `thiserror::Error` for custom error types
- Provide clear error messages
- Handle errors appropriately, don't silence them

### Testing

- Write unit tests for all non-trivial functions
- Aim for >80% code coverage
- Use descriptive test names: `test_actor_lifecycle`, `test_message_sending`
- Test both success and failure cases

## 🐛 Debugging

### Common Issues

1. **Build Errors**
   - Ensure workspace builds cleanly
   - Check for dependency issues
   - Verify LLVM is installed

2. **Test Failures**
   - Run tests individually: `cargo test -p zulon-typeck`
   - Check for test infrastructure issues
   - Verify test environment

3. **Runtime Panics**
   - Add logging to identify panic locations
   - Use `RUST_BACKTRACE=1` for detailed backtraces
   - Check for unwraps or array bounds errors

### Debug Tools

```bash
# Enable debug logging
RUST_LOG=debug cargo run

# Run with backtrace
RUST_BACKTRACE=full cargo run

# Use LLDB or GDB
lldb -- cargo run
rust-lldb -- cargo run
```

## 🔍 Understanding the Codebase

### Reading the Code

1. **Start with High-Level Documents**
   - Read [ARCHITECTURE.md](ARCHITECTURE.md) for overview
   - Read [DOCS_INDEX.md](DOCS_INDEX.md) for navigation

2. **Compiler Pipeline**
   - Parser → HIR → MIR → LIR → LLVM
   - Follow the data transformations

3. **Runtime System**
   - Understand actor and async runtime architecture
   - See how they interact with type system

### Finding Related Code

Use these patterns:
- **File search**: `grep -r "trait_name" crates/`
- **Symbol search**: Find where types/functions are defined
- **Test coverage**: See what code is tested
- **Git history**: `git log --oneline --grep "keyword"`

## 📚 Documentation Writing

### When to Write Documentation

1. **Public API Changes**
   - Always document new public types and functions
   - Add examples for complex APIs
   - Update relevant guides

2. **Breaking Changes**
   - Document migration guide
   - Update changelog
   - Mark deprecated features

3. **New Features**
   - Add feature documentation to guides
   - Provide examples
   - Update QUICK_START_GUIDE.md

### Documentation Location

- **User Guides**: `docs/guides/`
- **Design Docs**: `docs/design/`
- **API Docs**: `docs/api/`
- **Reference**: Keep important references in docs root

## 🎯 Contributing Effectively

### First Contribution

Start with simple tasks:
1. Fix typos in documentation
2. Improve error messages
3. Add missing examples
4. Write tests for uncovered code

### Growing into Core Components

After understanding basics:
1. Contribute to type system
2. Improve parser error messages
3. Add runtime optimizations
4. Enhance standard library

### Advanced Contributions

- Add new language features
- Improve compiler optimizations
- Implement new backend targets
- Design and implement new runtime features

## 📞 Getting Help

### Resources

1. **Documentation**
   - [DOCS_INDEX.md](DOCS_INDEX.md) - Complete documentation index
   - [ARCHITECTURE.md](ARCHITECTURE.md) - System architecture
   - [COMPILATION_GUIDE.md](guides/COMPILATION_GUIDE.md) - Build guide

2. **Existing Issues**
   - Check GitHub Issues: https://github.com/zulon-lang/zulon/issues
   - Search existing discussions

3. **Community**
   - Discord: https://discord.gg/zulon
   - Ask questions in relevant channels

### Asking Good Questions

```
# Bad Question
How do I fix the build?

# Good Question
I'm seeing a build error when running `cargo build`:
[error message]
Steps I've tried:
1. `cargo clean`
2. `cargo build`
Environment: macOS, Rust 1.92.0, LLVM 15.0
```

## 🔧 Development Tips

### Productivity Tips

1. **Use IDE Features**
   - Code navigation (Cmd+Click for symbols)
   - Auto-completion
   - Inline errors and warnings

2. **Script Repetitive Tasks**
   - Create shell scripts for common workflows
   - Use cargo aliases: `alias t="test --workspace"`

3. **Parallel Builds**
   - Use `cargo build --jobs <N>` to speed up workspace builds

4. **Incremental Development**
   - Test small changes frequently
   - Use `cargo check` for faster syntax checking

### Workspace Management

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Check for outdated crates
cargo outdated
```

## 📊 Measuring Success

### Key Metrics

- **Build Time**: Workspace builds in <10 seconds
- **Test Coverage**: Aim for >80%
- **Zero Warnings**: `cargo clippy -- -D warnings` should pass
- **Zero Errors**: All crates must build without errors

### Code Quality Checklist

- [ ] Code follows naming conventions
- [ ] Code is formatted with `cargo fmt`
- [ ] No clippy warnings
- [ ] All public APIs documented
- [ ] Tests for non-trivial code
- [ ] Examples compile and run

---

**Last Updated**: 2025-01-20
