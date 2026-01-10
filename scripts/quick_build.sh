#!/bin/bash
# Quick ZULON Build Script
# Simplified version that works with current toolchain

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

INPUT="$1"
OUTPUT="${2:-$(basename "$1" .zl)}"

echo -e "${YELLOW}[ZULON Builder]${NC} Building: $INPUT"
echo ""

# Step 1: Generate LLVM IR
echo "[1/4] Generating LLVM IR..."
cargo run --quiet --example test_error_compile \
    --manifest-path crates/zulon-codegen-llvm/Cargo.toml \
    -- "$INPUT" > "${OUTPUT}.ll" 2>&1 || echo "Note: Compilation output above"

# Extract LLVM IR
sed -n '/^; Generated/,$p' "${OUTPUT}.ll" > "${OUTPUT}.tmp"
mv "${OUTPUT}.tmp" "${OUTPUT}.ll"
echo -e "${GREEN}✓${NC} LLVM IR: ${OUTPUT}.ll"

# Step 2: Assemble to object
echo "[2/4] Assembling to object file..."
llc "${OUTPUT}.ll" -o "${OUTPUT}.o" -filetype=obj
echo -e "${GREEN}✓${NC} Object: ${OUTPUT}.o"

# Step 3: Link to executable
echo "[3/4] Linking to executable..."
clang "${OUTPUT}.o" -o "$OUTPUT"
echo -e "${GREEN}✓${NC} Executable: $OUTPUT"

# Step 4: Test run
echo "[4/4] Testing executable..."
if [ -x "$OUTPUT" ]; then
    ./"$OUTPUT"
    EXIT_CODE=$?
    echo ""
    if [ $EXIT_CODE -eq 0 ]; then
        echo -e "${GREEN}✓${NC} Program exited successfully (code: $EXIT_CODE)"
    else
        echo -e "${YELLOW}!${NC} Program exited with code: $EXIT_CODE"
    fi
fi

echo ""
echo -e "${GREEN}Build complete!${NC}"
echo "Run with: ./$OUTPUT"
