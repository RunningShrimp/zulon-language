# ZULON Build Scripts

This directory contains utility scripts for building ZULON programs.

## Available Scripts

### quick_build.sh

**Purpose**: Quickly compile a ZULON program to an executable

**Usage**:
```bash
./scripts/quick_build.sh <input.zl> [output_name]
```

**Example**:
```bash
# Build with default output name
./scripts/quick_build.sh examples/00_hello_world.zl

# Build with custom output name
./scripts/quick_build.sh examples/00_hello_world.zl hello

# Run the executable
./hello
```

**What It Does**:
1. Generates LLVM IR from ZULON source
2. Assembles LLVM IR to object file
3. Links object file to executable
4. Runs the executable to test it

**Requirements**:
- Rust and Cargo
- LLVM tools (llc, clang)
- ZULON compiler built

### build.sh

**Purpose**: Full-featured build script with more options

**Note**: Currently in development, use quick_build.sh for now

## Quick Start

1. Make sure ZULON is built:
```bash
cargo build --release
```

2. Build a program:
```bash
./scripts/quick_build.sh examples/00_hello_world.zl
```

3. Run it:
```bash
./00_hello_world
```

## Examples

### Hello World
```bash
./scripts/quick_build.sh examples/00_hello_world.zl hello
./hello
```

### Error Handling Test
```bash
./scripts/quick_build.sh tests/error_handling_integration_test.zl test
./test
```

### Extended Test
```bash
./scripts/quick_build.sh tests/error_handling_extended_test.zl extended
./extended
```

## Troubleshooting

### "command not found: llc"

Install LLVM:
```bash
# macOS
brew install llvm

# Linux
sudo apt-get install llvm
```

### "cargo: no such subcommand: --example"

Make sure you're in the ZULON root directory.

### Build fails

Check that:
1. ZULON compiler is built (`cargo build --release`)
2. LLVM tools are installed (`llc --version`)
3. Input file exists (`ls examples/*.zl`)

## Contributing

To add new build scripts:

1. Follow the existing naming convention
2. Add usage examples
3. Include error handling
4. Update this README

## See Also

- **Compilation Guide**: `../docs/COMPILATION_GUIDE.md`
- **Error Handling**: `../docs/ERROR_HANDLING_GUIDE.md`
- **Implementation Plan**: `../IMPLEMENTATION_PLAN.md`
