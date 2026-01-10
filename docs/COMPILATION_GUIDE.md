# ZULON Compilation Guide

**Last Updated**: 2026-01-10
**ZULON Version**: 0.1.0 (MVP)

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Compilation Pipeline](#compilation-pipeline)
3. [Building Programs](#building-programs)
4. [Advanced Options](#advanced-options)
5. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Prerequisites

You need the following tools installed:

1. **Rust** (1.70+)
   ```bash
   rustc --version
   cargo --version
   ```

2. **Clang/LLVM**
   ```bash
   clang --version
   llc --version
   opt --version  # Optional, for optimization
   ```

3. **ZULON Compiler**
   ```bash
   # Build the compiler
   cargo build --release
   ```

### Your First Build

```bash
# Navigate to ZULON directory
cd /path/to/zulon-language

# Build a simple program
./scripts/build.sh examples/00_hello_world.zl hello

# Run the executable
./hello
```

---

## Compilation Pipeline

ZULON programs go through several compilation stages:

```
.zl source
    ↓
[Parser] → AST (Abstract Syntax Tree)
    ↓
[HIR] → High-level IR
    ↓
[MIR] → Mid-level IR
    ↓
[LIR] → Low-level IR (SSA form)
    ↓
[LLVM Codegen] → LLVM IR (.ll files)
    ↓
[llc] → Assembly (.s files)
    ↓
[clang] → Object file (.o files)
    ↓
[clang] → Executable binary
```

### Stage Details

1. **Parsing**: Converts source code to AST
2. **HIR**: High-level Intermediate Representation
3. **MIR**: Mid-level IR with control flow
4. **LIR**: Low-level IR in SSA form
5. **LLVM Codegen**: Generates LLVM IR
6. **llc**: LLVM assembler (IR → machine code)
7. **clang**: Linker (object files → executable)

---

## Building Programs

### Using the Build Script (Recommended)

The easiest way to build ZULON programs:

```bash
./scripts/build.sh <input.zl> [output_name]
```

**Examples**:

```bash
# Build with default output name
./scripts/build.sh examples/00_hello_world.zl

# Build with custom output name
./scripts/build.sh examples/00_hello_world.zl myprogram

# Build test program
./scripts/build.sh tests/error_handling_extended_test.zl test
```

### Manual Compilation

For more control, you can compile manually:

#### Step 1: Generate LLVM IR

```bash
cargo run --example test_error_compile \
    --manifest-path crates/zulon-codegen-llvm/Cargo.toml \
    -- path/to/program.zl > program.ll
```

#### Step 2: Extract LLVM IR

```bash
sed -n '/^; Generated/,$p' program.ll > program_clean.ll
```

#### Step 3: Optimize (Optional)

```bash
opt -O2 program_clean.ll -o program.bc
```

#### Step 4: Assemble to Object

```bash
llc program.bc -o program.o -filetype=obj
```

#### Step 5: Link to Executable

```bash
clang program.o -o program
```

#### Step 6: Run

```bash
./program
```

---

## Advanced Options

### Optimization Levels

The build script uses `-O2` optimization by default. To change:

**No optimization** (faster compile, slower code):
```bash
opt -O0 program.ll -o program.bc
```

**Aggressive optimization** (slower compile, faster code):
```bash
opt -O3 program.ll -o program.bc
```

### Custom LLVM Flags

Pass flags to `llc`:

```bash
llc program.ll -o program.s \
    -march=native \  # Optimize for your CPU
    -O3             # Optimization level
```

### Linking with Runtime

If using the runtime library:

```bash
clang program.o \
    crates/zulon-runtime-core/target/release/libzulon_runtime_core.a \
    -o program
```

### Debug Builds

For debugging:

```bash
# Generate unoptimized LLVM IR
cargo run --example test_error_compile \
    --manifest-path crates/zulon-codegen-llvm/Cargo.toml \
    -- program.zl > program.ll

# Add debug symbols
llc program.ll -o program.s -g

# Link with debug info
clang program.o -o program -g
```

---

## Troubleshooting

### "llc: error: expected top-level entity"

**Problem**: LLVM IR file has extra text

**Solution**: Extract just the LLVM IR:
```bash
sed -n '/^; Generated/,$p' program.ll > program_clean.ll
```

### "undefined reference to..."

**Problem**: Missing runtime library

**Solution**: Link with runtime:
```bash
clang program.o \
    crates/zulon-runtime-core/target/release/libzulon_runtime_core.a \
    -o program
```

### "command not found: llc"

**Problem**: LLVM tools not installed

**Solution**:
```bash
# macOS
brew install llvm

# Linux
sudo apt-get install llvm

# Add to PATH
export PATH="/opt/homebrew/opt/llvm/bin:$PATH"
```

### Build Script Fails

**Problem**: Build script exits with error

**Solution**: Check each step manually:
```bash
# 1. Does LLVM IR generate?
cargo run --example test_error_compile \
    --manifest-path crates/zulon-codegen-llvm/Cargo.toml \
    -- program.zl

# 2. Does llc work?
llc program.ll -o program.s

# 3. Does clang work?
clang program.s -o program
```

---

## Build Examples

### Example 1: Hello World

```bash
./scripts/build.sh examples/00_hello_world.zl hello
./hello
```

**Expected Output**:
```
Hello, World!
```

### Example 2: Error Handling

```bash
./scripts/build.sh tests/error_handling_integration_test.zl test
./test
```

**Expected Output**:
```
Program exits with code 0
```

### Example 3: Extended Test

```bash
./scripts/build.sh tests/error_handling_extended_test.zl extended
./extended
```

**Expected Output**:
```
Program exits with code 0
```

---

## Performance Tips

### 1. Enable Optimizations

Always use optimization for production builds:
```bash
opt -O2 program.ll -o program.bc
```

### 2. Use Target-Specific Optimization

```bash
llc program.bc -o program.s -march=native
```

### 3. Link-Time Optimization (LTO)

```bash
clang program.o -o program -flto
```

### 4. Strip Symbols (Smaller Binary)

```bash
clang program.o -o program -s
```

---

## Cross-Compilation

### Target Different Architectures

```bash
# For x86_64
llc program.ll -o program.s -mtriple=x86_64-unknown-linux-gnu

# For ARM64
llc program.ll -o program.s -mtriple=aarch64-unknown-linux-gnu

# For WebAssembly
llc program.ll -o program.s -mtriple=wasm32-unknown-unknown
```

---

## Integration with Build Systems

### Makefile

```makefile
.PHONY: all clean run

all: program

program: program.zl
	./scripts/build.sh program.zl

run: program
	./program

clean:
	rm -f program program.ll program.bc program.o
```

### CMake

```cmake
add_custom_command(OUTPUT program.ll
    COMMAND cargo run --example test_error_compile
        --manifest-path ${CMAKE_SOURCE_DIR}/crates/zulon-codegen-llvm/Cargo.toml
        -- ${CMAKE_CURRENT_SOURCE_DIR}/program.zl
        > program.ll
    DEPENDS program.zl
    VERBATIM
)

add_custom_command(OUTPUT program
    COMMAND clang program.o -o program
    DEPENDS program.ll
    VERBATIM
)

add_custom_target(program ALL DEPENDS program)
```

---

## Advanced: Pipeline Customization

### Custom Build Pipeline

Create a custom build script:

```bash
#!/bin/bash
INPUT="$1"
OUTPUT="$2"

# Stage 1: Parse
echo "[1/6] Parsing..."
cargo run --bin parser -- "$INPUT" > ast.json

# Stage 2: HIR
echo "[2/6] Lowering to HIR..."
cargo run --bin hir_lower -- ast.json > hir.json

# Stage 3: MIR
echo "[3/6] Lowering to MIR..."
cargo run --bin mir_lower -- hir.json > mir.json

# Stage 4: LIR
echo "[4/6] Lowering to LIR..."
cargo run --bin lir_lower -- mir.json > lir.json

# Stage 5: LLVM IR
echo "[5/6] Generating LLVM IR..."
cargo run --bin codegen -- lir.json > "$OUTPUT.ll"

# Stage 6: Binary
echo "[6/6] Compiling to binary..."
llc "$OUTPUT.ll" -o "$OUTPUT.s"
clang "$OUTPUT.s" -o "$OUTPUT"
```

---

## Best Practices

### 1. Always Test After Build

```bash
./scripts/build.sh program.zl program && ./program
```

### 2. Keep Build Artifacts

```bash
mkdir build
./scripts/build.sh program.zl build/program
```

### 3. Use Verbose Output for Debugging

```bash
# Add -v to build script
./scripts/build.sh -v program.zl
```

### 4. Clean Build Periodically

```bash
cargo clean
rm -rf build/
```

---

## Current Limitations

### 1. No Automatic Dependencies

**Issue**: Build script doesn't track dependencies

**Workaround**: Rebuild when dependencies change:
```bash
touch program.zl
./scripts/build.sh program.zl
```

### 2. No Incremental Compilation

**Issue**: Always rebuilds everything

**Workaround**: Use build system (Make/CMake) for incremental builds

### 3. No Standard Library Linking

**Issue**: Standard library not automatically linked

**Workaround**: Manually link required libraries

---

## Next Steps

As ZULON matures, we'll add:

1. **YAN Build Tool** - Integrated build system
2. **Package Manager** - Dependency management
3. **Incremental Builds** - Faster compilation
4. **Build Cache** - Reuse unchanged components
5. **Parallel Compilation** - Build multiple files at once

---

## Resources

- **Implementation Plan**: `IMPLEMENTATION_PLAN.md`
- **Build System Design**: `PHASE_1_7_YAN_TOOL_COMPLETE.md`
- **Error Handling**: `docs/ERROR_HANDLING_GUIDE.md`
- **LLVM Documentation**: https://llvm.org/docs/

---

## Getting Help

### Build Issues

Check the troubleshooting section above or:

1. Check LLVM installation: `llc --version`
2. Check Rust installation: `cargo --version`
3. Check Clang installation: `clang --version`

### Compiler Issues

If a program fails to compile:

1. Check error messages carefully
2. Verify syntax is correct
3. Test with simpler programs first
4. Check `RALPH_LOOP_*_COMPLETE.md` for known issues

### Runtime Issues

If a program compiles but crashes:

1. Run with debugger: `lldb program`
2. Check for segmentation faults
3. Verify memory operations
4. Check error handling logic

---

**Happy Compiling!** 🚀

ZULON's compilation pipeline is designed to be:
- **Simple**: One command to build
- **Fast**: Efficient code generation
- **Flexible**: Manual control when needed
- **Transparent**: Each stage is observable

*Generated: 2026-01-10*
*Version: 0.1.0 MVP*
*Status: Build script available* ✅
