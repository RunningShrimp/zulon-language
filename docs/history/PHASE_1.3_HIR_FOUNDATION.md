# Phase 1.3: HIR Foundation - Implementation Status

**Date**: January 7, 2026
**Status**: 🚧 In Progress (Foundation Complete - Integration In Progress)
**Completion**: ~40% (Core structures defined, lowering needs refactoring)

## Executive Summary

Phase 1.3 (HIR - High-Level Intermediate Representation) foundation has been established with:
- ✅ Complete HIR type system (`HirTy`)
- ✅ Comprehensive HIR node definitions
- ✅ Lowering framework structure
- 🚧 AST integration in progress (needs API alignment)

**Key Achievement**: HIR architecture is designed and foundational structures are in place. The remaining work is primarily about aligning with the actual AST structure from the parser.

---

## What is HIR?

HIR (High-Level Intermediate Representation) is a **typed, desugared** representation of ZULON programs that sits between the AST and MIR.

### Purpose

1. **Explicit Types**: Every HIR node carries its type inline, eliminating type lookups
2. **Desugared**: Complex syntax is simplified to canonical forms
3. **Validation**: Confirms type checking works end-to-end
4. **Optimization**: Provides a stable target for optimizations

### Relationship to Other IRs

```
Source Code
    ↓
AST (zulon-parser)     - Syntax trees, close to source
    ↓ Type Checking
HIR (zulon-hir)         - Explicit types, desugared ← WE ARE HERE
    ↓ Borrow Checking
MIR (zulon-mir)         - Control flow, simplified
    ↓ Optimization
LIR                      - Low-level, machine-ready
    ↓ Code Generation
Machine Code
```

---

## Completed Components

### 1. HIR Type System ✅

**File**: `crates/zulon-hir/src/ty.rs` (~300 lines)

**Features**:
- Complete type hierarchy (primitives, composites, generics)
- No type variables (all resolved during lowering)
- Conversion from `zulon_typeck::Ty` to `HirTy`
- Helper methods: `is_numeric()`, `is_copy()`, `display_name()`

**Key Types**:
```rust
pub enum HirTy {
    // Primitives
    Bool, I32, F64, String, Char, Unit, Never,

    // Composites
    Ref { inner: Box<HirTy>, mutable: bool },
    Array { inner: Box<HirTy>, len: Option<u64> },
    Tuple(Vec<HirTy>),

    // Functions
    Function { params: Vec<HirTy>, return_type: Box<HirTy> },

    // ADTs
    Struct { name: String, generics: Vec<HirTy> },
    Enum { name: String, generics: Vec<HirTy> },

    // Optional
    Optional(Box<HirTy>),
}
```

**Design Decision**: Types are fully resolved (no `TyVar`). This simplifies later phases at the cost of requiring complete type inference before lowering.

### 2. HIR Node Definitions ✅

**File**: `crates/zulon-hir/src/hir.rs` (~440 lines)

**Features**:
- Complete node hierarchy for all ZULON constructs
- Every expression has inline type annotation
- Span information for error reporting
- Node IDs for referencing

**Key Nodes**:
```rust
// Top-level
pub struct HirCrate { pub items: Vec<HirItem>, pub span: Span }

// Functions
pub struct HirFunction {
    pub id: NodeId,
    pub name: String,
    pub params: Vec<HirParam>,
    pub return_type: HirTy,
    pub body: HirBlock,
}

// Expressions (all typed)
pub enum HirExpression {
    Literal(HirLiteral, NodeId, HirTy, Span),
    Variable(String, NodeId, HirTy, Span),
    BinaryOp { op, left, right, ty, span },
    Call { func, args, ty, span },
    If { condition, then_block, else_block, ty, span },
    // ... more variants
}

impl HirExpression {
    pub fn ty(&self) -> &HirTy  // Accessor for type
    pub fn span(&self) -> &Span  // Accessor for span
}
```

**Design Decision**: Types are stored inline in expressions rather than in a separate map. This makes type access O(1) and simplifies later phases.

### 3. Lowering Framework Structure ✅

**File**: `crates/zulon-hir/src/lower.rs` (~500 lines - needs refactoring)

**Features**:
- `LoweringContext` for managing state
- Per-node lowering functions
- Type checker integration (public API)
- Error handling

**Current State**:
- Framework is in place
- Needs updates to match actual AST structure
- Some type checker methods are private (need public wrappers)

---

## Current Challenges

### Challenge 1: AST Structure Mismatch

**Issue**: The lowering code was written based on assumed AST structure that differs from the actual parser output.

**Examples**:
- Expected: `ast::ItemKind::Mod`  → Actual: `ast::ItemKind::Module`
- Expected: `ast::Statement` enum → Actual: `ast::Statement` struct with `kind: StatementKind`
- Expected: `ExpressionKind::Variable` → Actual: `ExpressionKind::Path`

**Solution**: Update all pattern matches to use correct AST node types (in progress).

### Challenge 2: Type Checker Private API

**Issue**: TypeChecker's checking methods (`check_function`, `check_block`, `check_expression`) are private.

**Current Workaround**:
1. Use public `check_item()` if available
2. Or add public wrapper methods to `TypeChecker`
3. Or access types through `Env` (currently private)

**Proposed Solution**: Add public getter methods to TypeChecker:
```rust
impl TypeChecker {
    pub fn get_type(&self, name: &str) -> Option<Ty> {
        self.env.get_binding(name)
    }

    pub fn check_and_get_type(&mut self, expr: &Expression) -> Result<Ty> {
        self.check_expression(expr)
    }
}
```

### Challenge 3: Statement Structure

**Issue**: AST has `Statement { kind: StatementKind }` but lowering assumed enum directly.

**Fix**: Update lowering to handle:
```rust
match stmt.kind {
    ast::StatementKind::Local(local) => { /* ... */ }
    ast::StatementKind::Expr(expr) => { /* ... */ }
    ast::StatementKind::Empty => { /* skip */ }
}
```

---

## Implementation Plan

### Immediate (To Complete Foundation)

1. **Fix AST Compatibility** (~2-3 hours)
   - Update all pattern matches to use correct AST types
   - Handle `Statement` struct with `kind` field
   - Use `Path` instead of `Variable`
   - Use `Module` instead of `Mod`

2. **Type Checker API** (~1 hour)
   - Option A: Add public getter methods to `TypeChecker`
   - Option B: Make lowering internal to `zulon-typeck`
   - Recommendation: Option A for cleaner separation

3. **Integration Tests** (~1-2 hours)
   - Create simple end-to-end test
   - Verify AST → HIR → MIR pipeline works
   - Test with a minimal example program

### Next Steps (After Foundation)

4. **Complete HIR Features**
   - Generic parameter lowering
   - Trait system integration
   - Pattern type checking
   - Method resolution

5. **MIR Preparation**
   - Define MIR node types
   - HIR → MIR transformation
   - Control flow graph construction

---

## Architecture Highlights

### 1. Type Storage Strategy

HIR stores types **inline** in each node:

```rust
// HIR - types inline
BinaryOp {
    op: Add,
    left: Box<Expr>,  // Expr has ty: HirTy field
    right: Box<Expr>,
    ty: HirTy,        // ← Result type stored here
}
```

**Advantages**:
- No type map lookups needed
- Obvious what type each node has
- Simplifies later phases

**Disadvantages**:
- Larger memory footprint
- Types duplicated in similar subtrees

### 2. Node ID Allocation

HIR nodes get unique IDs during lowering:

```rust
pub struct HirFunction {
    pub id: NodeId,  // Assigned during lowering
    // ...
}
```

**Purpose**:
- Uniquely identify nodes
- Enable referencing (e.g., in debug info)
- Support future passes that build on HIR

### 3. Desugaring Examples

HIR performs some desugaring:

**Method Calls**:
```rust
// AST
obj.method(arg1, arg2)

// HIR (desugared)
MethodCall {
    receiver: obj,
    method_name: "method",
    args: [arg1, arg2],
}
```

**If Expressions**:
```rust
// AST - if without else
if cond { body }

// HIR - explicit else block
If {
    condition: cond,
    then_block: body,
    else_block: Some(Block { statements: [], trailing: None, ty: Unit }),
}
```

---

## Code Statistics

### Completed Files

| File | Lines | Status | Purpose |
|------|-------|--------|---------|
| `ty.rs` | ~300 | ✅ Complete | HIR type system |
| `hir.rs` | ~440 | ✅ Complete | HIR node definitions |
| `error.rs` | ~25 | ✅ Complete | Error types |
| `lib.rs` | ~15 | ✅ Complete | Public API |
| `lower.rs` | ~500 | 🚧 In Progress | AST → HIR transformation |
| **Total** | **~1,280** | **40%** | Core foundation |

### Test Coverage

Currently: 0 tests (needs to be added)

Target: 10-15 integration tests demonstrating:
- Simple function lowering
- Expression type preservation
- Block and statement lowering
- Basic control flow

---

## Design Decisions

### Decision 1: No Type Variables in HIR

**Choice**: HIR has fully resolved types (no `HirTy::TyVar`)

**Rationale**:
- Type inference is complete before lowering
- Simplifies HIR significantly
- Makes MIR lowering easier

**Trade-off**: Cannot represent partially typed code, which is acceptable since HIR comes after type checking.

### Decision 2: Inline Type Storage

**Choice**: Store types in each node rather than side-table

**Rationale**:
- O(1) type access
- Clearer API
- Simpler than managing ID-to-type maps

**Trade-off**: Higher memory usage, but acceptable for HIR's relatively small size.

### Decision 3: Separate Crate

**Choice**: HIR is a separate crate (`zulon-hir`)

**Rationale**:
- Clean separation of concerns
- Can be independently tested
- Follows Rust convention (like `rustc_hir`)

**Trade-off**: More complex than a module, but worth it for organization.

---

## Lessons Learned

### 1. Understand Before Building

I initially wrote lowering code based on assumed AST structure. Lesson learned: **always check the actual AST definitions first**.

### 2. Public API Matters

TypeChecker's methods being private caused integration issues. Lesson learned: **design public APIs from the start**.

### 3. Incremental Validation

Trying to implement everything at once led to many compounding errors. Lesson learned: **build incrementally and validate each step**.

---

## Next Session Plan

### If Continuing Implementation

**Priority 1**: Fix AST compatibility
- Update `lower.rs` pattern matches
- Test with a simple example
- Verify build succeeds

**Priority 2**: Type Checker API
- Add public getter to TypeChecker
- Or make lowering internal to typeck
- Update lowering to use public API

**Priority 3**: Documentation
- Add examples to this doc
- Create architecture diagram
- Write lowering guide

### If Pausing Implementation

**Current State**: Foundation is solid (~40% complete)

**What Works**:
- HIR type system is complete
- HIR node definitions are comprehensive
- Crate structure is correct

**What Needs Work**:
- AST integration compatibility
- Type checker public API
- Initial test cases

**Recommendation**: The foundation is solid enough to serve as a reference for continuing later. The remaining work is straightforward integration debugging.

---

## Comparison with Similar Systems

### Rust Compiler (rustc)

**rustc_hir**:
- Very similar design goals
- Inline type storage
- Node IDs for referencing
- Desugared representation

**Key Difference**: Rust's HIR is much more complex due to Rust's complexity (lifetimes, traits, macros). ZULON's HIR can be simpler.

### LLVM IR

**LLVM**:
- Lower-level than HIR (more like MIR)
- SSA form
- Explicit types

**Key Difference**: HIR preserves high-level structure (functions, classes) that LLVM IR doesn't have.

### Cranelift IR (Clif)

**Cranelift**:
- Low-level, register-based
- No high-level concepts
- Very explicit

**Key Difference**: HIR is much higher-level, suitable for optimizations before lowering to machine code.

---

## Conclusion

Phase 1.3 (HIR Foundation) is **~40% complete** with solid architectural foundations:

✅ **Complete**: Type system, node definitions, crate structure
🚧 **In Progress**: AST integration, type checker API
⏸️ **Deferred**: MIR lowering, optimizations, full feature support

**Status**: The architecture is sound and the foundation is stable. The remaining work is straightforward integration and testing.

**Recommendation**: Continue with fixing AST compatibility (2-3 hours work) to reach a working initial version, or pause and return later with fresh perspective. The current code serves as excellent documentation of the intended design.

---

**Generated**: 2026-01-07
**Author**: Claude Code
**Status**: Phase 1.3 Foundation - 40% Complete
**Next Phase**: MIR (Mid-Level Intermediate Representation)
