# Contributing to ZULON

**Thank you for your interest in contributing to ZULON!** 🚀

This document provides guidelines and instructions for contributing to the ZULON programming language, including development setup, coding standards, and contribution workflow.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Coding Standards](#coding-standards)
- [Testing Guidelines](#testing-guidelines)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Community Guidelines](#community-guidelines)

---

## Quick Start

### Prerequisites

- **Rust**: 1.70 or later
- **LLVM**: 15.0 or later
- **Clang**: For linking
- **Git**: For version control

### First-Time Setup

```bash
# 1. Fork and clone the repository
git clone https://github.com/YOUR_USERNAME/zulon.git
cd zulon

# 2. Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 3. Install LLVM
# macOS: brew install llvm
# Ubuntu: sudo apt-get install llvm-15-dev
# See: https://llvm.org/docs/GettingStarted.html

# 4. Build ZULON
cargo build --release

# 5. Verify installation
./target/release/zulon-compiler --version
```

### Your First Contribution

```bash
# 1. Create a feature branch
git checkout -b feature/my-feature

# 2. Make your changes
# Edit files...

# 3. Test your changes
cargo test --workspace
cargo run --bin zulon-compiler -- examples/working/01_hello.zl

# 4. Commit your changes
git add .
git commit -m "feat: add my feature"

# 5. Push and create PR
git push origin feature/my-feature
# Then create PR on GitHub
```

---

## Development Setup

### Workspace Structure

ZULON uses a Cargo workspace with multiple crates:

```
zulon/
├── Cargo.toml                 # Workspace root
├── crates/
│   ├── zulon-parser/         # Lexer and parser
│   ├── zulon-typeck/         # Type checker
│   ├── zulon-hir/            # High-level IR
│   ├── zulon-mir/            # Mid-level IR
│   ├── zulon-lir/            # Low-level IR
│   ├── zulon-codegen-llvm/   # LLVM code generator
│   ├── zulon-runtime-core/   # Runtime system
│   ├── zulon-std-core/       # Standard library
│   ├── zulon-tools-yan/      # YAN toolchain
│   ├── zulon-macros/         # Macro system
│   ├── zulon-compiler/       # Compiler driver
│   └── zulon-diagnostic/     # Diagnostics
└── examples/
    └── working/              # Example programs
```

### Building

**Release Build** (Optimized):
```bash
cargo build --release
```

**Debug Build** (Faster compilation):
```bash
cargo build
```

**Specific Crate**:
```bash
cargo build -p zulon-parser
```

### Testing

**All Tests**:
```bash
cargo test --workspace
```

**Specific Crate**:
```bash
cargo test -p zulon-typeck
```

**With Output**:
```bash
cargo test --workspace -- --nocapture
```

### Running Examples

**Compile and Run**:
```bash
# Using compiler directly
./target/release/zulon-compiler examples/working/01_hello.zl
./examples/working/01_hello.zl

# Using YAN tool
cargo run -p zulon-tools-yan -- run examples/working/01_hello.zl
```

---

## Project Structure

### Compiler Pipeline

ZULON uses an 8-stage compilation pipeline:

```
1. Macro Expansion    - zulon-macros
2. Lexical Analysis    - zulon-parser (lexer)
3. Parsing            - zulon-parser (parser)
4. Type Checking      - zulon-typeck
5. HIR Lowering       - zulon-hir
6. MIR Lowering       - zulon-mir
7. LIR Lowering       - zulon-lir
8. LLVM Code Gen      - zulon-codegen-llvm
```

### Key Crates

**Parser** (`zulon-parser`):
- Tokenizes source code
- Parses tokens into AST
- Handles syntax errors

**Type Checker** (`zulon-typeck`):
- Type inference
- Type checking
- Generic instantiation

**HIR** (`zulon-hir`):
- High-level intermediate representation
- Type-checked AST
- Preserves high-level structure

**MIR** (`zulon-mir`):
- Mid-level intermediate representation
- Control flow graph
- Borrow checking
- Effect checking

**LIR** (`zulon-lir`):
- Low-level intermediate representation
- Memory layout
- External function linkage

**Codegen** (`zulon-codegen-llvm`):
- LLVM IR generation
- Optimization
- Assembly generation

---

## Coding Standards

### Rust Code Style

**Formatting**:
```bash
# Format all code
cargo fmt --all

# Check formatting
cargo fmt --all -- --check
```

**Linting**:
```bash
# Run linter
cargo clippy --all -- -D warnings

# Auto-fix some issues
cargo clippy --all --fix
```

### Naming Conventions

**Rust Code**:
- **Modules**: `snake_case` (e.g., `type_checker`)
- **Types**: `PascalCase` (e.g., `TypeChecker`)
- **Functions**: `snake_case` (e.g., `check_type`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SIZE`)
- **Lifetime Parameters**: Short names (`'a`, `'b`)

**ZULON Code** (Examples/Tests):
- **Files**: `snake_case.zl` (e.g., `hello_world.zl`)
- **Functions**: `snake_case` (e.g., `my_function`)
- **Types**: `PascalCase` (e.g., `MyType`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_SIZE`)

### Code Organization

**File Layout**:
```rust
// 1. Imports
use std::collections::HashMap;

// 2. Constants
const MAX_SIZE: usize = 100;

// 3. Type definitions
struct MyType {
    field: i32,
}

// 4. Trait implementations
impl MyType {
    // Methods...
}

// 5. Functions
fn my_function() -> Result<(), Error> {
    // ...
}
```

### Documentation

**Public Functions**: Must have documentation comments
```rust
/// Checks if a type is valid.
///
/// # Arguments
///
/// * `ty` - The type to check
///
/// # Returns
///
/// * `Ok(())` if the type is valid
/// * `Err(TypeError)` if the type is invalid
///
/// # Examples
///
/// ```
/// use zulon_typeck::check_type;
/// let result = check_type(&my_type);
/// assert!(result.is_ok());
/// ```
pub fn check_type(ty: &Type) -> Result<(), TypeError> {
    // ...
}
```

**Module Documentation**: Add `//!` comments at top of file
```rust
//! # Type Checker
//!
//! This module provides type checking and inference for ZULON.
//!
//! ## Overview
//!
//! The type checker uses Robinson's unification algorithm...
```

---

## Testing Guidelines

### Unit Tests

**In-Source Tests** (for small utilities):
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(add(2, 3), 5);
    }
}
```

### Integration Tests

**Integration Test Files** (`tests/` directory):
```rust
// crates/zulon-codegen-llvm/tests/integration_test.rs
use zulon_codegen_llvm::codegen;

#[test]
fn test_function_compilation() {
    let ir = generate_test_ir();
    let result = codegen(&ir);
    assert!(result.is_ok());
}
```

### Example Tests

**Working Examples** (`examples/working/`):
- Must compile without errors
- Should run successfully
- Demonstrate specific features
- Include comments explaining the feature

**Example Template**:
```zulon
// Example: Feature Name
// Description: What this example demonstrates

extern fn printf(s: &u8, ...) -> i32;

fn helper() -> i32 {
    // Helper code
    0
}

fn main() -> i32 {
    // Main example code
    printf("Feature output\n");
    0
}
```

### Test Coverage

**Goals**:
- Aim for >80% code coverage
- Test all public APIs
- Test error cases
- Test edge cases

**Checking Coverage** (optional):
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --workspace --out Html
```

---

## Commit Messages

### Format

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **test**: Adding or updating tests
- **chore**: Build process or tooling changes
- **perf**: Performance improvements

### Examples

**Feature**:
```
feat(parser): add support for variadic functions

Implemented parsing of ... syntax for variadic function
declarations. Added is_variadic field to Function struct.

Closes #123
```

**Bug Fix**:
```
fix(typeck): resolve inference loop in generic functions

Fixed issue where type inference would loop infinitely
when resolving generic function type parameters.

Fixes #456
```

**Documentation**:
```
docs(readme): update installation instructions

 clarified LLVM installation steps for macOS.
Added troubleshooting section.
```

### Best Practices

- **Separate commits**: One logical change per commit
- **Present tense**: Use "add" not "added" or "adds"
- **Imperative mood**: Use "move" not "moves" or "moving"
- **Explain why**: Not just what, but why
- **Reference issues**: Link to related issues/PRs

---

## Pull Request Process

### Before Submitting

**Checklist**:
- [ ] Code follows style guidelines (`cargo fmt`)
- [ ] Code passes linter (`cargo clippy`)
- [ ] All tests pass (`cargo test --workspace`)
- [ ] Documentation updated (if needed)
- [ ] Commit messages follow conventions
- [ ] PR description clearly describes changes

### Creating a PR

**Title Format**:
```
<type>: <short description>
```

**Description Template**:
```markdown
## Summary
Brief description of changes.

## Changes
- Bullet point for each major change
- Keep it concise

## Testing
- How you tested the changes
- Which tests were added/updated

## Related Issues
Closes #123
Related to #456
```

### Review Process

**What to Expect**:
1. **Automated Checks**: CI runs tests and lints
2. **Code Review**: Maintainers review your code
3. **Feedback**: Address review comments
4. **Approval**: Once approved, PR will be merged

**Response Time**: Typically 1-3 days

### Merging

**After Approval**:
- Squash commits if needed
- Ensure branch is up to date
- Rebase if there are conflicts
- Merge when ready

---

## Community Guidelines

### Our Pledge

We strive to:
- Be inclusive and welcoming
- Be respectful and professional
- Focus on constructive feedback
- Show empathy toward others

### Expected Behavior

**Do**:
- Use welcoming and inclusive language
- Respect differing viewpoints and experiences
- Gracefully accept constructive criticism
- Focus on what is best for the community
- Show empathy toward other community members

**Don't**:
- Use derogatory or harassing language
- Post personal attacks or insults
- Publicly or privately harass others
- Submit spam or irrelevant content

### Reporting Issues

**If You Experience Problems**:
- Email: zulon-lang@example.com
- Open issue with "CONDUCT" label
- Contact maintainers directly

**Enforcement**:
- Moderators will investigate
- Appropriate action will be taken
- Confidentiality will be maintained

---

## Getting Help

### Resources

**Documentation**:
- [README.md](README.md) - Project overview
- [EXAMPLES_INDEX.md](EXAMPLES_INDEX.md) - Example programs
- [MVP_V0.1.0_FINAL_RELEASE.md](MVP_V0.1.0_FINAL_RELEASE.md) - Release notes

**Technical Docs**:
- `docs/` directory - Detailed documentation
- Source code comments - Inline documentation
- Test files - Usage examples

### Communication

**For Questions**:
- GitHub Discussions: Ask questions
- GitHub Issues: Report bugs/request features
- Discord: [ZULON Community](https://discord.gg/zulon)

**For Contributions**:
- Pull Requests: Submit code
- Issues: Discuss changes first

---

## Development Workflow

### Typical Contribution Flow

1. **Discuss** (Optional but recommended)
   - Open issue to discuss proposed changes
   - Get feedback from maintainers
   - Avoid wasted effort

2. **Implement**
   - Create feature branch from `master`
   - Make changes following guidelines
   - Test thoroughly
   - Document changes

3. **Test**
   - Run all tests
   - Add new tests if needed
   - Verify examples work
   - Check formatting and linting

4. **Submit**
   - Push to your fork
   - Create pull request
   - Fill in PR template
   - Link related issues

5. **Review**
   - Address feedback
   - Make requested changes
   - Respond to comments
   - Iterate until approved

6. **Merge**
   - Squash if needed
   - Update branch
   - Merge to master
   - Celebrate! 🎉

---

## Recognition

**Contributors will be**:
- Listed in CONTRIBUTORS.md
- Mentioned in release notes
- Credited in significant features
- Celebrated in community updates

**All contributions matter**:
- Bug fixes
- Feature additions
- Documentation improvements
- Test enhancements
- Bug reports
- Feature requests

---

## License

By contributing, you agree that your contributions will be licensed under the **Apache License 2.0** or **MIT License** (your choice).

---

## Thank You! 🙏

We appreciate your interest in contributing to ZULON! Every contribution helps make ZULON better, whether it's a bug fix, new feature, documentation, or just reporting an issue.

**Let's build something amazing together!** 🚀

---

*Last Updated: 2026-01-09*
*ZULON v0.1.0 MVP*
*Status: Production Ready* ✅

---

**Questions?** Open an issue or start a discussion!
**Ready to contribute?** Check out [Good First Issues](https://github.com/zulon-lang/zulon/labels/good%20first%20issue)!

---

*"The best way to predict the future is to implement it."*
*- ZULON Development Team*
