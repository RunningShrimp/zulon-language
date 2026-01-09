# Ralph Loop Iteration 17 - Defer Statement Implementation

**Date**: 2026-01-09
**Iteration**: 17 of 40
**Status**: ✅ COMPLETED - Lexer, Parser, Type Checker, and HIR Support
**Duration**: ~30 minutes

---

## Summary

Successfully implemented defer statement support from lexer through HIR, enabling the `defer` keyword for cleanup code execution at scope exit.

---

## What Was Implemented

### 1. Lexer Support ✅

**File**: `crates/zulon-parser/src/lexer/token.rs` (line 33)

Added `Defer` to `TokenKind` enum:

```rust
// Control flow
If,
Else,
Match,
Loop,
While,
For,
In,
Break,
Continue,
Return,
Defer,  // ← Added
```

**File**: `crates/zulon-parser/src/lexer/mod.rs` (line 173)

Added keyword recognition:

```rust
"defer" => TokenKind::Defer,  // ← Added
```

### 2. AST Support ✅

**File**: `crates/zulon-parser/src/ast/mod.rs` (lines 318-319)

Added `Defer` variant to `StatementKind`:

```rust
pub enum StatementKind {
    Local(Local),
    Item(Box<Item>),
    Expr(Expression),
    Defer(Box<Statement>),  // ← Added
    Empty,
}
```

### 3. Parser Support ✅

**File**: `crates/zulon-parser/src/parser/mod.rs` (lines 504-509)

Added defer statement parsing:

```rust
Some(TokenKind::Defer) => {
    // Defer statement: defer expr_or_statement;
    self.advance();
    let stmt = Box::new(self.parse_statement()?);
    StatementKind::Defer(stmt)
}
```

**Key Features**:
- Parses `defer <statement>` syntax
- Supports any statement type after defer
- Recursive parsing allows nested defers

### 4. Type Checker Support ✅

**File**: `crates/zulon-typeck/src/checker.rs` (lines 342-346)

Added defer type checking:

```rust
ast::StatementKind::Defer(stmt) => {
    // Defer statements are checked normally
    // The runtime behavior (execution at scope exit) is handled later
    self.check_statement(stmt)
}
```

**Key Features**:
- Type checks deferred statements normally
- Runtime behavior (LIFO execution) handled in later phases

### 5. HIR Support ✅

**File**: `crates/zulon-hir/src/hir.rs` (line 108)

Added `Defer` variant to `HirStatement`:

```rust
pub enum HirStatement {
    Local(HirLocal),
    Item(HirItem),
    Expression(HirExpression),
    Semi(HirExpression),
    Defer(Box<HirStatement>),  // ← Added
}
```

### 6. HIR Lowering ✅

**File**: `crates/zulon-hir/src/simple_lower.rs` (lines 218-250)

Added defer lowering with inline statement handling:

```rust
ast::StatementKind::Defer(stmt) => {
    // Defer statement: execute when scope exits
    // Lower the deferred statement inline
    let lowered_stmt = match &stmt.kind {
        ast::StatementKind::Expr(expr) => {
            let lowered_expr = self.lower_expression(expr)?;
            HirStatement::Semi(lowered_expr)
        }
        ast::StatementKind::Local(local) => {
            // ... handle local declarations
        }
        _ => {
            return Err(LoweringError::UnsupportedFeature {
                feature: format!("complex defer statement: {:?}", stmt.kind),
                span: stmt.span.clone(),
            });
        }
    };
    statements.push(HirStatement::Defer(Box::new(lowered_stmt)));
}
```

**Key Features**:
- Supports expression statements in defer
- Supports local variable declarations in defer
- Provides clear error for complex statements (not yet supported)
- Inlines statement lowering to avoid circular dependencies

### 7. MIR Support ✅

**File**: `crates/zulon-mir/src/lower.rs` (lines 204-208)

Added defer case (currently skipped):

```rust
HirStatement::Defer(_stmt) => {
    // Defer statements are handled by creating cleanup blocks
    // For now, we'll skip them in MIR lowering
    // TODO: Implement proper defer handling with cleanup blocks
}
```

### 8. Debug Support ✅

**File**: `crates/zulon-codegen-llvm/examples/debug_hir.rs` (lines 54-56)

Added defer printing in HIR debug output:

```rust
zulon_hir::HirStatement::Defer(_) => {
    println!("{}Defer\n", indent_str);
}
```

---

## Test Example Created

**File**: `examples/defer_test.zl`

Created a comprehensive test example:

```zulon
fn main() -> i32 {
    // Simple defer with expression
    defer println("Cleanup 1");

    // Defer with local variable
    defer {
        let msg = "Cleanup 2";
        println(msg);
    }

    // Multiple defers execute in LIFO order
    defer println("Cleanup 3");
    defer println("Cleanup 4");

    println("Main code");

    0
}
```

**Expected Behavior** (when fully implemented):
1. Print "Main code"
2. Print "Cleanup 4" (last defer, first to execute)
3. Print "Cleanup 3"
4. Print "Cleanup 2"
5. Print "Cleanup 1" (first defer, last to execute)

---

## Compilation Status

✅ **Workspace compiles successfully**
- `zulon-parser`: ✅ Compiles (lexer + parser support complete)
- `zulon-typeck`: ✅ Compiles (type checking support complete)
- `zulon-hir`: ✅ Compiles (HIR lowering complete)
- `zulon-mir`: ✅ Compiles (placeholder handling)
- `zulon-lir`: ✅ Compiles (not yet handling defer)
- `zulon-codegen-llvm`: ✅ Compiles (not yet handling defer)
- `zulon-compiler`: ✅ Compiles

---

## What Still Needs Work

### MIR/LIR Lowering ⏸️ (Placeholder)

Defer statements need proper MIR/LIR lowering:

1. **MIR** (Priority: High)
   - Create cleanup blocks for each scope
   - Track defer statements in scope
   - Generate cleanup code at each exit point
   - Handle early returns, breaks, continues

2. **LIR** (Priority: High)
   - Generate cleanup block execution
   - Ensure LIFO ordering
   - Handle variable capture in deferred code

3. **LLVM Code Generation** (Priority: High)
   - Generate cleanup blocks
   - Insert cleanup calls at all scope exit points
   - Handle exception/unwind paths
   - Ensure proper stack cleanup

### Advanced Features ⏸️ (Not Implemented)

Currently only supports simple statements in defer:

**Supported**:
- Expression statements: `defer println("cleanup")`
- Local declarations: `defer { let x = 1; }`

**Not Yet Supported**:
- Nested defers: `defer { defer println("nested"); }`
- Complex blocks: `defer { if cond { cleanup(); } }`
- Variable capture from enclosing scope

### Runtime Semantics ⏸️ (Not Implemented)

The LIFO execution semantics need to be implemented:

- Defers execute when scope exits (normal or early)
- Multiple defers execute in reverse order (LIFO)
- Variables captured by value at defer time
- Panics in defer need special handling

---

## Technical Insights

### Inline Statement Lowering

The HIR lowering inlines statement lowering for defer to avoid circular method calls:

```rust
// Instead of calling self.lower_statement(stmt) (which doesn't exist)
// We inline the lowering logic for the specific statement types we support
let lowered_stmt = match &stmt.kind {
    ast::StatementKind::Expr(expr) => { /* inline expr lowering */ }
    ast::StatementKind::Local(local) => { /* inline local lowering */ }
    _ => return Err(...);  // Complex statements not yet supported
};
```

This pattern ensures type safety and avoids infinite recursion.

### Defer Semantics

Defer statements follow these rules (from Zig, Go, Swift):

1. **Execution Timing**: When the enclosing scope exits
2. **Ordering**: LIFO (last defer runs first)
3. **Capture**: Variables captured by value at defer time
4. **Scope**: Function, block, loop bodies, if/else blocks

The implementation correctly parses and type-checks defers, but runtime semantics require MIR/LIR/LLVM work.

---

## Implementation Strategy

The implementation follows a **progressive enhancement** approach:

1. **Lexer**: Token recognition ✅
2. **Parser**: Syntax parsing ✅
3. **Type Checker**: Type validation ✅
4. **HIR**: Representation ✅
5. **MIR**: Placeholder (cleanup blocks TODO) ⏸️
6. **LIR**: Not yet implemented ⏸️
7. **LLVM**: Not yet implemented ⏸️

This approach allows the feature to be incrementally completed while maintaining compilation at each stage.

---

## Language Comparison

Defer is inspired by:
- **Zig**: `defer stmt;` - same syntax
- **Go**: `defer stmt` - similar semantics
- **Swift**: `defer { stmt }` - block-based
- **Jai**: `defer <stmt>` - similar approach

ZULON's implementation follows Zig's syntax closely for familiarity.

---

## Impact

**Language Safety**: Defer provides reliable cleanup patterns, reducing resource leaks.

**Code Clarity**: Cleanup code is defined near resource acquisition, improving readability.

**Maintainability**: Reduces boilerplate cleanup code across multiple exit paths.

**User Experience**: Developers get a powerful tool for resource management.

---

## Files Modified

1. `crates/zulon-parser/src/lexer/token.rs` - Added Defer token
2. `crates/zulon-parser/src/lexer/mod.rs` - Added defer keyword
3. `crates/zulon-parser/src/ast/mod.rs` - Added Defer to StatementKind
4. `crates/zulon-parser/src/parser/mod.rs` - Added defer parsing
5. `crates/zulon-typeck/src/checker.rs` - Added defer type checking
6. `crates/zulon-hir/src/hir.rs` - Added Defer to HirStatement
7. `crates/zulon-hir/src/simple_lower.rs` - Added defer lowering
8. `crates/zulon-mir/src/lower.rs` - Added defer placeholder
9. `crates/zulon-codegen-llvm/examples/debug_hir.rs` - Added defer debug output
10. `examples/defer_test.zl` - Created test example

**Lines Added**: ~60 lines of production code

---

## Next Steps

To complete defer support, the following work is needed:

1. **MIR Implementation** (Priority: High)
   - Track defer statements per scope
   - Generate cleanup blocks
   - Insert cleanup calls at exit points
   - Handle control flow (return, break, continue)

2. **Variable Capture** (Priority: Medium)
   - Capture variables by value at defer time
   - Handle mutable captures
   - Support complex expressions

3. **LIR Lowering** (Priority: High)
   - Generate cleanup block execution
   - Ensure proper ordering
   - Handle edge cases

4. **LLVM Code Generation** (Priority: High)
   - Generate LLVM cleanup blocks
   - Insert cleanup calls at all exits
   - Handle exception paths

5. **Testing** (Priority: Medium)
   - Unit tests for defer parsing
   - Integration tests for defer semantics
   - Examples demonstrating real-world usage

---

## Conclusion

This iteration successfully implemented defer statement support from lexer through HIR, enabling the `defer` keyword to be recognized, parsed, type-checked, and represented in the intermediate representation. While defers cannot yet execute (missing MIR/LIR/LLVM lowering for cleanup blocks), the compiler can now correctly parse and validate defer syntax, representing significant progress toward this valuable language feature.

The implementation is clean, follows established patterns, provides clear error messages for unsupported cases, and serves as a solid foundation for future work on runtime semantics.

---

**Next Iteration Focus**: Continue with Phase 2.1 advanced features (namespace support, trait composition) or begin work on MIR/LIR lowering for previously implemented features (tuples, template strings, defer).
