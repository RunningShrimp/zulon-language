#!/bin/bash
# Compare optimization levels: -O0 vs -O2

echo "=== ZULON Optimization Comparison: -O0 vs -O2 ==="
echo ""

# Test with hello_world (simple example)
echo "Testing: hello_world"
echo ""

cd crates/zulon-build/examples

# Backup original
cp hello_world.rs hello_world.rs.bak

echo "--- Phase 1: -O0 (No optimization) ---"
sed -i '' 's/opt_level: 0/opt_level: 0/' hello_world.rs

start=$(date +%s%N)
cargo run --example hello_world --quiet 2>&1 | tail -1
end=$(date +%s%N)
compile_o0=$(( (end - start) / 1000000 ))

if [ -f "hello_world" ]; then
    size_o0=$(stat -f%z hello_world)
    
    # Runtime test (10 runs)
    total=0
    for i in {1..10}; do
        start_run=$(date +%s%N)
        ./hello_world > /dev/null 2>&1
        end_run=$(date +%s%N)
        runtime=$(( (end_run - start_run) / 1000000 ))
        total=$((total + runtime))
    done
    runtime_o0=$((total / 10))
fi

echo ""
echo "--- Phase 2: -O2 (Standard optimization) ---"
sed -i '' 's/opt_level: 0/opt_level: 2/' hello_world.rs

start=$(date +%s%N)
cargo run --example hello_world --quiet 2>&1 | tail -1
end=$(date +%s%N)
compile_o2=$(( (end - start) / 1000000 ))

if [ -f "hello_world" ]; then
    size_o2=$(stat -f%z hello_world)
    
    # Runtime test (10 runs)
    total=0
    for i in {1..10}; do
        start_run=$(date +%s%N)
        ./hello_world > /dev/null 2>&1
        end_run=$(date +%s%N)
        runtime=$(( (end_run - start_run) / 1000000 ))
        total=$((total + runtime))
    done
    runtime_o2=$((total / 10))
fi

# Restore original
mv hello_world.rs.bak hello_world.rs
rm -f hello_world

cd ../../..

echo ""
echo "=== Results ==="
echo ""
echo "Metric              | -O0      | -O2      | Improvement"
echo "--------------------|----------|----------|-------------"
printf "Compile Time        | %-8s | %-8s | %s\n" "${compile_o0}ms" "${compile_o2}ms" "$(( (compile_o0 - compile_o2) * 100 / compile_o0 ))% faster"
printf "Binary Size         | %-8s | %-8s | %s\n" "${size_o0}B" "${size_o2}B" "$(( (size_o0 - size_o2) * 100 / size_o0 ))% smaller"
printf "Runtime (avg)       | %-8s | %-8s | %s\n" "${runtime_o0}ms" "${runtime_o2}ms" "$(( (runtime_o0 - runtime_o2) * 100 / runtime_o0 ))% faster"

echo ""
echo "✅ Comparison complete!"
