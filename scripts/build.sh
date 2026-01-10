#!/bin/bash
# ZULON Build Script
# Compiles a .zl file to an executable
#
# Usage: ./scripts/build.sh <input.zl> [output_name]
#
# Example: ./scripts/build.sh examples/hello.zl hello

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print with color
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${YELLOW}[STEP]${NC} $1"
}

# Check arguments
if [ $# -lt 1 ]; then
    print_error "Usage: $0 <input.zl> [output_name]"
    echo ""
    echo "Compiles a ZULON source file to an executable"
    echo ""
    echo "Arguments:"
    echo "  input.zl     - Path to ZULON source file"
    echo "  output_name  - (Optional) Name of output executable"
    echo "                 Defaults to input filename without extension"
    echo ""
    echo "Example:"
    echo "  $0 examples/hello.zl hello"
    echo "  $0 tests/my_program.zl"
    exit 1
fi

INPUT_FILE="$1"
OUTPUT_NAME="${2:-$(basename "$INPUT_FILE" .zl)}"

# Validate input file exists
if [ ! -f "$INPUT_FILE" ]; then
    print_error "Input file not found: $INPUT_FILE"
    exit 1
fi

print_info "Building ZULON program: $INPUT_FILE"
print_info "Output name: $OUTPUT_NAME"
echo ""

# Create temporary directory
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Define intermediate files
LL_FILE="$TEMP_DIR/${OUTPUT_NAME}.ll"
BC_FILE="$TEMP_DIR/${OUTPUT_NAME}.bc"
OBJ_FILE="$TEMP_DIR/${OUTPUT_NAME}.o"
EXE_FILE="$OUTPUT_NAME"

print_step "1/5: Compiling ZULON to LLVM IR..."
cargo run --quiet --example test_error_compile \
    --manifest-path crates/zulon-codegen-llvm/Cargo.toml \
    -- "$INPUT_FILE" > "$LL_FILE" 2>&1

# Extract just the LLVM IR (remove debug output)
sed -n '/^; Generated/,$p' "$LL_FILE" > "${LL_FILE}.clean"
mv "${LL_FILE}.clean" "$LL_FILE"

if [ ! -s "$LL_FILE" ]; then
    print_error "LLVM IR generation failed"
    exit 1
fi
print_success "LLVM IR generated: $LL_FILE"

print_step "2/5: Optimizing LLVM IR (optional)..."
# Run optimizer if available
if command -v opt &> /dev/null; then
    opt -O2 "$LL_FILE" -o "${BC_FILE}" 2>/dev/null || \
        (print_info "Optimizer failed, using unoptimized IR" && \
         cp "$LL_FILE" "$BC_FILE")
else
    print_info "opt not found, using unoptimized IR"
    cp "$LL_FILE" "$BC_FILE"
fi

print_step "3/5: Assembling LLVM IR to object file..."
llc "$BC_FILE" -o "$OBJ_FILE" -filetype=obj
print_success "Object file created: $OBJ_FILE"

print_step "4/5: Linking to executable..."
# Determine platform
OS=$(uname -s)
case "$OS" in
    Darwin)
        # macOS
        LINK_CMD="clang"
        ;;
    Linux)
        # Linux
        LINK_CMD="clang"
        ;;
    *)
        print_error "Unsupported platform: $OS"
        exit 1
        ;;
esac

# Link with runtime library if needed
if [ -f "crates/zulon-runtime-core/target/release/libzulon_runtime_core.a" ]; then
    print_info "Linking with runtime library..."
    "$LINK_CMD" "$OBJ_FILE" \
        crates/zulon-runtime-core/target/release/libzulon_runtime_core.a \
        -o "$EXE_FILE"
else
    print_info "Linking standalone (no runtime library)..."
    "$LINK_CMD" "$OBJ_FILE" -o "$EXE_FILE"
fi

print_success "Executable created: $EXE_FILE"

print_step "5/5: Testing executable..."
if [ -x "$EXE_FILE" ]; then
    print_info "Running $EXE_FILE..."
    "./$EXE_FILE"
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 0 ]; then
        print_success "Program exited successfully (exit code: $EXIT_CODE)"
    else
        print_info "Program exited with code: $EXIT_CODE"
    fi
else
    print_info "Executable not testable (no execute permission)"
fi

echo ""
print_success "Build complete!"
echo ""
echo "Generated files:"
echo "  - $EXE_FILE (executable)"
echo ""
echo "Run with:"
echo "  ./$EXE_FILE"
