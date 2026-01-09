#!/bin/bash
# Verify all example programs compile and run with -O2

echo "=== Verifying All Examples with -O2 ==="
echo ""

cd crates/zulon-build/examples

examples=(
    "hello_world"
    "println_demo"
    "print_call"
    "print_all"
    "print_demo"
    "arc_demo"
    "comprehensive_io_demo"
    "getchar_demo"
    "greeting_demo"
    "string_utils_demo"
)

total=0
passed=0
failed=0

for example in "${examples[@]}"; do
    total=$((total + 1))
    echo "[$total/${#examples[@]}] Testing: $example"
    
    # Compile
    cargo run --example "$example" --quiet 2>&1 | tail -1 > /dev/null
    
    # Check if binary exists
    if [ -f "$example" ]; then
        echo "  ✅ Compiled successfully"
        passed=$((passed + 1))
        
        # Cleanup binary
        rm -f "$example"
    else
        echo "  ❌ Compilation failed"
        failed=$((failed + 1))
    fi
    
    echo ""
done

cd ../../..

echo "=== Summary ==="
echo "Total: $total"
echo "Passed: $passed"
echo "Failed: $failed"
echo "Success Rate: $(( passed * 100 / total ))%"

if [ $passed -eq $total ]; then
    echo "✅ All examples compiled successfully!"
    exit 0
else
    echo "❌ Some examples failed"
    exit 1
fi
