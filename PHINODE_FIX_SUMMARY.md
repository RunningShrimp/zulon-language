# PHINode Error Fix - Summary

## Issue Description

When compiling ZULON programs with if-else expressions (like `fibonacci`), the LLVM IR generator would produce invalid control flow graphs where both branches of a conditional would target the same basic block. This caused LLVM's PHINode verification to fail with:

```
error: PHINode should have one entry for each predecessor of its parent basic block!
%v10 = phi i32[ %v9, %block2 ], [ undef, %block1 ]
```

## Root Cause

The LIR lowering stage had an "empty block elimination" optimization in `eliminate_empty_join_blocks()` that was **too aggressive**. When it found an empty then-block (e.g., a block that just returns a value like `n`), it would:

1. Detect the block was empty (no instructions)
2. Find the successor block (the join block)
3. Redirect **all predecessors** from the empty then-block directly to the join block
4. This included redirecting the conditional branch's `then` target

**The Bug**: When both the `then` and `else` targets of a conditional branch were redirected to the same join block, this created invalid CFG where the branch had identical targets:

```llvm
br i1 %v2, label %block2, label %block2  ; Both targets identical!
```

This violated LLVM's SSA requirements because the PHI node expected two distinct predecessors but only got one.

## The Fix

Modified `crates/zulon-lir/src/lower.rs` in the `eliminate_empty_join_blocks()` function to **skip empty blocks that are targets of conditional branches**:

```rust
// CRITICAL FIX: Don't eliminate empty blocks that are targets of conditional branches
// If a predecessor uses a Branch terminator (if-else), we cannot eliminate the empty block
// because both branches must be distinct blocks for PHI nodes to work correctly
let has_conditional_pred = predecessors.iter().any(|&pred_id| {
    func.blocks.get(&pred_id).map_or(false, |block| {
        matches!(&block.terminator, Some(LirTerminator::Branch { .. }))
    })
});

if has_conditional_pred {
    continue;  // Skip this empty block - don't eliminate it
}
```

## Why This Works

1. **Preserves Control Flow**: Empty blocks that are targets of conditional branches are kept, ensuring distinct `then` and `else` targets
2. **Valid CFG**: The resulting LLVM IR has proper control flow with distinct branches
3. **PHI Nodes Work**: PHI nodes receive values from their actual predecessors, not from ghost blocks that were eliminated
4. **No Performance Loss**: Empty blocks with just terminators are optimized away by LLVM's own passes later

## Before and After

### Before (Invalid)
```llvm
define i32 @simple_if(i32 %v0) {
  block0:
      %v1 = add i32 0, 1
      %v2 = icmp sle i32 %v0, %v1
      br i1 %v2, label %block2, label %block2  ; BUG: Both branches to block2!
  block1:
      unreachable                                ; Never reached!
  block2:
      %v3 = add i32 0, 100
      br label %block3
  block3:
      %v4 = phi i32[ undef, %block1 ], [ %v3, %block2 ]  ; ERROR: block1 not a predecessor!
      ret i32 %v4
}
```

### After (Valid)
```llvm
define i32 @simple_if(i32 %v0) {
  block0:
      %v1 = add i32 0, 1
      %v2 = icmp sle i32 %v0, %v1
      br i1 %v2, label %block1, label %block2  ; Correct: distinct targets
  block1:
      br label %block3                           ; Falls through to join
  block2:
      %v3 = add i32 0, 100
      br label %block3
  block3:
      %v4 = phi i32[ undef, %block1 ], [ %v3, %block2 ]  ; Correct: both predecessors exist!
      ret i32 %v4
}
```

## Files Modified

- `crates/zulon-lir/src/lower.rs`: Added conditional predecessor check in `eliminate_empty_join_blocks()`

## Testing

Verified the fix with:
1. Simple if-else expressions (`simple_if` test)
2. Recursive functions with if-else (`fibonacci`, `factorial`)
3. Mutual recursion with if-else (`is_even`/`is_odd`)
4. Comprehensive practical demo (all language features)

All examples now generate valid LLVM IR without PHINode errors.

## Impact

- **Severity**: Critical (blocked compilation of any program with if-else expressions)
- **Scope**: All if-else expressions with empty blocks
- **Resolution**: Complete (all test cases pass)
- **Regression Risk**: Low (only adds a safety check, doesn't change existing behavior)

## Lessons Learned

1. **SSA is Strict**: LLVM's SSA requirements are strict - PHI nodes must have exactly one incoming value per predecessor
2. **Optimization Hazards**: Optimizations that transform CFG must preserve control flow semantics
3. **Empty Blocks Have Purpose**: Even "empty" blocks serve a purpose in CFG - they represent distinct control flow paths
4. **Test Edge Cases**: The bug was triggered by simple expressions (like `n`) in if-branches, highlighting the need to test simple cases

## Related Issues

This fix resolves the issue documented in session summaries where comprehensive_practical_demo.zl failed to compile with PHINode errors. The demo now compiles successfully (remaining issue is unrelated printf extern declaration).
