# Phase 4.1: LLVM Codegen Infrastructure - COMPLETE

**Date**: 2026-01-08
**Phase**: 4.1 - Understand Codegen Infrastructure
**Status**: ✅ COMPLETE
**Time**: 30 minutes

---

## Objective

Understand the LLVM IR code generation infrastructure to determine where and how to add error handling code generation.

---

## Key Findings

### 1. Codegen Architecture

**File**: `crates/zulon-codegen-llvm/src/codegen.rs`

**Main Structure**:
```rust
pub struct CodeGenerator<W: Write> {
    writer: W,
    indent: usize,
    layout_cache: Arc<LayoutCache>,          // Struct layouts
    enum_cache: Arc<EnumLayoutCache>,        // Enum layouts ✅
    struct_types: HashMap<String, StructLayout>,
    enum_types: HashMap<String, EnumLayout>, // Registered enums ✅
    calling_convention: CallingConvention,
    string_constants: Vec<StringConstant>,
    string_vreg_map: HashMap<usize, usize>,
}
```

**Key Methods**:
- `register_enum()` - Already exists! ✅
- `generate_function()` - Generates LLVM IR for functions
- `generate_terminator()` - Handles return, jump, branch, switch
- `generate_instruction()` - Handles all LIR instructions

### 2. Enum Layout Infrastructure ✅

**File**: `crates/zulon-codegen-llvm/src/enum_layout.rs`

**EnumLayout Structure**:
```rust
pub struct EnumLayout {
    pub name: String,
    pub variants: Vec<VariantInfo>,
    pub discriminant_type: LirTy,        // Type of discriminant (i8)
    pub discriminant_size: u64,          // Size of discriminant
    pub discriminant_offset: u64,        // Offset of discriminant (0)
    pub size: u64,                       // Total enum size
    pub align: u64,                      // Alignment
    pub data_offset: u64,                // Where data starts
}
```

**Key Methods**:
- `new()` - Create enum with discriminant type
- `add_variant()` - Add variant with discriminant value
- `finalize()` - Finalize layout

**This is perfect for Outcome<T, E>!**

### 3. Return Terminator Implementation

**Location**: `codegen.rs:706-734`

```rust
LirTerminator::Return(value) => {
    let ret_ty: LlvmType = func.return_type.clone().into();

    if let Some(vreg) = value {
        // Return with value
        writeln!(
            self.writer,
            "{}  ret {} %v{}",
            "  ".repeat(self.indent),
            ret_ty.to_llvm_ir(),
            vreg
        ).unwrap();
    } else {
        // Return without value
        // ...
    }
}
```

**For throw statements**: We'll need to construct an `Outcome::Err` variant before returning.

### 4. Load Instruction Implementation

**Location**: `codegen.rs:226-228`

```rust
LirInstruction::Load { dest, src, ty } => {
    self.generate_load(*dest, src, ty)?;
}
```

**Implementation**:
```rust
fn generate_load(&mut self, dest: VReg, src: &LirOperand, ty: &LirTy) -> Result<()> {
    let llvm_ty: LlvmType = ty.clone().into();
    let src_str = self.operand_to_llvm(src)?;

    writeln!(
        self.writer,
        "{}  %v{} = load {}, {}* {}",
        "  ".repeat(self.indent),
        dest,
        llvm_ty.to_llvm_ir(),
        llvm_ty.to_llvm_ir(),
        src_str
    ).unwrap();

    Ok(())
}
```

**For discriminant checking**: MIR already generates field access for discriminant, so this should work!

### 5. GEP (GetElementPtr) Instruction

**Location**: `codegen.rs:234-236`

```rust
LirInstruction::Gep { dest, base, indices, ty } => {
    self.generate_gep(*dest, *base, indices, ty)?;
}
```

**Critical for enum field access**: GEP computes pointer to discriminant and data fields.

---

## Implementation Strategy

### Phase 4.2: Register Outcome Type

**Approach**:
1. Create `EnumLayout` for Outcome<T, E>
2. Register with `codegen.register_enum()`
3. Generate LLVM type definition

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

### Phase 4.3: Throw Statement Codegen

**Current MIR**:
```
temp1 = DivideError::Zero
Return(temp1)
```

**Need to enhance Return terminator**:
- Check if function has error type
- If so, construct Outcome::Err variant
- Set discriminant to 1
- Store error value in data field
- Return the Outcome

### Phase 4.4: ? Operator Codegen

**Current MIR**:
```
temp1 = load outcome.discriminant  // GEP + Load
temp2 = (temp1 == 0)               // Cmp
if temp2 goto success else goto error  // Branch

success: temp3 = load outcome.data
error: temp4 = load outcome.data, return temp4
```

**This should already work!** The MIR generates:
- GEP for field access (discriminant, data)
- Load for reading values
- Cmp for comparison
- Branch for conditional control flow
- Return for early return

**Potential enhancement**: May need to ensure field names match.

---

## Success Criteria - Phase 4.1 ✅

- [x] Understand codegen structure (CodeGenerator)
- [x] Know where to add enum handling (enum_types HashMap)
- [x] Know how to generate LLVM IR instructions (terminator, instruction)
- [x] Understand EnumLayout infrastructure
- [x] Identify where to enhance Return terminator
- [x] Identify where discriminant checking happens

**All criteria met!** ✅

---

## Key Insights

`★ Insight ─────────────────────────────────────`

**1. Infrastructure Already Exists**:
The EnumLayout infrastructure is complete and ready to use. We don't need to build it from scratch.

**2. MIR Does Most of the Work**:
The MIR lowering already generates proper control flow for discriminant checking. The LLVM codegen just needs to translate it.

**3. Minimal Enhancement Needed**:
- Register Outcome<T, E> type (simple)
- Enhance Return terminator for throw (moderate)
- ? operator should work automatically (bonus!)

**4. Clear Path Forward**:
Each phase has a clear, achievable goal. The infrastructure is solid.
`─────────────────────────────────────────────────`

---

## Next Steps

### Phase 4.2: Register Outcome Type

**Task**: Create and register Outcome<T, E> enum layout

**File**: Likely in type lowering or codegen setup

**Time**: 30 minutes

**Approach**:
1. Find where enums are registered
2. Create Outcome<T, E> layout
3. Register with codegen

---

## Conclusion

**Phase 4.1 Status**: ✅ COMPLETE

**Understanding Achieved**:
- LLVM codegen architecture ✅
- Enum layout infrastructure ✅
- Return terminator implementation ✅
- Load/GEP instruction implementation ✅

**Ready for Phase 4.2**: Register Outcome<T, E> type

**Confidence**: HIGH - Infrastructure is excellent, clear path forward

---

**Document Version**: 1.0
**Date**: 2026-01-08
**Status**: ✅ Phase 4.1 Complete
**Next**: Phase 4.2 - Register Outcome Type
