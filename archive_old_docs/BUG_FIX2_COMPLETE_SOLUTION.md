# Bug #2 Complete Analysis and Solution Plan

**Date**: 2026-01-10
**Status**: Root cause identified, solution designed but not implemented
**Priority**: P0 (Blocking)

---

## Executive Summary

Bug #2 involves nested `if` statements without `else` branches generating empty join blocks that get marked as `unreachable`, preventing execution of subsequent code.

**Current State**: Partially fixed
- ✅ HIR type inference: Correctly types if statements as Unit
- ✅ MIR lowering: Tracks else_final_block, adds Gotot to join
- ⚠️ Remaining: Empty join blocks still created, causing unreachable

---

## Problem Statement

### Test Case
```rust
fn check_prime(n: i32) -> i32 {
    let mut result = 0;
    let mut i = 2;
    let mut found_divisor = 0;

    while i < n {
        if n % i == 0 {
            found_divisor = 1;
            i = n;
        } else {
            i = i + 1;
        }
    }

    if found_divisor == 0 {        // outer if
        if n > 1 {                 // inner if (Unit type, no else)
            result = 1;
        }
    }

    result  // ← Never reached!
}
```

### Expected vs Actual
- **Expected**: check_prime(2) = 1, check_prime(3) = 1, check_prime(4) = 0
- **Actual**: check_prime(2) = 0, check_prime(3) = 0, check_prime(4) = 0 ❌

### Generated LLVM IR
```llvm
block10:  // inner if then
  store i32 1, i32* %result
  br label %block12
block11:  // inner if else (implicit)
  br label %block12
block12:  // inner join block
  unreachable  ← Should be fallthrough to return!
}
```

---

## Root Cause Analysis

### MIR Lowering Flow

For `if n > 1 { result = 1; }` (Unit type, no else):

1. **Allocate blocks** (line 532-538 in `crates/zulon-mir/src/lower.rs`):
   ```rust
   let then_block_id = func.alloc_block();   // block7
   let else_block_id = func.alloc_block();   // block8 (implicit)
   let join_block_id = func.alloc_block();    // block12 ← Created!
   ```

2. **Lower then block**: Contains `If { condition: ..., then: block10, else: block11 }`
   - block10: sets result, Goto to block12
   - block11: empty, Goto to block12

3. **Set continuation** (line 622):
   ```rust
   *current_block = join_block_id;  // block12
   ```

4. **Problem**: block12 is empty! Subsequent statements (`result`) should be lowered into block12, but lowering returns before processing them.

### Why LIR Elimination Failed

Attempted to eliminate empty join blocks in LIR, but:
1. Block 12 is the **last block** - no successor to redirect to
2. Breaking SSA: "Instruction does not dominate all uses!"
3. The return statement needs to be in block 12, but it's empty

---

## Solution Design

### Option 1: Don't Create Join Block ⭐ **RECOMMENDED**

**Location**: `crates/zulon-mir/src/lower.rs:529-650`

**Logic**:
```rust
HirExpression::If { condition, then_block, else_block, ty, .. } => {
    let mir_ty: MirTy = ty.clone().into();
    let is_unit_statement = matches!(mir_ty, MirTy::Unit);
    let has_explicit_else = else_block.is_some();

    // Special case: Unit-type if without else branch
    if is_unit_statement && !has_explicit_else {
        // Don't create join block
        // Then branch: continues to subsequent code
        // Else branch: jumps to continuation (skipping then)

        let cond_temp = self.lower_expression(func, current_block, condition)?;
        let then_block_id = func.alloc_block();
        let continuation_block = func.alloc_block();  // Instead of join

        // Set conditional branch
        block_obj.set_terminator(MirTerminator::If {
            condition: cond_temp,
            then_block: then_block_id,
            else_block: continuation_block,  // Skip to continuation
        });

        // Lower then block (no terminator, falls through)
        *current_block = then_block_id;
        let (then_final, _) = self.lower_block(func, then_block, then_block_id, false)?;

        // Then block falls through to continuation automatically
        *current_block = continuation_block;

        // Lower implicit else (just a placeholder)
        // It already branches to continuation_block

        return Ok(func.alloc_temp());
    }

    // ... rest of existing logic for explicit else or value-producing ifs
}
```

**Key Changes**:
1. Allocate `continuation_block` instead of `join_block`
2. Else branch (implicit) directly targets `continuation_block`
3. Then block falls through to `continuation_block`
4. Subsequent statements are lowered into `continuation_block`

**Pros**:
- No empty join blocks
- Clean fallthrough
- Maintains SSA properties

**Cons**:
- Requires refactoring existing if-else lowering
- Need to handle fallthrough correctly

### Option 2: Populate Join Block

Instead of returning immediately after setting `*current_block = join_block_id`, continue lowering subsequent statements into the join block.

**Pros**: Join block won't be empty
**Cons**: Complex interaction with `lower_block` return value

### Option 3: Special Case in LIR

Handle last-block case specially in LIR elimination:
- If empty block is last block, move Return from previous block into it
- Or merge it with predecessor blocks

**Pros**: Can fix in LIR only
**Cons**: Still complex, SSA issues remain

---

## Implementation Plan

### Phase 1: MIR Fix (Option 1)

**File**: `crates/zulon-mir/src/lower.rs`

**Steps**:
1. Extract the special case before the general if-else logic
2. Handle Unit-type if without else:
   - Allocate `continuation_block`
   - Set If terminator to branch then/continuation
   - Lower then block (no fallthrough terminator needed)
   - Set `*current_block = continuation_block`
   - Return
3. Keep existing logic for all other cases

**Testing**:
- `test_prime_debug.zl` should output correct values
- All existing examples must still pass
- No SSA validation errors

### Phase 2: Verification

1. Test simple if statement:
   ```rust
   if 1 == 1 {
       result = 42;
   }
   result  // Should return 42
   ```

2. Test nested if without else:
   ```rust
   if true {
       if true {
           result = 1;
       }
   }
   result  // Should return 1
   ```

3. Test if with explicit else:
   ```rust
   if cond {
       result = 1;
   } else {
       result = 2;
   }
   result  // Should work as before
   ```

### Phase 3: LIR Cleanup

Remove or disable the `eliminate_empty_join_blocks` function since it won't be needed after MIR fix.

---

## Files to Modify

1. **Primary**: `crates/zulon-mir/src/lower.rs:529-650`
   - Implement Option 1 for Unit if without else

2. **Secondary**: `crates/zulon-lir/src/lower.rs:354-434`
   - Can disable `eliminate_empty_join_blocks` or keep as safety net

3. **Tests**:
   - `test_prime_debug.zl`
   - `test_nested_if.zl`
   - All examples in `examples/`

---

## Risk Assessment

**Low Risk**:
- Change is isolated to specific case (Unit + no else)
- Existing code paths unchanged
- Clear testing strategy

**Medium Risk**:
- Refactoring complex MIR lowering logic
- Potential for breaking other control flow

**Mitigation**:
- Comprehensive testing before/after
- Keep existing logic as fallback
- Add debug output during development

---

## Success Criteria

1. ✅ `test_prime_debug.zl` outputs:
   - check_prime(2) = 1
   - check_prime(3) = 1
   - check_prime(4) = 0

2. ✅ All existing examples pass:
   - fibonacci.zl (recursive and iterative)
   - factorial.zl (recursive and iterative)
   - test_nested_if.zl
   - All other examples

3. ✅ No SSA validation errors
4. ✅ No regressions in other control flow

---

## Timeline Estimate

- **Design**: 1 hour (completed)
- **Implementation**: 2-3 hours
- **Testing**: 1-2 hours
- **Total**: 4-6 hours

---

## References

- **HIR Type Fix**: `crates/zulon-typeck/src/checker.rs:836-844` ✅ Complete
- **MIR Lowering**: `crates/zulon-mir/src/lower.rs:529-650` ← To fix
- **LIR Lowering**: `crates/zulon-lir/src/lower.rs:291-352`
- **Test Cases**: `test_prime_debug.zl`, `test_nested_if.zl`

---

**Status**: 🔄 Ready for implementation
**Next Action**: Implement Option 1 in MIR lowering
**Assignee**: ZULON Development Team

**Last Updated**: 2026-01-10
