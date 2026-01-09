#!/bin/bash
# Full optimization benchmark across multiple examples

echo "=== ZULON Full Optimization Benchmark ==="
echo ""

cd crates/zulon-build/examples

# Results header
echo "example,metric,o0,o2,improvement" > ../../benchmark_results.csv

examples=("hello_world" "println_demo" "arc_demo")

for example in "${examples[@]}"; do
    echo ""
    echo "=== Testing: $example ==="
    
    # Backup
    cp "${example}.rs" "${example}.rs.bak"
    
    # Test -O0
    echo "  Testing -O0..."
    sed -i '' 's/opt_level: [0-9]/opt_level: 0/' "${example}.rs"
    cargo run --example "$example" --quiet 2>&1 | tail -1 > /dev/null
    
    if [ -f "$example" ]; then
        size_o0=$(stat -f%z "$example")
        
        # Runtime (10 runs)
        total=0
        for i in {1..10}; do
            start=$(date +%s%N)
            ./"$example" > /dev/null 2>&1
            end=$(date +%s%N)
            total=$((total + (end - start) / 1000000))
        done
        runtime_o0=$((total / 10))
    fi
    rm -f "$example"
    
    # Test -O2
    echo "  Testing -O2..."
    sed -i '' 's/opt_level: [0-9]/opt_level: 2/' "${example}.rs"
    cargo run --example "$example" --quiet 2>&1 | tail -1 > /dev/null
    
    if [ -f "$example" ]; then
        size_o2=$(stat -f%z "$example")
        
        # Runtime (10 runs)
        total=0
        for i in {1..10}; do
            start=$(date +%s%N)
            ./"$example" > /dev/null 2>&1
            end=$(date +%s%N)
            total=$((total + (end - start) / 1000000))
        done
        runtime_o2=$((total / 10))
    fi
    rm -f "$example"
    
    # Restore
    mv "${example}.rs.bak" "${example}.rs"
    
    # Calculate improvements
    size_improvement=$(( (size_o0 - size_o2) * 100 / size_o0 ))
    runtime_improvement=$(( (runtime_o0 - runtime_o2) * 100 / runtime_o0 ))
    
    echo "  Binary Size:     ${size_o0}B → ${size_o2}B (${size_improvement}% smaller)"
    echo "  Avg Runtime:     ${runtime_o0}ms → ${runtime_o2}ms (${runtime_improvement}% faster)"
    
    # Save results
    echo "$example,binary_size,$size_o0,$size_o2,${size_improvement}%" >> ../../benchmark_results.csv
    echo "$example,runtime,$runtime_o0,$runtime_o2,${runtime_improvement}%" >> ../../benchmark_results.csv
done

cd ../..

echo ""
echo "=== Summary ==="
cat benchmark_results.csv | column -t -s,

echo ""
echo "✅ Benchmark complete! Results saved to benchmark_results.csv"
