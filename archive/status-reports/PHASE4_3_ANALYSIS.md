# Phase 4.3 Analysis: Throw Statement Codegen Strategy

**Date**: 2026-01-08
**Phase**: 4.3 - Implement Throw Statement Codegen
**Status**: ⏸️ PAUSED - Requires architecture decision
**Analysis**: Complete

---

## Critical Discovery

**Problem**: The `error_type` information is **NOT passed from HIR to MIR/LIR**!

**Evidence**:
```rust
// HIR (has error_type)
pub struct HirFunction {
    pub return_type: HirTy,
    pub error_type: Option<HirTy>,  // ✅ Exists
    pub effects: Vec<HirTy>,
}

// MIR (no error_type)
pub struct MirFunction {
    pub return_type: MirTy,  // ❌ No error_type field
}

// LIR (no error_type)
pub struct LirFunction {
    pub return_type: LirTy,  // ❌ No error_type field
}
```

**Impact**: LLVM codegen **cannot detect** if a function has error type!

---

## Root Cause Analysis

### HIR→MIR Lowering

**Location**: `crates/zulon-mir/src/lower.rs`

```rust
pub fn lower_function(&mut self, func: &HirFunction) -> Result<MirFunction> {
    let mir_func = MirFunction::new(
        func.name.clone(),
        func.params.iter().map(|p| MirParam {
            name: p.name.clone(),
            ty: p.ty.clone().into(),
        }).collect(),
        func.return_type.clone().into(),  // Only return_type!
        // ❌ error_type is LOST!
    );
}
```

**Missing**:
- No conversion of `fn() -> T | E` to `fn() -> Outcome<T, E>`
- No passing of error_type information
- MIR function only knows about return_type

---

## Strategic Options

### Option A: Fix Type Lowering (RECOMMENDED ⭐)

**Approach**: Convert `T | E` to `Outcome<T, E>` during HIR→MIR lowering

**Implementation**:
1. In HIR type lowering, detect `fn() -> T | E`
2. Convert to `fn() -> Outcome<T, E>`
3. MIR/LIR only see Outcome type

**Pros**:
- Clean architecture
- MIR/LIR don't need error_type field
- Consistent with "explicit Outcome" approach
- Matches user-facing syntax

**Cons**:
- Requires HIR→MIR lowering changes
- May need type system updates

**Effort**: 2-3 hours

**Code Changes**:
```rust
// In HIR→MIR lowering
let return_type = if func.error_type.is_some() {
    // Convert T | E to Outcome<T, E>
    HirTy::Enum("Outcome".to_string(), vec![
        func.return_type.clone(),
        func.error_type.clone().unwrap(),
    ])
} else {
    func.return_type.clone()
};

let mir_func = MirFunction::new(
    func.name.clone(),
    params,
    return_type.into(),  // Now always concrete type
);
```

### Option B: Add error_type Field

**Approach**: Pass error_type through MIR/LIR layers

**Implementation**:
1. Add `error_type: Option<LirTy>` to MirFunction
2. Add `error_type: Option<LirTy>` to LirFunction
3. Update HIR→MIR→LIR lowering
4. Update LLVM codegen to check error_type

**Pros**:
- Preserves type information
- Explicit error tracking

**Cons**:
- More complex (changes 4+ layers)
- Redundant information (return_type + error_type)
- Inconsistent with "explicit Outcome" design

**Effort**: 4-6 hours

### Option C: Detect Outcome in Codegen (FALLBACK)

**Approach**: LLVM codegen detects if return_type is Outcome

**Implementation**:
1. Check if return_type name == "Outcome"
2. If so, check if return value is error type (not wrapped)
3. Construct Outcome::Err if needed

**Pros**:
- Minimal changes (only LLVM codegen)
- Works with current architecture

**Cons**:
- Fragile (depends on type name)
- Doesn't fix root cause
- May miss edge cases

**Effort**: 1-2 hours

---

## Recommendation

### Implement Option A: Fix Type Lowering ⭐

**Why**:
1. **Clean Architecture**: Each layer has clear responsibility
2. **Consistent Design**: Matches explicit Outcome<T, E> syntax
3. **Simpler Codegen**: LLVM doesn't need special error handling
4. **Future-Proof**: Works for all error types

**How**:
1. **Phase 4.3.1**: Update HIR→MIR lowering (1h)
   - Detect `fn() -> T | E`
   - Convert to `fn() -> Outcome<T, E>`

2. **Phase 4.3.2**: Update MIR→LIR lowering (0.5h)
   - Pass Outcome return_type through

3. **Phase 4.3.3**: Update LLVM codegen (1h)
   - Detect Outcome return type
   - For throw: wrap error in Outcome::Err

**Total**: 2.5 hours

**Success Criteria**:
- `fn() -> T | E` becomes `fn() -> Outcome<T, E>` in MIR
- throw statements generate Outcome::Err construction
- ? operators work (already should with Phase 4.2)

---

## Alternative: Quick Win Path

**If time is limited**, implement Option C first:

**Phase 4.3 Quick**: Simple throw codegen (1h)
```rust
// In generate_terminator
LirTerminator::Return(value) => {
    if let Some(vreg) = value {
        // Check if function returns Outcome
        if is_outcome_type(&func.return_type) {
            // Check if value is already wrapped
            if !is_outcome_variant(vreg) {
                // Wrap in Outcome::Err
                return self.generate_error_return(vreg, func);
            }
        }
        // Normal return
        writeln!(self.writer, "ret {} %v{}", ...)?;
    }
}
```

**Then**: Fix type lowering properly (Option A) later

**Total**: 1h for quick fix, 2.5h for proper fix

---

## Decision Matrix

| Option | Effort | Quality | Risk | Recommendation |
|--------|--------|--------|------|----------------|
| A: Fix type lowering | 2.5h | ⭐⭐⭐⭐⭐ | Low | **BEST** ✅ |
| B: Add error_type field | 4-6h | ⭐⭐⭐ | Medium | Overkill |
| C: Detect in codegen | 1-2h | ⭐⭐ | High | Quick fallback |

---

## Next Actions

### Immediate (Recommended)

**Start Option A Implementation**:

1. **Step 1**: Update HIR→MIR lowering
   - File: `crates/zulon-mir/src/lower.rs`
   - Function: `lower_function`
   - Add Outcome type conversion

2. **Step 2**: Test type conversion
   - Verify `fn() -> T | E` becomes `fn() -> Outcome<T, E>`
   - Check MIR output

3. **Step 3**: Update LLVM codegen
   - File: `crates/zulon-codegen-llvm/src/codegen.rs`
   - Function: `generate_terminator`
   - Add Outcome::Err construction

### Alternative (If Rushed)

**Implement Option C**:
- Quick throw codegen in LLVM
- Defer proper fix to later session

---

## Risk Assessment

### Current Risk: MEDIUM ⚠️

**Issue**: Cannot implement throw without error type info

**Mitigation**: Option A solves this cleanly

**Confidence**: HIGH - Clear path forward

### Implementation Risks

**Low Risk**: Type conversion logic
- Well-understood pattern
- Similar to existing type conversions

**Medium Risk**: Outcome type definition
- Need to ensure Outcome exists in type system
- May need to add to builtin types

---

## Conclusion

**Status**: Phase 4.3 blocked by architecture issue

**Root Cause**: error_type not passed from HIR to MIR

**Solution**: Option A - Fix type lowering (convert T|E to Outcome<T,E>)

**Effort**: 2.5 hours

**Quality**: ⭐⭐⭐⭐⭐ - Clean, maintainable

**Recommendation**: ✅ Implement Option A

**Alternative**: Option C for quick win (1h), then Option A later

---

**Document Version**: 1.0
**Date**: 2026-01-08
**Status**: Analysis complete, awaiting decision
**Next**: Choose Option A or C, then implement
