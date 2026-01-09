#!/bin/bash
# Compile ZULON program to executable

set -e  # Exit on error

echo "=== ZULON Compilation Script ==="
echo ""

# Step 1: Generate LLVM IR
echo "[1/4] Generating LLVM IR..."
cargo run -p zulon-codegen-llvm --example full_to_llvm 2>&1 | \
    awk '/^define i32 @main/,/^$/' | \
    awk '/^define i32 @add/,/^ret i32 %v2/' | \
    head -n -1 > /tmp/main.ll

# Better: Extract all LLVM IR
cargo run -p zulon-codegen-llvm --example full_to_llvm 2>&1 | \
    awk '/^define i32 @add/,/^}$/' > /tmp/main.ll

echo "  ✅ LLVM IR saved to /tmp/main.ll"
echo ""

# Step 2: Compile LLVM IR to assembly
echo "[2/4] Compiling LLVM IR to assembly..."
llc /tmp/main.ll -o /tmp/main.s
echo "  ✅ Assembly saved to /tmp/main.s"
echo ""

# Step 3: Assemble to object file
echo "[3/4] Assembling to object file..."
clang -c /tmp/main.s -o /tmp/main.o
echo "  ✅ Object file saved to /tmp/main.o"
echo ""

# Step 4: Link and run
echo "[4/4] Linking and running..."
clang /tmp/main.o -o /tmp/zulon_program
echo "  ✅ Executable created: /tmp/zulon_program"
echo ""

echo "=== Running Program ==="
echo ""
/tmp/zulon_program
EXIT_CODE=$?
echo ""
echo "Program exited with code: $EXIT_CODE"

if [ $EXIT_CODE -eq 20 ]; then
    echo "✅ SUCCESS! (Expected: 15 + 5 = 20)"
else
    echo "⚠️  Unexpected result (expected 20)"
fi
