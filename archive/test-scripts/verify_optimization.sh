#!/bin/bash
echo "=== Verifying -O2 Optimization ==="
echo ""

# Test hello_world
if [ -f "hello_world" ]; then
    echo "✅ Binary exists: hello_world"
    
    # Benchmark runtime (10 runs)
    total=0
    for i in {1..10}; do
        start=$(date +%s%N)
        ./hello_world > /dev/null 2>&1
        end=$(date +%s%N)
        runtime=$(( (end - start) / 1000000 ))
        total=$((total + runtime))
    done
    avg_runtime=$((total / 10))
    
    echo "   Average runtime (10 runs): ${avg_runtime}ms"
    
    # Previous -O0 result was 84ms
    # Expected -O2 result is ~24ms
    if [ $avg_runtime -lt 50 ]; then
        echo "   ✅ Optimization confirmed! (< 50ms)"
    else
        echo "   ⚠️  May not be optimized (>= 50ms)"
    fi
else
    echo "❌ Binary not found"
fi

echo ""
echo "✅ Verification complete!"
