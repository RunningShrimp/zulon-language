# LLVM Code Generation Design - Error Handling (Phase 4)

**Date**: 2026-01-08
**Focus**: Phase 4 - LLVM Code Generation for Error Handling
**Status**: ✅ Design Complete - Ready for Implementation

---

## Executive Summary

This document provides a comprehensive design for implementing LLVM IR code generation for error handling (throw and ? operators). This is the final 20% needed to complete the error handling runtime.

**Estimated Implementation Time**: 10-14 hours
**Complexity**: High (requires understanding LLVM IR, enum layout, and control flow)
**Dependencies**: MIR lowering must be complete (✅ done)

---

## Architecture Overview

### Compilation Pipeline

```
Source Code (ZULON)
    ↓ Parser
AST
    ↓ HIR Lowering
HIR (with Throw/QuestionMark)
    ↓ Type Checker
Typed HIR
    ↓ MIR Lowering
MIR (with discriminant checking)
    ↓ LIR Lowering
LIR (SSA form)
    ↓ LLVM Codegen ←─ WE ARE HERE
LLVM IR
    ↓ llc
Machine Code
```

### Current Status

- ✅ MIR has correct control flow for error handling
- ✅ Discriminant checking implemented in MIR
- ✅ Enum layout infrastructure exists
- ⏳ LLVM codegen needs to handle Outcome<T, E> specifically

---

## Design Decisions

### 1. Outcome<T, E> Enum Layout in LLVM

**Strategy**: Use Rust-like enum layout (discriminant + data)

**LLVM Representation**:
```llvm
; Outcome<i32, DivideError> as opaque struct
%enum.Outcome = type { i8, [0 x i8], i32, [0 x i8] }

; Packed layout (more efficient):
%enum.Outcome = type <{ i8, i32 }>
```

**Field Access**:
- Field 0: discriminant (i8): 0 = Ok, 1 = Err
- Field 1: data (union of T and E)

**GEP Indices**:
- Discriminant: `[0, 0]` (struct_ptr, field_0)
- Data: `[0, 1]` (struct_ptr, field_1)

### 2. Throw Statement Codegen

**MIR Input**:
```
[block_current]
    temp1 = DivideError::Zero
    Return(temp1)
```

**LLVM IR Output**:
```llvm
; Allocate Outcome on stack
%outcome = alloca %enum.Outcome

; Set discriminant to 1 (Err variant)
%disc_ptr = getelementptr %enum.Outcome, %outcome, i32 0, i32 0
store i8 1, %disc_ptr

; Construct error value
%error_val = call @DivideError_Zero()

; Store error in data field
%data_ptr = getelementptr %enum.Outcome, %outcome, i32 0, i32 1
store i32 %error_val, %data_ptr

; Load Outcome for return
%result = load %enum.Outcome, %outcome

; Return the Outcome
ret %enum.Outcome %result
```

**Key Points**:
1. Allocate stack space for Outcome
2. Set discriminant to 1 (Err)
3. Construct error value
4. Store in data field
5. Return the Outcome

### 3. Question Mark Operator Codegen

**MIR Input**:
```
[block_current]
    temp1 = call divide(10, 2)  ; Returns Outcome<i32, DivideError>
    temp2 = load temp1.discriminant
    temp3 = (temp2 == 0)
    if temp3 goto block_success else goto block_error

[block_success]
    temp4 = load temp1.data  ; Extract T
    goto block_continue

[block_error]
    temp5 = load temp1.data  ; Extract E
    Return(temp5)

[block_continue]
    ; Continue execution
```

**LLVM IR Output**:
```llvm
; Call divide function
%outcome = call @divide(i32 10, i32 2)
; %outcome: %enum.Outcome

; Load discriminant
%disc_ptr = getelementptr %enum.Outcome, %outcome, i32 0, i32 0
%disc = load i8, %disc_ptr

; Compare to 0 (Ok variant)
%is_ok = icmp eq i8 %disc, 0

; Branch on discriminant
br i1 %is_ok, label %success_block, label %error_block

success_block:
  ; Extract T from data field
  %data_ptr = getelementptr %enum.Outcome, %outcome, i32 0, i32 1
  %value = load i32, %data_ptr

  ; Continue to next code
  br label %continue_block

error_block:
  ; Extract E from data field
  %data_ptr = getelementptr %enum.Outcome, %outcome, i32 0, i32 1
  %error = load i32, %data_ptr

  ; Return error early
  ret %enum.Outcome %outcome

continue_block:
  ; Use %value in subsequent code
  ...
```

**Key Points**:
1. Load discriminant field
2. Compare to 0 (Ok variant)
3. Conditional branch
4. Extract data field (T or E)
5. Early return on error path

---

## Implementation Plan

### Phase 4.1: Understand Codegen Infrastructure (1 hour)

**Tasks**:
1. Read `zulon-codegen-llvm/src/codegen.rs`
2. Understand how LIR instructions map to LLVM IR
3. Check how Return terminator is generated
4. Check how If terminator is generated
5. Understand GEP instruction generation

**Success Criteria**:
- ✅ Understand codegen structure
- ✅ Know where to add enum handling
- ✅ Know how to generate LLVM IR instructions

### Phase 4.2: Register Outcome Type (30 min)

**Tasks**:
1. Create `EnumLayout` for Outcome<T, E>
2. Register with `CodeGenerator::register_enum`
3. Add to enum layout cache
4. Generate LLVM type definition

**Code**:
```rust
// In codegen setup
let mut outcome_layout = EnumLayout::new("Outcome".to_string(), LirTy::I8);
outcome_layout.add_variant("Ok".to_string(), 0, vec![
    ("value".to_string(), success_ty.clone())
])?;
outcome_layout.add_variant("Err".to_string(), 1, vec![
    ("error".to_string(), error_ty.clone())
])?;
outcome_layout.finalize();

codegen.register_enum(outcome_layout);
```

**Success Criteria**:
- ✅ Outcome type registered
- ✅ LLVM type definition generated
- ✅ Enum layout cached

### Phase 4.3: Implement Return Terminator Enhancement (2 hours)

**Tasks**:
1. Modify `generate_terminator` for Return
2. Check if returning an Outcome (error context)
3. If so, construct Outcome::Err variant
4. Set discriminant and store error value
5. Generate proper LLVM IR

**File**: `crates/zulon-codegen-llvm/src/codegen.rs`

**Function**: `generate_terminator`

**Changes**:
```rust
LirTerminator::Return(value) => {
    // Check if function has error type
    if let Some(error_ty) = self.current_function_error_type() {
        if let Some(vreg) = value {
            // Construct Outcome::Err(error_value)
            self.generate_error_return(vreg, error_ty)?;
        }
    } else {
        // Normal return (existing code)
        // ...
    }
}
```

**Success Criteria**:
- ✅ Generates Outcome::Err construction
- ✅ Sets discriminant correctly
- ✅ Stores error value
- ✅ All existing tests pass

### Phase 4.4: Implement Discriminant Checking (3 hours)

**Tasks**:
1. Recognize discriminant load pattern in LIR
2. Generate GEP for discriminant field
3. Generate load of discriminant value
4. Generate comparison to 0
5. Generate conditional branch

**File**: `crates/zulon-codegen-llvm/src/codegen.rs`

**Function**: `generate_instruction`

**Pattern to Recognize**:
```rust
LirInstruction::Load {
    dest: disc_temp,
    src: LirOperand::Temp(outcome_temp),
    ty: LirTy::I8,  // ← Indicates discriminant
} => {
    // Check if this is loading from a field access
    if self.is_discriminant_load(outcome_temp) {
        self.generate_discriminant_load(disc_temp, outcome_temp)?;
    }
}
```

**Success Criteria**:
- ✅ Generates discriminant GEP
- ✅ Loads discriminant value
- ✅ Generates comparison
- ✅ Works for all Outcome types

### Phase 4.5: Implement Value Extraction (3 hours)

**Tasks**:
1. Generate GEP for data field
2. Load value from data field
3. Handle both T and E types
4. Generate proper bitcasts if needed

**File**: `crates/zulon-codegen-llvm/src/codegen.rs`

**Pattern to Recognize**:
```rust
LirInstruction::Load {
    dest: result_temp,
    src: LirOperand::FieldAccess { base, field: "data" },
    ty: result_ty,
} => {
    self.generate_data_field_load(result_temp, base, result_ty)?;
}
```

**Success Criteria**:
- ✅ Generates data field GEP
- ✅ Loads correct value (T or E)
- ✅ Handles type conversions
- ✅ Works for all Outcome types

### Phase 4.6: Add Tests (2 hours)

**Tasks**:
1. Create test for throw codegen
2. Create test for ? codegen
3. Verify LLVM IR is correct
4. Add to test suite

**File**: `crates/zulon-codegen-llvm/tests/error_handling_codegen.rs`

**Tests**:
```rust
#[test]
fn test_throw_codegen() {
    let mir = generate_mir_with_throw();
    let llvm_ir = codegen.generate(&mir);

    // Verify Outcome::Err construction
    assert!(llvm_ir.contains("store i8 1"));  // discriminant = Err
    assert!(llvm_ir.contains("getelementptr"));  // field access
}

#[test]
fn test_question_mark_codegen() {
    let mir = generate_mir_with_question_mark();
    let llvm_ir = codegen.generate(&mir);

    // Verify discriminant checking
    assert!(llvm_ir.contains("icmp eq i8"));  // compare to 0
    assert!(llvm_ir.contains("br i1"));  // conditional branch
}
```

**Success Criteria**:
- ✅ Tests pass
- ✅ LLVM IR is valid
- ✅ No regressions

---

## Technical Details

### LLVM IR Instruction Reference

**getelementptr (GEP)**:
```llvm
; Pointer to discriminant field
%disc_ptr = getelementptr %enum.Outcome, %outcome_ptr, i32 0, i32 0

; Pointer to data field
%data_ptr = getelementptr %enum.Outcome, %outcome_ptr, i32 0, i32 1
```

**load/store**:
```llvm
; Load discriminant
%disc = load i8, %disc_ptr

; Store discriminant
store i8 1, %disc_ptr
```

**icmp (comparison)**:
```llvm
; Compare to 0 (Ok variant)
%is_ok = icmp eq i8 %disc, 0
```

**br (conditional branch)**:
```llvm
; Branch on discriminant
br i1 %is_ok, label %success, label %error
```

### Outcome Type Definition

**Option 1: Opaque Struct** (Current)
```llvm
%enum.Outcome = type [16 x i8]  ; Adjust size based on T+E
```

**Option 2: Packed Struct** (Better)
```llvm
%enum.Outcome = type <{ i8, T }>  ; Where T is max(sizeof(T), sizeof(E))
```

**Recommendation**: Start with opaque struct, optimize later.

---

## Risk Assessment

### Current State: MEDIUM RISK ⚠️

**Complexity Factors**:
- LLVM IR has steep learning curve
- Enum layout requires careful offset calculation
- Need to handle generic types (T and E)
- Control flow must be exactly right

**Mitigation Strategies**:
1. **Start Simple**: Implement throw first (easier than ?)
2. **Use Existing Patterns**: Follow Rust's Result codegen
3. **Incremental Testing**: Test each phase independently
4. **Fallback**: Can use opaque byte array initially

### Specific Risks

**Risk 1**: Generic Types (T, E)
- **Issue**: LLVM doesn't support generics
- **Solution**: Monomorphize at LIR level (already done)
- **Impact**: Low (LIR handles this)

**Risk 2**: Enum Layout Calculation**
- **Issue**: Must compute correct offsets
- **Solution**: Use `EnumLayout` infrastructure
- **Impact**: Low (already implemented)

**Risk 3**: Control Flow Bugs
- **Issue**: Incorrect branching
- **Solution**: Extensive testing
- **Impact**: Medium (can be debugged)

---

## Success Criteria

### Phase 4 Complete When:

1. ✅ **Throw Works**:
   - `throw DivideError::Zero` compiles
   - Generates correct Outcome::Err construction
   - Discriminant set to 1
   - Error value stored correctly

2. ✅ **Question Mark Works**:
   - `expr?` compiles
   - Generates discriminant check
   - Branches to success/error blocks
   - Extracts correct value (T or E)

3. ✅ **Integration Works**:
   - All error handling examples compile
   - LLVM IR is valid
   - No regressions in existing code
   - All tests pass

4. ✅ **End-to-End Works**:
   - Can compile and run simple error handling program
   - throw and ? work correctly at runtime
   - Error types are properly propagated

---

## Timeline Estimate

| Phase | Task | Estimate |
|-------|------|----------|
| 4.1 | Understand codegen | 1h |
| 4.2 | Register Outcome type | 0.5h |
| 4.3 | Throw codegen | 2h |
| 4.4 | Discriminant checking | 3h |
| 4.5 | Value extraction | 3h |
| 4.6 | Testing | 2h |
| **Total** | | **11.5 hours** |

**Confidence**: Medium (±3 hours)
- Could be faster if codegen infrastructure is good
- Could take longer if unexpected issues arise

---

## Next Steps

### Immediate: Start Phase 4.1

**Task**: Understand LLVM codegen infrastructure

**Actions**:
1. Read `crates/zulon-codegen-llvm/src/codegen.rs`
2. Understand LIR → LLVM IR mapping
3. Check how terminators are generated
4. Check how instructions are generated

**Time**: 1 hour

**Output**: Clear understanding of where to add error handling codegen

### Then: Phase 4.2-4.6

Follow the implementation plan sequentially, testing at each step.

---

## Conclusion

### Design Status: ✅ COMPLETE

This document provides a comprehensive design for implementing LLVM code generation for error handling. The design is:

1. **Feasible**: Builds on existing infrastructure
2. **Complete**: Covers all aspects (throw, ?, enum layout)
3. **Testable**: Clear success criteria
4. **Incremental**: Can be implemented phase by phase

### Recommendation: PROCEED WITH IMPLEMENTATION

**Why**:
- 80% of error handling is complete
- MIR lowering is production-ready
- Design is clear and achievable
- Final piece to make error handling functional

**Next Action**: Begin Phase 4.1 (Understand codegen infrastructure)

---

**Document Version**: 1.0
**Author**: ZULON Language Development Team
**Date**: 2026-01-08
**Status**: ✅ DESIGN COMPLETE - Ready for Implementation
**Estimated Time**: 10-14 hours
**Overall Progress**: Error handling 80% complete, this is the final 20%
