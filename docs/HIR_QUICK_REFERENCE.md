# HIR Quick Reference

**Status**: Foundation Complete (~40%)
**Build**: ✅ Passing
**Tests**: Typeck (21/21 passing), HIR (pending)

---

## What is HIR?

HIR (High-Level Intermediate Representation) = Typed AST

- **Before**: AST (syntax trees, inferred types)
- **After**: HIR (explicit types, desugared)
- **Purpose**: Validate type checking, prepare for MIR

---

## Key Files

```
crates/zulon-hir/
├── src/
│   ├── lib.rs       (Public API)
│   ├── ty.rs        (HIR types - 300 lines) ✅
│   ├── hir.rs       (HIR nodes - 440 lines) ✅
│   ├── error.rs     (Errors) ✅
│   └── lower.rs     (AST→HIR, disabled) ⚠️
└── Cargo.toml
```

---

## HIR Type System

```rust
pub enum HirTy {
    // Primitives
    Bool, I32, I64, F64, Char, String, Unit, Never,

    // Composites
    Ref { inner: Box<HirTy>, mutable: bool },
    Array { inner: Box<HirTy>, len: Option<u64> },
    Tuple(Vec<HirTy>),

    // Functions
    Function { params: Vec<HirTy>, return_type: Box<HirTy> },

    // ADTs
    Struct { name: String, generics: Vec<HirTy> },
    Enum { name: String, generics: Vec<HirTy> },
}
```

**Key Point**: No type variables - all resolved!

---

## HIR Expression Examples

```rust
// Literal
HirExpression::Literal(HirLiteral::Integer(42), id, HirTy::I32, span)

// Variable
HirExpression::Variable("x".to_string(), id, HirTy::I32, span)

// Binary Operation
HirExpression::BinaryOp {
    op: HirBinOp::Add,
    left: Box::new(expr1),
    right: Box::new(expr2),
    ty: HirTy::I32,  // ← Result type
    span,
}

// Function Call
HirExpression::Call {
    func: Box::new(func_expr),
    args: vec![arg1, arg2],
    ty: HirTy::I32,  // ← Return type
    span,
}
```

**Key Point**: Every expression has inline `ty` field!

---

## Accessor Methods

```rust
impl HirExpression {
    pub fn ty(&self) -> &HirTy { /* ... */ }
    pub fn span(&self) -> &Span { /* ... */ }
}
```

Use them to get expression info without pattern matching.

---

## Type Conversion

```rust
// From typeck to HIR
let ty: zulon_typeck::Ty = /* ... */;
let hir_ty: HirTy = ty.into();  // Automatic conversion

// Manual conversion
let hir_ty = HirTy::from(ty);
```

---

## Using HIR (When Lowering Works)

```rust
use zulon_hir::{lower_ast, HirCrate};
use zulon_parser::Parser;

// Parse
let source = "fn main() { }";
let parser = Parser::new(source);
let ast = parser.parse().unwrap();

// Lower to HIR
let hir = lower_ast(&ast).unwrap();

// Access HIR
for item in &hir.items {
    match item {
        HirItem::Function(func) => {
            println!("Function: {}", func.name);
            println!("Return type: {}", func.return_type.display_name());
        }
        _ => {}
    }
}
```

---

## Current Status

### ✅ Complete
- Type system (HirTy)
- Node definitions (HirExpression, HirStatement, etc.)
- Public API
- Documentation

### ⚠️ In Progress
- AST → HIR lowering (needs AST compatibility fixes)
- Tests (framework ready, need to write tests)

### ❌ Not Started
- Generic parameter lowering
- Trait system integration
- Optimization passes
- MIR lowering

---

## Common Patterns

### Getting Expression Types

```rust
// Don't do this:
match expr {
    HirExpression::Literal(_, _, ty, _) => ty,
    HirExpression::BinaryOp { ty, .. } => ty,
    // ... many cases
}

// Do this:
let ty = expr.ty();
```

### Creating Typed Expressions

```rust
// Binary operation with type
HirExpression::BinaryOp {
    op: HirBinOp::Add,
    left: Box::new(left_expr),
    right: Box::new(right_expr),
    ty: HirTy::I32,  // ← Must specify!
    span: span.clone(),
}
```

### Type Display

```rust
println!("Type: {}", hir_ty.display_name());
// Output: "i32" or "fn(i32, i32) -> i32" etc.
```

---

## Architecture

```
Parser → AST → TypeChecker → HIR → MIR → CodeGen
                         ↑
                    (we are here)
```

**HIR Responsibilities**:
- ✅ Store types inline
- ✅ Desugar syntax
- ✅ Preserve structure
- ❌ No optimization (that's MIR's job)
- ❌ No borrow checking (happens after HIR)

---

## Next Steps

1. Fix AST compatibility in lowering
2. Add public TypeChecker API
3. Write integration tests
4. Complete generic lowering
5. Design MIR structure

---

## Documentation

- **Full Report**: `docs/PHASE_1.3_HIR_FOUNDATION.md`
- **Session Summary**: `docs/SESSION_2026_01_07_HIR_FOUNDATION_COMPLETE.md`
- **Type System**: `crates/zulon-hir/src/ty.rs`
- **Node Definitions**: `crates/zulon-hir/src/hir.rs`

---

**Last Updated**: 2026-01-07
**Status**: Foundation Complete (40%)
**Build**: ✅ Zero warnings
