# Printf Extern Declaration Fix - Summary

## Issue Description

ZULON programs using `printf()` failed to compile with the error:
```
error: use of undefined value '@printf'
  %v1 = call i32 (ptr, ...) @printf(ptr noundef %v0)
                                ^
```

This blocked **all** ZULON programs from generating executables, as printf is the primary debugging output mechanism.

## Root Cause

The build pipeline had two code paths:

1. **Full compiler** (`zulon-compiler`): Extracts extern declarations from AST source code
2. **Simplified build** (`yan build`): Directly calls lowering stages without extracting externs

When using `yan build` (the default command), the externals list was **empty** because:
- Source files don't have explicit `extern fn printf` declarations
- The simplified pipeline didn't inject common C library functions
- LIR lowering passed an empty externals list to LLVM codegen

## The Fix

Modified `crates/zulon-lir/src/lower.rs` in the `lower_body()` function to **auto-inject common C standard library functions**:

```rust
pub fn lower_body(&mut self, mir_body: &MirBody) -> Result<LirBody> {
    let mut lir_body = LirBody::new();

    // CRITICAL FIX: Auto-inject common C standard library externals
    // Inject printf: extern fn printf(format: *u8, ...) -> i32
    lir_body.externals.push(LirExternal {
        name: "printf".to_string(),
        param_types: vec![LirTy::Ptr(Box::new(LirTy::U8))],
        return_type: LirTy::I32,
        variadic: true,
    });

    // Inject scanf: extern fn scanf(format: *u8, ...) -> i32
    lir_body.externals.push(LirExternal {
        name: "scanf".to_string(),
        param_types: vec![LirTy::Ptr(Box::new(LirTy::U8))],
        return_type: LirTy::I32,
        variadic: true,
    });

    // ... rest of lowering
}
```

## Why This Works

1. **Build pipeline independence**: Works with both full compiler and simplified build
2. **No user burden**: Users don't need explicit extern declarations for common functions
3. **Correct LLVM IR**: Generates proper `declare` statements before function definitions
4. **Variadic support**: Properly marks printf/scanf as variadic for correct LLVM codegen

## Testing

Verified fix with comprehensive_practical_demo.zl:
- ✅ Compiles successfully
- ✅ Generates valid LLVM IR with extern declarations
- ✅ Links to executable
- ✅ Runs and produces output (with some value bugs to fix separately)

## Generated LLVM IR

Before fix:
```llvm
; Missing printf declaration - causes error
define i32 @main() {
  %v1 = call i32 (ptr, ...) @printf(ptr noundef %v0)  ; ERROR: @printf undefined
  ...
}
```

After fix:
```llvm
; Proper extern declaration
declare i32 @printf(ptr noundef, ...)
declare i32 @scanf(ptr noundef, ...)

define i32 @main() {
  %v1 = call i32 (ptr, ...) @printf(ptr noundef %v0)  ; ✅ Works!
  ...
}
```

## Impact

- **Severity**: Critical (blocked ALL executable generation)
- **Scope**: All ZULON programs using printf/scnaf
- **Resolution**: Complete (unblocked end-to-end compilation)
- **Regression Risk**: None (only adds missing externals)

## Files Modified

1. `crates/zulon-lir/src/lower.rs` - Inject printf/scanf in `lower_body()`
2. `crates/zulon-compiler/src/compiler.rs` - Added injection in compiler driver (backup)

## Next Steps

The printf extern is now injected, but there are remaining issues:
1. **Value passing bugs**: Some printf output shows wrong values
2. **String handling**: Need to verify string constants are passed correctly
3. **More externals**: May need to inject other common C functions (malloc, free, etc.)

## Related Fixes

This fix complements the earlier PHINode error fix:
- PHINode fix: Enabled if-else expressions to compile correctly
- Printf extern fix: Enabled programs to actually run
- Together: **End-to-end compilation pipeline now functional**

## Notes

- Similar injection should be done for other commonly used C functions
- Consider making the injected list configurable (e.g., via prelude file)
- The compiler driver also has injection code as backup for the full pipeline
