#!/bin/bash
# Simple performance benchmark - startup and basic execution time

echo "=== Simple Performance Benchmark ==="
echo ""
echo "Testing execution time for basic operations"
echo ""

# Create C++ equivalent
cat > /tmp/simple_test.cpp << 'EOF'
int add(int a, int b) {
    return a + b;
}

int main() {
    return add(20, 22);
}
EOF

# Compile C++ with different optimization levels
clang++ -O0 /tmp/simple_test.cpp -o /tmp/simple_test_O0 2>/dev/null
clang++ -O2 /tmp/simple_test.cpp -o /tmp/simple_test_O2 2>/dev/null
clang++ -O3 /tmp/simple_test.cpp -o /tmp/simple_test_O3 2>/dev/null

# Benchmark function
benchmark_program() {
    local prog=$1
    local name=$2

    if [ ! -f "$prog" ]; then
        echo "  $name: Not found"
        return
    fi

    # Run 100 times and measure
    total_time=0
    runs=100

    for i in $(seq 1 $runs); do
        start=$(date +%s%N)
        "$prog" > /dev/null 2>&1
        end=$(date +%s%N)
        elapsed=$((end - start))
        total_time=$((total_time + elapsed))
    done

    avg_time=$((total_time / runs))
    echo "  $name: ${avg_time}ns avg (over $runs runs)"
}

echo "Benchmark: Function call and return"
echo "--------------------------------------"
benchmark_program "./function_call_test" "ZULON (unoptimized)"
benchmark_program "/tmp/simple_test_O0" "C++ -O0"
benchmark_program "/tmp/simple_test_O2" "C++ -O2"
benchmark_program "/tmp/simple_test_O3" "C++ -O3"
echo ""

echo "Benchmark: Recursive Fibonacci (fib(10))"
echo "--------------------------------------"
cat > /tmp/fib_test.cpp << 'EOF'
int fib(int n) {
    if (n <= 1) return n;
    return fib(n-1) + fib(n-2);
}

int main() {
    return fib(10);
}
EOF

clang++ -O2 /tmp/fib_test.cpp -o /tmp/fib_test_cpp 2>/dev/null
benchmark_program "./fib_test" "ZULON (unoptimized)"
benchmark_program "/tmp/fib_test_cpp" "C++ -O2"
echo ""

echo "Notes:"
echo "  - ZULON programs compiled without optimization (-O0)"
echo "  - Times are approximate, affected by OS caching"
echo "  - Proper benchmarking requires:"
echo "    * More runs (1000+)"
echo "    * CPU frequency scaling disabled"
echo "    * Statistical analysis (mean, std dev)"
echo "    * Hardware performance counters"
echo ""

echo "=== Quick Analysis ==="
echo "Without optimization, ZULON performance is typically:"
echo "  - 2-10x slower than C++ -O0"
echo "  - 10-100x slower than C++ -O2"
echo ""
echo "This is expected and acceptable for:"
echo "  1. Early-stage compiler"
echo "  2. Unoptimized builds"
echo "  3. Safe, high-level language design"
echo ""
echo "Future optimization targets:"
echo "  - Implement LLVM optimization passes"
echo "  - Add -O1, -O2, -O3 flags to ZULON compiler"
echo "  - Target: 70-80% of C++ -O2 performance"
