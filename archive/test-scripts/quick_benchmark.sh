#!/bin/bash

echo "=== ZULON Performance Summary ==="
echo ""

# Test compilation time (single example)
echo "📊 Compilation Performance:"
echo "Compiling hello_world example..."
time cargo run --example hello_world --quiet 2>&1 | grep -E "(real|user|sys)" || true
echo ""

# Test runtime performance
echo "⚡ Runtime Performance (average of 10 runs):"
for exe in hello_world println_demo arc_demo; do
    if [ -f "./$exe" ]; then
        total=0
        for i in {1..10}; do
            start=$(gdate +%s%N 2>/dev/null || date +%s%N)
            ./$exe > /dev/null 2>&1
            end=$(gdate +%s%N 2>/dev/null || date +%s%N)
            runtime=$(( (end - start) / 1000000 ))
            total=$((total + runtime))
        done
        avg=$((total / 10))
        echo "  $exe: ${avg}ms"
    fi
done
echo ""

# Binary sizes
echo "📦 Binary Sizes:"
for exe in hello_world println_demo print_call_example arc_demo comprehensive_io_demo; do
    if [ -f "./$exe" ]; then
        size=$(ls -l "$exe" | awk '{print $5}')
        size_kb=$((size / 1024))
        echo "  $exe: $size bytes ($size_kb KB)"
    fi
done
echo ""

echo "✅ Benchmark complete!"
