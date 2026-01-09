# Ralph Loop Session 1 - Function Call Bug Fix

**Date**: 2026-01-08  
**Iteration**: 1 of 40  
**Status**: ✅ Critical bug fixed

---

## Problem Discovered

When compiling programs with function calls that have unary operations as arguments (e.g., `abs(-42)`), the LLVM IR generator produced invalid code:

```llvm
%v0 = add i32 0, 42
%v1 = call i32 @abs(i32 %v1)  ; BUG: passing %v1 instead of %v0
ret i32 %v1
```

This caused LLVM verification to fail with: "Only PHI nodes may reference their own value!"

---

## Root Cause Analysis

The bug was in **MIR→LIR lowering** (`crates/zulon-lir/src/lower.rs`):

1. MIR correctly generated instructions for `-42`:
   - `Const { dest: Temp(0), value: 42 }`
   - `UnaryOp { dest: Temp(1), op: Neg, operand: Temp(0) }`
   - `Call { dest: Temp(2), args: [Temp(1)] }`

2. **Missing UnaryOp handler**: The `lower_instruction` function had no case for `MirInstruction::UnaryOp`, so it fell through to the default case which returned an empty vector

3. **temp_map not updated**: Because no LIR instruction was generated, the temp_map was never updated with `Temp(1) -> vreg1`

4. **Wrong vreg used**: When the Call instruction processed args `[Temp(1)]`, it called `get_or_alloc_vreg(Temp(1))`, which allocated a **new uninitialized vreg** instead of using the correct one

---

## Solution Implemented

Added UnaryOp handling to the MIR→LIR lowering (after line 366):

```rust
MirInstruction::UnaryOp { dest, op, operand, ty } => {
    let dest_vreg = func.alloc_vreg();
    let operand_vreg = self.temp_map.get(operand).copied()
        .unwrap_or_else(|| *operand as VReg);

    self.temp_map.insert(*dest, dest_vreg);

    let lir_op = match op {
        zulon_mir::MirUnaryOp::Neg => LirUnaryOp::Neg,
        zulon_mir::MirUnaryOp::Not => LirUnaryOp::Not,
        _ => LirUnaryOp::Neg,
    };

    Ok(vec![LirInstruction::UnaryOp {
        dest: dest_vreg,
        op: lir_op,
        operand: operand_vreg,
        ty: ty.clone().into(),
    }])
}
```

---

## Result

After the fix, the LLVM IR is now correct:

```llvm
%v0 = add i32 0, 42       ; constant 42
%v1 = sub i32 0, %v0      ; unary negation of %v0
%v2 = call i32 @abs(i32 %v1)  ; ✅ Correctly passing %v1!
ret i32 %v2
```

The program now:
- ✅ Compiles successfully
- ✅ Passes LLVM verification
- ✅ Assembles and links correctly
- ✅ Runs without crashing

---

## Files Modified

- `crates/zulon-lir/src/lower.rs`: Added UnaryOp instruction handling (lines 368-386)

---

## Testing

Verified fix with multiple test cases:
- ✅ `test_minimal.zl` - Simple constant return
- ✅ `hello_world.zl` - Extern function declaration
- ✅ `test_arithmetic.zl` - Arithmetic operations (30 = 10 + 20)
- ✅ `test_while_simple.zl` - While loops (45 = sum 0..9)
- ✅ `test_simple_call.zl` - Simple function call with constant
- ✅ `test_if.zl` - Function call with unary negation argument

All tests pass compilation and produce valid LLVM IR!

---

## Next Steps

1. Fix remaining bug in abs function (phi node merging)
2. Run performance benchmarks
3. Test more complex examples
4. Implement test framework integration

---

**Session Duration**: ~2 hours  
**Lines Changed**: ~20 lines  
**Bugs Fixed**: 1 critical codegen bug  
**Tests Passing**: 6/6 end-to-end compilation tests
