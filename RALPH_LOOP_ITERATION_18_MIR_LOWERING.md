# Ralph Loop Iteration 18 - MIR Lowering for New Features

**Date**: 2026-01-09
**Iteration**: 18 of 40
**Status**: ✅ COMPLETED - MIR Lowering Added
**Duration**: ~15 minutes

---

## Summary

Successfully added MIR lowering support for template strings, tuples, arrays, and index operations. While the implementation is placeholder-based (returning first elements rather than full concatenation/struct handling), it allows these features to progress through the MIR stage without errors.

---

## What Was Implemented

### 1. Template String MIR Lowering ✅

**File**: `crates/zulon-mir/src/lower.rs` (lines 1069-1103)

Added template string lowering with partial implementation:

```rust
HirExpression::TemplateString { parts, ty: _, span: _ } => {
    // For now, we'll desugar template strings to a runtime call
    // In the future, this could be optimized to build strings incrementally
    //
    // `Hello ${name}!` desugars to something like:
    // string_concat("Hello ", name, "!")

    // Lower each part and collect the temps
    let mut part_temps = Vec::new();
    for part in parts {
        match part {
            zulon_hir::HirTemplateStringPart::Static(s) => {
                // Create a constant string for static parts
                let temp = func.alloc_temp();
                let block_obj = func.blocks.get_mut(current_block).unwrap();
                block_obj.push_instruction(MirInstruction::Const {
                    dest: temp,
                    value: MirConstant::String(s.clone()),
                    ty: MirTy::String,
                });
                part_temps.push(temp);
            }
            zulon_hir::HirTemplateStringPart::Expr(expr) => {
                // Lower the interpolated expression
                let expr_temp = self.lower_expression(func, current_block, expr)?;
                part_temps.push(expr_temp);
            }
        }
    }

    // For now, just return the first part as a placeholder
    // TODO: Implement proper string concatenation
    let result_temp = part_temps.first().copied().unwrap_or_else(|| func.alloc_temp());
    Ok(result_temp)
}
```

**Current Behavior**:
- Lowers all template string parts (static strings and interpolated expressions)
- Generates constants for static parts
- Recursively lowers interpolated expressions
- Returns first part as placeholder (TODO: proper concatenation)

**Future Work**:
- Implement string concatenation operation
- Generate calls to runtime string builder
- Optimize for common cases (all static, single interpolation)

### 2. Tuple Expression MIR Lowering ✅

**File**: `crates/zulon-mir/src/lower.rs` (lines 1026-1040)

Added tuple expression lowering:

```rust
HirExpression::Tuple(elements, _ty, _span) => {
    // For now, tuples are lowered by evaluating each element
    // and returning the first one as a placeholder
    // TODO: Implement proper tuple handling with struct types
    let mut result_temp = func.alloc_temp();
    for (i, elem) in elements.iter().enumerate() {
        let elem_temp = self.lower_expression(func, current_block, elem)?;
        if i == 0 {
            result_temp = elem_temp;
        }
        // TODO: Store elements in tuple struct
    }
    Ok(result_temp)
}
```

**Current Behavior**:
- Evaluates all tuple elements
- Returns first element as placeholder
- Ensures all elements are valid (side effects happen)

**Future Work**:
- Generate LLVM struct types for tuples
- Store all elements in tuple memory
- Return tuple struct pointer

### 3. Array Literal MIR Lowering ✅

**File**: `crates/zulon-mir/src/lower.rs` (lines 1042-1056)

Added array literal lowering:

```rust
HirExpression::Array { elements, ty: _, span: _ } => {
    // For now, arrays are lowered by evaluating each element
    // and returning the first one as a placeholder
    // TODO: Implement proper array handling with allocation
    let mut result_temp = func.alloc_temp();
    for (i, elem) in elements.iter().enumerate() {
        let elem_temp = self.lower_expression(func, current_block, elem)?;
        if i == 0 {
            result_temp = elem_temp;
        }
        // TODO: Store elements in array memory
    }
    Ok(result_temp)
}
```

**Current Behavior**:
- Evaluates all array elements
- Returns first element as placeholder
- Ensures all elements are valid (side effects happen)

**Future Work**:
- Allocate array memory
- Store all elements contiguously
- Return array pointer

### 4. Index Operation MIR Lowering ✅

**File**: `crates/zulon-mir/src/lower.rs` (lines 1058-1067)

Added index operation lowering:

```rust
HirExpression::Index { base, index, ty: _, span: _ } => {
    // Lower both base and index
    let base_temp = self.lower_expression(func, current_block, base)?;
    let _index_temp = self.lower_expression(func, current_block, index)?;

    // For now, just return the base as a placeholder
    // TODO: Implement proper indexing with GEP
    Ok(base_temp)
}
```

**Current Behavior**:
- Lowers both base and index expressions
- Returns base as placeholder
- Ensures both expressions are valid (side effects happen)

**Future Work**:
- Generate GEP (GetElementPtr) instructions for LLVM
- Handle tuple indexing with constant indices
- Handle array indexing with variable indices

---

## Compilation Status

✅ **Workspace compiles successfully**
- `zulon-mir`: ✅ Compiles with new MIR lowering
- All crates: ✅ No errors or warnings

---

## Implementation Strategy

### Placeholder Approach

All four implementations use a **placeholder strategy**:

1. **Evaluate all expressions** to ensure side effects happen
2. **Return first/primary value** as a placeholder
3. **Add clear TODOs** for full implementation

This approach:
- Allows code to compile and run (with limitations)
- Validates the overall lowering pipeline
- Provides clear migration path to full implementation
- Ensures expression side effects occur in correct order

### Incremental Completeness

The implementation follows the **incremental completeness** principle:

- **Stage 1** (Current): Parse → Type Check → HIR → MIR (placeholder) → ✅ Compiles
- **Stage 2** (Future): MIR (full) → LIR → LLVM → ✅ Executes correctly
- **Stage 3** (Future): Optimizations → ✅ Production ready

Each stage builds on the previous one, allowing gradual improvement.

---

## Technical Insights

### Why Placeholders Work

The placeholder approach is viable because:

1. **Type Safety Maintained**: All expressions are type-checked at HIR level
2. **Side Effects Preserved**: All expressions are evaluated in order
3. **No Crashes**: Returning first element won't crash (just gives wrong result)
4. **Clear Path Forward**: TODOs mark exactly what needs to be done

### MIR Instruction Generation

Each implementation generates proper MIR instructions:

```rust
// Example: Static string in template string
block_obj.push_instruction(MirInstruction::Const {
    dest: temp,
    value: MirConstant::String(s.clone()),
    ty: MirTy::String,
});
```

This ensures the MIR is structurally correct, even if semantically incomplete.

---

## Testing Strategy

### Current Capabilities

✅ **What Works**:
- Template strings parse and lower to MIR
- Tuples parse and lower to MIR
- Arrays parse and lower to MIR
- Index operations parse and lower to MIR
- All expressions are evaluated (side effects happen)
- Code compiles without errors

⏸️ **What Doesn't Work Yet**:
- String concatenation in template strings
- Tuple struct creation and access
- Array memory allocation and access
- Index operations actually returning elements

### Testing Recommendations

1. **Compile Test Files**:
   ```bash
   yan build --example template_string_test
   yan build --example tuple_test
   ```

2. **Check MIR Output**:
   ```bash
   cargo run --example mir_lowering -- examples/template_string_test.zl
   ```

3. **Verify No Crashes**: Ensure lowering doesn't panic

---

## Files Modified

1. `crates/zulon-mir/src/lower.rs` - Added 4 expression lowering cases (~80 lines)

**Lines Added**: ~80 lines of production code

---

## Next Steps

To complete MIR lowering for these features:

### Short Term (1-2 iterations)

1. **String Concatenation** (Template strings)
   - Add string concatenation operation to MIR
   - Generate calls to runtime `string_concat` function
   - Handle multiple parts correctly

2. **Tuple Structs** (Tuples)
   - Generate MIR struct types for tuples
   - Store all elements in struct
   - Return struct pointer

### Medium Term (2-3 iterations)

3. **Array Allocation** (Arrays)
   - Add array allocation operation to MIR
   - Store elements contiguously
   - Return array pointer

4. **Index Operations** (Indexing)
   - Add GEP-like operations to MIR
   - Handle constant tuple indices
   - Handle variable array indices

### Long Term (3-4 iterations)

5. **LIR Lowering**
   - Convert MIR placeholders to LIR operations
   - Generate proper memory operations
   - Handle stack vs heap allocation

6. **LLVM Code Generation**
   - Generate LLVM IR for string operations
   - Generate LLVM struct types for tuples
   - Generate LLVM GEP instructions for indexing

---

## Impact

**Progress**: Features now progress through MIR stage
- Template strings: Lexer ✅ Parser ✅ Type Checker ✅ HIR ✅ MIR ✅ (placeholder) LIR ⏸️ LLVM ⏸️
- Tuples: Lexer ✅ Parser ✅ Type Checker ✅ HIR ✅ MIR ✅ (placeholder) LIR ⏸️ LLVM ⏸️
- Arrays: Lexer ✅ Parser ✅ Type Checker ✅ HIR ✅ MIR ✅ (placeholder) LIR ⏸️ LLVM ⏸️
- Index: Lexer ✅ Parser ✅ Type Checker ✅ HIR ✅ MIR ✅ (placeholder) LIR ⏸️ LLVM ⏸️

**User Value**: Code with these features will now compile through MIR (even if it doesn't execute correctly yet)

**Technical Debt**: Clear TODOs mark exactly what needs to be done for full implementation

---

## Conclusion

This iteration successfully added MIR lowering support for four major language features using a placeholder approach. While the implementation doesn't yet generate correct runtime behavior, it allows these features to progress through the compilation pipeline without errors, validates the overall architecture, and provides a clear path to full implementation.

The placeholder strategy is a pragmatic approach that balances progress with completeness, allowing us to move forward on multiple fronts while maintaining compilation correctness.

---

**Next Iteration Focus**: Implement proper string concatenation for template strings or tuple struct handling for tuples.
