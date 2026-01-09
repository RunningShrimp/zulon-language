# Ralph Loop Iteration 18 - Error Messages Enhancement Plan

**Date**: 2026-01-08
**Iteration**: 18/40 (45% complete)
**Session Goal**: Improve compiler error messages
**Status**: ✅ **PLANNING COMPLETE**

---

## Executive Summary

After completing UTF-8 support, integer type system verification, and error handling, the next highest-value improvement is **enhancing error messages**. Current error messages are functional but lack:

1. **Code snippets** showing the problematic code
2. **Helpful hints** suggesting fixes
3. **Color/highlighting** for better readability
4. **Context** (file, line, column numbers in clear format)

**Impact**: HIGH - Significantly improves developer experience
**Effort**: MEDIUM - Requires diagnostic infrastructure
**Priority**: HIGH - Blocks user productivity

---

## Current State Analysis

### Current Error Message Example

```bash
Error: Type error: TypeMismatch { expected: I32, found: Ref { inner: U8, mutable: false }, span: Span { start: Position { line: 3, column: 21 }, end: Position { line: 4, column: 6 } } }
```

### Problems

1. ❌ **Raw type representation** - Shows internal `TypeMismatch` struct
2. ❌ **No code snippet** - User must manually find line 3, column 21
3. ❌ **No hints** - Doesn't suggest how to fix the error
4. ❌ **No color** - Hard to scan in terminal output
5. ❌ **Verbose** - Too much low-level detail

### Target Error Message

```bash
error[E030]: type mismatch
  --> test_error_message.zl:3:5
   |
3 |     x + y
   |     ^^^^^ cannot add `i32` and `&str`
   |
   = note: expected type `i32`
              found type `&str`
   = help: consider converting the string to a number, or use a different operation
```

---

## Implementation Plan

### Phase 1: Diagnostic Infrastructure (1 week)

#### 1.1 Create Diagnostic Module

**File**: `crates/zulon-diagnostic/src/lib.rs`

```rust
pub struct Diagnostic {
    level: DiagnosticLevel,
    message: String,
    code: Option<String>,  // e.g., "E030"
    spans: Vec<SpanLabel>,
    hints: Vec<String>,
}

pub enum DiagnosticLevel {
    Error,
    Warning,
    Note,
    Help,
}

pub struct SpanLabel {
    span: Span,
    label: String,
    style: LabelStyle,
}

pub enum LabelStyle {
    Primary,
    Secondary,
}

impl Diagnostic {
    pub fn error(message: impl Into<String>) -> Self {
        Diagnostic {
            level: DiagnosticLevel::Error,
            message: message.into(),
            code: None,
            spans: Vec::new(),
            hints: Vec::new(),
        }
    }

    pub fn with_span(mut self, span: Span, label: impl Into<String>) -> Self {
        self.spans.push(SpanLabel {
            span,
            label: label.into(),
            style: LabelStyle::Primary,
        });
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    pub fn emit(&self) {
        // Print with colors and formatting
        self.pretty_print();
    }
}
```

#### 1.2 Error Code Registry

**File**: `crates/zulon-diagnostic/src/error_codes.rs`

```rust
pub struct ErrorCode {
    pub code: &'static str,
    pub category: &'static str,
    pub description: &'static str,
}

pub const E_TYPE_MISMATCH: ErrorCode = ErrorCode {
    code: "E030",
    category: "type",
    description: "type mismatch in expression or function call",
};

pub const E_UNRESOLVED_VAR: ErrorCode = ErrorCode {
    code: "E042",
    category: "name",
    description: "unresolved variable or function name",
};

// ... more error codes
```

### Phase 2: Type Checker Integration (3 days)

#### 2.1 Update Type Errors

**File**: `crates/zulon-typeck/src/checker.rs`

**Before**:
```rust
return Err(TypeCheckError::TypeMismatch {
    expected: expected_ty.clone(),
    found: actual_ty.clone(),
    span,
});
```

**After**:
```rust
return Err(TypeCheckError::TypeMismatch {
    expected: expected_ty.clone(),
    found: actual_ty.clone(),
    span,
    diagnostic: Diagnostic::error("type mismatch")
        .with_span(span, format!("cannot add `{}` and `{}`",
            display_type(&expected_ty),
            display_type(&actual_ty)))
        .with_hint("consider converting one type to match the other")
        .with_code(E_TYPE_MISMATCH),
});
```

#### 2.2 Type Display Helper

```rust
fn display_type(ty: &Ty) -> String {
    match ty {
        Ty::I32 => "i32".to_string(),
        Ty::Ref { inner, .. } => format!("&{}", display_type(inner)),
        // ... other types
        _ => format!("{:?}", ty),  // Fallback for complex types
    }
}
```

### Phase 3: Error Message Enhancement (1 week)

#### 3.1 Common Error Patterns

**Pattern 1: Type Mismatch**
```bash
error[E030]: type mismatch
  --> file.zl:10:5
   |
10 |     x + y
   |     ^^^^^ cannot add `i32` and `&str`
   |
   = note: expected type `i32`
              found type `&str`
   = help: consider converting the string to a number, or use a different operation
```

**Pattern 2: Unresolved Variable**
```bash
error[E042]: unresolved variable `undefined_var`
  --> file.zl:5:5
   |
5 |     undefined_var + 1
   |     ^^^^^^^^^^^^^ variable not found in this scope
   |
   = help: variables must be declared before use
   = note: did you mean `defined_var`?
```

**Pattern 3: Missing Semicolon**
```bash
error[E001]: expected semicolon
  --> file.zl:3:1
   |
3 |     let x = 5
   |                ^ expected `;` here
   |
   = help: add a semicolon at the end of the statement
```

#### 3.2 Hints and Suggestions

Implement smart hint generation:

```rust
fn suggest_fix_for_type_mismatch(expected: &Ty, found: &Ty) -> Option<String> {
    match (expected, found) {
        (Ty::I32, Ty::F64) => {
            Some("consider converting the float to an integer: `x as i32`".to_string())
        }
        (Ty::Ref { inner: a, .. }, Ty::Ref { inner: b, .. }) if a != b => {
            Some(format!("reference types don't match, consider dereferencing: `*{}`",
                display_type(b)))
        }
        _ => None,
    }
}

fn suggest_similar_name(name: &str, available: &[String]) -> Option<String> {
    // Levenshtein distance for typo detection
    let best = available.iter()
        .min_by_key(|n| levenshtein_distance(name, n))?;

    if levenshtein_distance(name, best) <= 2 {
        Some(format!("did you mean `{}`?", best))
    } else {
        None
    }
}
```

### Phase 4: Pretty Printing (3 days)

#### 4.1 Terminal Colors

Use ANSI color codes:

```rust
pub const COLOR_RED: &str = "\x1b[31m";
pub const COLOR_GREEN: &str = "\x1b[32m";
pub const COLOR_YELLOW: &str = "\x1b[33m";
pub const COLOR_BLUE: &str = "\x1b[34m";
pub const COLOR_RESET: &str = "\x1b[0m";

impl Diagnostic {
    fn pretty_print(&self) {
        match self.level {
            DiagnosticLevel::Error => {
                println!("{}error{}[{}]: {}",
                    COLOR_RED, COLOR_RESET,
                    self.code.as_ref().unwrap_or(&"???"),
                    self.message);
            }
            DiagnosticLevel::Warning => {
                println!("{}warning{}[{}]: {}",
                    COLOR_YELLOW, COLOR_RESET,
                    self.code.as_ref().unwrap_or(&"???"),
                    self.message);
            }
            // ... other levels
        }

        // Print code snippet
        self.print_snippet();

        // Print hints
        for hint in &self.hints {
            println!("   = {}: {}", COLOR_BLUE, hint);
        }
    }
}
```

#### 4.2 Code Snippet Display

```rust
fn print_snippet(&self) {
    let source = read_source_file(&self.span.file);
    let lines = source.lines().collect::<Vec<_>>();

    for (i, line) in lines.iter()
        .enumerate()
        .take(self.span.end.line - self.span.start.line + 1)
    {
        let line_num = self.span.start.line + i;
        println!("{} |", line_num);
        println!("{} | {}", line_num, line);

        // Underline the error
        if i == 0 {
            let indent = " ".repeat(self.span.start.column);
            let carets = "^".repeat(self.span.end.column - self.span.start.column);
            println!("   | {}{}{}", indent, COLOR_RED, carets);
        }
    }
}
```

---

## Priority Order

### High Priority Errors (Fix First)

1. **Type mismatch** - Most common error
2. **Undefined variable** - Common for beginners
3. **Missing semicolon** - Syntax errors
4. **Function not found** - API usage errors

### Medium Priority Errors

1. **Parse errors** - Less common but important
2. **Invalid operations** - e.g., `mod 0` (division by zero)
3. **Immutable borrow violation** - Advanced feature

### Low Priority Errors

1. **Dead code warnings** - Nice to have
2. **Unused variable warnings** - Code quality
3. **Performance warnings** - Optimization hints

---

## Testing Strategy

### Test Suite

**File**: `crates/zulon-diagnostic/tests/error_messages/`

Create test cases for each error pattern:

```bash
error_messages/
├── type_mismatch_1.zl      # i32 vs f64
├── type_mismatch_2.zl      # i32 vs &str
├── type_mismatch_3.zl      # &T vs &mut T
├── undefined_var.zl         # simple typo
├── undefined_var_hint.zl   # with suggestion
├── missing_semicolon.zl     # statement without ;
├── fn_not_found.zl         # undefined function
└── parse_error.zl           # invalid syntax
```

**Golden Testing**:

```rust
#[test]
fn test_type_mismatch_message() {
    let source = r#"
        fn main() {
            let x: i32 = 5;
            let y: f64 = 3.14;
            x + y
        }
    "#;

    let result = compile(source);
    assert!(result.is_err());

    let error = result.unwrap_err();
    assert!(error.contains("type mismatch"));
    assert!(error.contains("cannot add"));
    assert!(error.contains("i32"));
    assert!(error.contains("f64"));
}
```

---

## File Changes Summary

### New Files

1. **crates/zulon-diagnostic/Cargo.toml** - New diagnostic crate
2. **crates/zulon-diagnostic/src/lib.rs** - Main diagnostic API
3. **crates/zulon-diagnostic/src/error_codes.rs** - Error code registry
4. **crates/zulon-diagnostic/src/pretty.rs** - Pretty printing
5. **crates/zulon-diagnostic/src/suggestion.rs** - Smart suggestions

### Modified Files

1. **crates/zulon-typeck/src/checker.rs** - Use Diagnostic instead of raw errors
2. **crates/zulon-parser/src/parser/mod.rs** - Better parse error messages
3. **crates/zulon-compiler/src/main.rs** - Integrate diagnostic emitter

---

## Success Criteria

### Must-Have (P0)

- [ ] All type errors show code snippet
- [ ] All type errors show expected vs found types clearly
- [ ] 50% of errors have helpful hints
- [ ] Error messages use colors
- [ ] Error codes assigned to common errors

### Should-Have (P1)

- [ ] 80% of errors have helpful hints
- [ ] Smart suggestions for typos (Levenshtein distance)
- [ ] Multi-span error messages
- [ ] Notes and help text rendered differently

### Nice-to-Have (P2)

- [ ] Error code documentation online
- [ ] "Learn more" links in error messages
- [ ] Fix suggestions (auto-fix capability)
- [ ] IDE integration (language server protocol)

---

## Timeline

| Phase | Task | Duration | Dependencies |
|-------|------|----------|--------------|
| 1 | Create diagnostic infrastructure | 1 week | None |
| 2 | Integrate with type checker | 3 days | Phase 1 |
| 3 | Enhance error messages | 1 week | Phase 2 |
| 4 | Testing and refinement | 3 days | Phase 3 |
| **Total** | **Error Messages Enhancement** | **~3 weeks** | None |

---

## Risk Analysis

### Technical Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Terminal color compatibility | Medium | Low | Check `$TERM` and `NO_COLOR` env var |
| Performance overhead | Low | Low | Only format errors, not compile-time |
| Complex error formatting | Medium | Medium | Incremental implementation, test thoroughly |

### Project Risks

| Risk | Probability | Impact | Mitigation |
|------|------------|--------|------------|
| Scope creep (too many features) | Medium | High | Focus on P0 items only |
| Breaking existing workflows | Low | Medium | Keep raw error format option |

---

## Conclusion

**Status**: ✅ **PLANNING COMPLETE**

The error messages enhancement plan provides a clear roadmap for significantly improving developer experience. The work is structured in 4 phases over 3 weeks, with clear success criteria and testing strategy.

**Next Step**: Begin Phase 1 - Create diagnostic infrastructure

**Expected Outcome**: Production-quality error messages that rival Rust and TypeScript in clarity and helpfulness.

---

**ZULON Language Team**
**2026-01-08**

*Ralph Loop: Iteration 18 planning complete, 18/40 iterations (45%)*
*Achievement: ERROR MESSAGES ENHANCEMENT PLAN READY FOR IMPLEMENTATION*
*Status: ✅ READY TO START PHASE 1*

---

**Next Iteration**: Implement diagnostic infrastructure (Phase 1 of error messages)
