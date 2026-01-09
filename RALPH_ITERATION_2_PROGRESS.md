# Ralph Loop Iteration 2: Implementation Progress

**Date**: 2026-01-08
**Iteration**: 2 / 40
**Status**: 🔨 Implementation Started

---

## Executive Summary

Started implementation of error handling system based on Iteration 1 design. **Key discovery**: The lexer and AST are already prepared! Only parser implementation is needed.

### Progress So Far

1. ✅ **Analysis Complete** (1 hour)
   - Reviewed lexer implementation
   - Reviewed AST structure
   - Reviewed parser structure
   - Identified implementation points

2. 🔨 **Implementation In Progress**
   - Next: Add parser support for throw, ?, and |

---

## Key Findings

### ✅ What's Already Done

**Lexer** (crates/zulon-parser/src/lexer/mod.rs):
- ✅ Line 186: `throw` keyword recognized
- ✅ Line 121: `?` operator recognized
- ✅ Line 631-642: `|` pipe recognized (TokenKind::Pipe)
- ✅ All required tokens already implemented!

**AST** (crates/zulon-parser/src/ast/mod.rs):
- ✅ Line 376: `ExpressionKind::Throw(Box<Expression>)` defined
- ✅ Type system already supports needed constructs

### ❌ What Needs Implementation

**Parser** (crates/zulon-parser/src/parser/mod.rs):
- ❌ No handling for `throw` statement (needs to be added to parse_statement)
- ❌ No handling for `?` operator (needs to be added to parse_postfix)
- ❌ No handling for `|` separator in function return types (needs to be added to parse_function)

---

## Implementation Plan (Revised)

Since lexer and AST are done, I can skip the lexer implementation phase entirely!

### Revised Timeline

**Original**: 2 weeks (lexer + parser + typeck + codegen)
**New Estimate**: 1.5 weeks (parser + typeck + codegen)

**Week 1** (Originally lexer + parser, now just parser):
- Day 1-2: Parser implementation (current work)
  - Add throw statement parsing
  - Add ? operator parsing
  - Add | separator parsing
  - Add 15 tests

**Week 2** (Type system + codegen):
- Day 3-5: Type system integration
- Day 6-7: Code generation

**Saved**: 2-3 days by skipping lexer work!

---

## Implementation Details

### 1. Throw Statement (parse_statement)

**Location**: `crates/zulon-parser/src/parser/mod.rs:335`

**Add to match statement**:
```rust
Some(TokenKind::Throw) => {
    let error_expr = self.parse_throw_statement()?;
    StatementKind::Expr(error_expr)
}
```

**New function**:
```rust
fn parse_throw_statement(&mut self) -> ParseResult<Expression> {
    let throw_span = self.current_span();
    self.consume(TokenKind::Throw)?;

    let error = Box::new(self.parse_expression()?);

    self.consume(TokenKind::Semicolon)?;

    Ok(Expression {
        span: throw_span,
        kind: ExpressionKind::Throw(error),
    })
}
```

### 2. Question Mark Operator (parse_postfix)

**Location**: `crates/zulon-parser/src/parser/mod.rs:623`

**Add to match statement in parse_postfix**:
```rust
// Question mark operator: expr?
Some(TokenKind::Question) => {
    self.advance();

    expr = Expression {
        span,
        kind: ExpressionKind::QuestionMark(Box::new(expr)),
    };
}
```

**AST enhancement needed**: Add `QuestionMark` variant to `ExpressionKind`:
```rust
// In ast/mod.rs
QuestionMark(Box<Expression>),  // The ? operator for error propagation
```

### 3. Pipe Separator (parse_function)

**Location**: `crates/zulon-parser/src/parser/mod.rs:217`

**Modify return type parsing**:
```rust
// Current: Option<Type>
// New: Parse Option<Type> then check for |

let return_type = if self.check(&TokenKind::Arrow) {
    self.advance();
    let success_type = self.parse_type()?;

    // Check for | separator
    if self.check(&TokenKind::Pipe) {
        self.advance();
        let error_type = self.parse_type()?;

        // Check for effects (another |)
        let mut effects = Vec::new();
        if self.check(&TokenKind::Pipe) {
            self.advance();
            // Parse effect list...
        }

        // Store both types in function metadata
        // This requires enhancing the Function struct
    }

    Some(success_type)
} else {
    None
};
```

**AST enhancement needed**: Function struct needs to store error type:
```rust
pub struct Function {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,      // Success type
    pub error_type: Option<Type>,        // NEW: Error type
    pub effects: Vec<Type>,              // NEW: Effect list
    pub body: Block,
    pub is_async: bool,
    pub is_unsafe: bool,
}
```

---

## Next Steps (Immediate)

### Right Now
1. [ ] Add `QuestionMark` to `ExpressionKind` in ast/mod.rs
2. [ ] Add `error_type` and `effects` fields to `Function` struct
3. [ ] Implement `parse_throw_statement` function
4. [ ] Add throw handling to `parse_statement`
5. [ ] Add ? handling to `parse_postfix`
6. [ ] Modify `parse_function` to handle | separator

### Testing (After Implementation)
7. [ ] Create test file: `crates/zulon-parser/src/parser/error_handling_tests.rs`
8. [ ] Add 15 unit tests:
   - 3 tests for throw statement
   - 5 tests for ? operator
   - 5 tests for | separator
   - 2 integration tests

### Verification
9. [ ] Run all parser tests (existing + new)
10. [ ] Ensure no regressions (existing 28 tests still pass)
11. [ ] Commit changes with clear message

---

## Code Changes Needed

### File 1: crates/zulon-parser/src/ast/mod.rs

**Line 376**: Already has `Throw(Box<Expression>)`

**After line 401**, add:
```rust
/// Question mark operator for error propagation
QuestionMark(Box<Expression>),
```

**Line 72-82**: Modify Function struct:
```rust
pub struct Function {
    pub name: Identifier,
    pub generics: Option<Generics>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,      // Success type
    pub error_type: Option<Type>,        // NEW: Error type
    pub effects: Vec<Type>,              // NEW: Effects
    pub body: Block,
    pub is_async: bool,
    pub is_unsafe: bool,
}
```

### File 2: crates/zulon-parser/src/parser/mod.rs

**After line 348**, in parse_statement match:
```rust
Some(TokenKind::Throw) => {
    let error_expr = self.parse_throw_statement()?;
    StatementKind::Expr(error_expr)
}
```

**New function** (after parse_local, around line 394):
```rust
fn parse_throw_statement(&mut self) -> ParseResult<Expression> {
    let throw_span = self.current_span();
    self.consume(TokenKind::Throw)?;

    let error = Box::new(self.parse_expression()?);

    self.consume(TokenKind::Semicolon)?;

    Ok(Expression {
        span: throw_span,
        kind: ExpressionKind::Throw(error),
    })
}
```

**In parse_postfix**, after line 674 (before the break):
```rust
// Question mark operator: expr?
Some(TokenKind::Question) => {
    self.advance();

    expr = Expression {
        span,
        kind: ExpressionKind::QuestionMark(Box::new(expr)),
    };
}
```

**Modify parse_function** (line 217) - major changes to return type parsing

### File 3: crates/zulon-parser/src/parser/error_handling_tests.rs

**Create new file** with 15 tests

---

## Risk Assessment

### Low Risk ✅
- Lexer: No changes needed (already working)
- AST: Minimal changes (add 2 fields + 1 variant)
- Parser: Well-understood changes, localized

### Medium Risk ⚠️
- Function struct changes may break existing code
- Need to update all Function construction sites
- May affect HIR/MIR lowering

### Mitigation
- Run all tests after each change
- Commit frequently
- Update HIR/MIR in same iteration if needed

---

## Success Criteria

### Parser Implementation
- [ ] All 15 new tests passing
- [ ] All 28 existing tests still passing
- [ ] Zero compiler warnings
- [ ] Can parse throw statements
- [ ] Can parse ? operator
- [ ] Can parse | separator

### Code Quality
- [ ] Clear code with comments
- [ ] Follows existing patterns
- [ ] No dead code
- [ ] Proper error messages

---

## Time Tracking

**Estimate**: 1-2 days for parser implementation

**Breakdown**:
- AST changes: 1 hour
- Throw statement: 1 hour
- ? operator: 1 hour
- | separator: 2 hours (complex)
- Tests: 2 hours
- Debugging: 2 hours

**Total**: ~9 hours (1-2 days)

---

## Next Session

After parser is complete, move to:
1. Type system integration (Day 3-5)
2. Code generation (Day 6-7)
3. Integration tests (Day 8-9)

---

**Document Version**: 1.0
**Author**: ZULON Language Team
**Date**: 2026-01-08
**Status**: In Progress 🔨
**Completion**: 15% (analysis complete, implementation started)
