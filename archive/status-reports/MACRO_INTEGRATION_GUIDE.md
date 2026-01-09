# Macro Integration Guide

**Created**: January 8, 2026
**Status**: Planning Phase
**Priority**: P0 (Blocks Phase 2.1 progress)

---

## Overview

The ZULON macro system exists but is not integrated into the compiler pipeline. This guide documents the integration work needed to enable `panic!`, `assert!`, and other macros to work in ZULON code.

---

## Current State

### What Exists

1. **Macro System** (`crates/zulon-macros`)
   - `MacroExpanderEngine` - Expands macros with pattern matching
   - Test suite with 8/8 tests passing
   - Macros defined: `panic!`, `assert!`, `assert_eq!`, `assert_ne!`, `stringify!`

2. **Macro Definition**
```rust
// crates/zulon-macros/src/lib.rs
self.macros.insert("panic".to_string(), Macro {
    name: Identifier::new("panic"),
    rules: vec![
        MacroRule {
            matcher: MacroMatcher { /* ... */ },
            expander: MacroExpander {
                template: vec![
                    TemplateFragment::Literal("::__zulon_builtin_panic($message)".to_string()),
                ],
            },
        },
    ],
});
```

### What's Missing

1. **Compiler Integration**
   - Parser doesn't expand macros
   - HIR lowering doesn't expand macros
   - Type checker doesn't recognize macro invocations

2. **Test Evidence**
```zulon
// This fails with "Undefined variable: panic"
fn main() -> i32 {
    panic!("test message");
    0
}
```

---

## Integration Architecture

### Option 1: Preprocessing Phase (RECOMMENDED)

```
Source → Macro Expansion → Token Stream → Parser → HIR → ...
```

**Implementation**:

1. Add macro phase in `crates/zulon-compiler/src/compiler.rs`:
```rust
pub struct Compiler {
    // ... existing fields ...
    macro_engine: MacroExpanderEngine,
}

impl Compiler {
    fn expand_macros(&self, source: &str) -> Result<String, CompilerError> {
        let mut expanded = source.to_string();
        
        // Find macro invocations (pattern: name!(...))
        for (macro_name, macro_def) in &self.macro_engine.macros {
            expanded = self.expand_single_macro(&expanded, macro_name, macro_def)?;
        }
        
        Ok(expanded)
    }
    
    fn compile_source(&self, source: &str, path: &Path) -> CompilerResult<()> {
        // NEW: Expand macros first
        let expanded_source = self.expand_macros(source)?;
        
        // Continue with existing pipeline
        let tokens = self.lexer.tokenize(&expanded_source)?;
        let ast = self.parser.parse(&tokens)?;
        // ... rest of pipeline
    }
}
```

**Pros**:
- Clean separation
- Macro expansion happens before parsing
- Simpler error messages
- Can show both original and expanded code

**Cons**:
- Need to track source locations for errors
- Requires careful handling of nested macros

### Option 2: Parser Integration

```
Source → Tokens → Parser (with macro expansion) → AST → ...
```

**Implementation**:

1. Modify `crates/zulon-parser/src/parser/mod.rs`:
```rust
impl Parser {
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        // Check for macro invocation first
        if self.peek_is_macro_invocation() {
            return self.parse_macro_invocation();
        }
        
        // Existing expression parsing
        // ...
    }
    
    fn parse_macro_invocation(&mut self) -> Result<Expr, ParseError> {
        let name = self.consume_ident()?.to_string();
        self.consume(Token::Bang)?;
        self.consume(Token::LParen)?;
        
        // Get macro arguments
        let args = self.parse_macro_args()?;
        
        self.consume(Token::RParen)?;
        
        // Expand macro
        let expanded = MACRO_ENGINE.expand(&name, &args)?;
        
        // Parse the expanded result
        // ... recursively parse expanded tokens
    }
}
```

**Pros**:
- Single pass
- Uses existing parser infrastructure

**Cons**:
- More complex parser
- Harder to debug macro issues
- Recursive parsing complexity

### Option 3: HIR Lowering Integration

```
Source → Tokens → Parser → AST → HIR Lowering (with macro expansion) → MIR → ...
```

**Implementation**:

1. Modify `crates/zulon-hir/src/simple_lower.rs`:
```rust
impl HirLowering {
    fn lower_expr(&mut self, expr: &ast::Expr) -> hir::Expr {
        match expr {
            ast::Expr::MacroInvocation { name, args } => {
                // Expand macro during HIR lowering
                let expanded = self.macro_engine.expand(name, args)?;
                self.lower_expanded_expr(&expanded)
            }
            // ... existing cases
        }
    }
}
```

**Pros**:
- Works with existing AST
- Can use type information
- Less invasive to parser

**Cons**:
- Later in pipeline than ideal
- May need to re-type-check expanded code

---

## Recommended Implementation Plan

### Phase 1: Basic Macro Expansion (4-6 hours)

**Goal**: Make `panic!("test")` work

**Steps**:

1. **Create Macro Expansion Module**
   - File: `crates/zulon-compiler/src/macro_expander.rs`
   - Implement basic pattern matching
   - Handle simple macros (no nesting)

2. **Integrate into Compiler**
   - Add preprocessing phase
   - Call macro expansion before parsing
   - Track source locations

3. **Test**
   - Compile `panic!` example
   - Verify LLVM IR contains `call void @__zulon_builtin_panic`
   - Run executable and check output

### Phase 2: Advanced Features (2-3 hours)

**Goal**: Support nested macros and complex patterns

**Steps**:

1. **Nested Macro Support**
   - Detect macro invocations in expanded code
   - Recursively expand until no macros remain
   - Prevent infinite recursion

2. **Variable Substitution**
   - Handle `$var` patterns correctly
   - Support `$($var:pat)*` repetition
   - Validate macro arity

3. **Error Reporting**
   - Map expanded locations back to source
   - Show original macro invocation in errors
   - Provide helpful hints

### Phase 3: Parser Enhancement (1-2 hours)

**Goal**: Allow `__zulon_builtin_*` identifiers

**Steps**:

1. **Update Lexer**
   - File: `crates/zulon-parser/src/lexer/token.rs`
   - Allow identifiers starting with `__`
   - Add `BuiltinIdent` token type

2. **Update Parser**
   - File: `crates/zulon-parser/src/parser/mod.rs`
   - Accept `BuiltinIdent` where identifiers are expected
   - Add validation for builtin names

---

## Testing Strategy

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_panic_macro_expansion() {
        let source = r#"
            fn main() -> i32 {
                panic!("test message");
                0
            }
        "#;
        
        let expected = r#"
            fn main() -> i32 {
                __zulon_builtin_panic("test message");
                0
            }
        "#;
        
        let expanded = expand_macros(source).unwrap();
        assert_eq!(expanded, expected);
    }
    
    #[test]
    fn test_nested_macros() {
        // Test macros within macros
    }
    
    #[test]
    fn test_variable_substitution() {
        // Test $var substitution
    }
}
```

### Integration Tests

```zulon
// examples/test_panic_simple.zl
fn main() -> i32 {
    panic!("Simple panic message");
}

// examples/test_panic_with_value.zl
fn main() -> i32 {
    let x = 42;
    panic!("Value is: {}", x);
}

// examples/test_assert_macro.zl
fn main() -> i32 {
    assert!(1 + 1 == 2, "Math is broken");
    0
}
```

---

## Success Criteria

- [ ] `panic!("message")` compiles and runs
- [ ] `assert!(condition)` compiles and runs
- [ ] Error messages show original source locations
- [ ] Nested macros work correctly
- [ ] All 8 macro tests still pass
- [ ] Integration tests pass
- [ ] Documentation updated

---

## References

- Macro System: `crates/zulon-macros/src/lib.rs`
- Compiler: `crates/zulon-compiler/src/compiler.rs`
- Parser: `crates/zulon-parser/src/parser/mod.rs`
- HIR Lowering: `crates/zulon-hir/src/simple_lower.rs`

---

*Macro Integration Guide*
*Created: January 8, 2026*
*ZULON Language Team* 🦀
