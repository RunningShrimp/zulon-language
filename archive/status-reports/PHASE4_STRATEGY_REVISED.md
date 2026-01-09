# Phase 4 Strategy Revised

**Date**: 2026-01-08
**Status**: ✅ Strategy Updated
**Key Finding**: LIR lowering needs field access GEP generation

---

## Critical Discovery

After analyzing the codebase, I discovered that **LIR lowering does NOT yet implement GEP instruction generation for struct/enum field access**.

### Current State

**MIR Lowering** (✅ Complete):
```rust
MirPlace::Field {
    base: Box::new(MirPlace::Temp(outcome_temp)),
    field: "discriminant".to_string(),
}
```

**LIR Lowering** (❌ Incomplete):
```rust
// Load from Field access -> SSA rename (loses field info!)
let src_vreg = self.get_or_alloc_vreg(src, func); // Returns new vreg
self.temp_map.insert(*dest, src_vreg);
Ok(vec![])  // No GEP instruction generated!
```

**Problem**: Field access information is lost during LIR lowering!

---

## Revised Strategy

### Phase 4.2: Implement Field Access in LIR Lowering (NEW)

**Location**: `crates/zulon-lir/src/lower.rs`

**Task**: Enhance Load instruction lowering to generate GEP for field access

**Implementation**:
```rust
MirInstruction::Load { dest, src, ty } => {
    match src {
        MirPlace::Field { base, field } => {
            // Generate GEP for field access
            let base_vreg = self.get_or_alloc_vreg(base, func);
            let dest_vreg = func.alloc_vreg();
            self.temp_map.insert(*dest, dest_vreg);

            // Calculate field offset
            // For Outcome<T, E>: discriminant=0, data=1
            let field_index = match field.as_str() {
                "discriminant" => 0,
                "data" => 1,
                _ => return Err(...),
            };

            Ok(vec![
                LirInstruction::Gep {
                    dest: dest_vreg,
                    base: base_vreg,
                    indices: vec![
                        LirOperand::Imm(0),  // struct pointer
                        LirOperand::Imm(field_index),  // field index
                    ],
                    ty: ty.clone().into(),
                },
                LirInstruction::Load {
                    dest: dest_vreg,
                    src: LirOperand::Reg(dest_vreg),  // Load from GEP result
                    ty: ty.clone().into(),
                },
            ])
        }
        MirPlace::Local(name) => { /* existing code */ }
        _ => { /* existing code */ }
    }
}
```

**Estimated Time**: 2 hours

**Success Criteria**:
- GEP instructions generated for field access
- Field indices correct (discriminant=0, data=1)
- All tests still passing

### Phase 4.3: Register Outcome Type (Simplified)

**Task**: LLVM codegen needs to know Outcome layout for GEP offsets

**Approach**:
- Option A: Define Outcome as LLVM struct type
- Option B: Use byte offsets (simpler, current GEP uses i8*)

**Recommendation**: Start with Option B (byte offsets)
- Discriminant: offset 0, size 1 byte
- Data: offset 1 (or aligned), size sizeof(max(T,E))

**Implementation**: None needed if using byte offsets!

**Estimated Time**: 0.5 hour

### Phase 4.4: Throw Statement Codegen

**Task**: Enhance Return terminator to construct Outcome::Err

**Current MIR**:
```
temp1 = DivideError::Zero
Return(temp1)
```

**Need**:
- Detect when function has error type
- Construct Outcome::Err on stack
- Set discriminant = 1
- Store error value in data field
- Return the Outcome

**Implementation**: Add to `generate_terminator` in codegen.rs

**Estimated Time**: 2 hours

### Phase 4.5: ? Operator Codegen

**Good News**: Should work automatically after Phase 4.2!

**Why**:
- MIR already generates discriminant checking
- LIR will generate GEP + Load for field access
- LLVM codegen already handles GEP, Load, Cmp, Branch
- Return terminator already works

**Verification**: Test with error handling examples

**Estimated Time**: 0.5 hour (testing only)

### Phase 4.6: Integration Testing

**Task**: Test end-to-end error handling

**Tests**:
1. Throw statement generates correct LLVM IR
2. ? operator generates correct LLVM IR
3. Simple error handling program compiles and runs
4. All existing tests still pass

**Estimated Time**: 1 hour

---

## Updated Timeline

| Phase | Task | Original | Revised |
|-------|------|----------|---------|
| 4.1 | Understand codegen | 1h | ✅ 0.5h |
| 4.2 | Register Outcome type | 0.5h | **2h** (LIR field access) |
| 4.3 | Throw codegen | 2h | 2h |
| 4.4 | ? operator codegen | 3h | 0.5h (auto) |
| 4.5 | Value extraction | 3h | - (merged with 4.2) |
| 4.6 | Tests | 2h | 1h |
| **Total** | | **11.5h** | **6h** |

**Good News**: Less time than estimated! But critical path changed.

---

## Risk Assessment

### High Risk: LIR Field Access Not Implemented

**Impact**: BLOCKER - Cannot proceed without this

**Mitigation**: Implement Phase 4.2 first (field access GEP)

**Confidence**: HIGH - Clear implementation path

### Medium Risk: LLVM Struct Type Definition

**Impact**: May need proper struct definition for type safety

**Mitigation**: Start with byte offsets, enhance later if needed

**Confidence**: MEDIUM - Byte offsets should work for now

---

## Next Action

**Start Phase 4.2 Revised**: Implement field access GEP generation in LIR lowering

**File**: `crates/zulon-lir/src/lower.rs`

**Function**: `lower_instruction` -> `MirInstruction::Load`

**Test**: Compile error handling examples, check for GEP instructions

---

**Document Version**: 2.0 (Revised)
**Date**: 2026-01-08
**Status**: Strategy updated based on codebase analysis
**Confidence**: HIGH - Clear path forward
