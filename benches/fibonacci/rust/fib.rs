// fib.rs - Rust Fibonacci benchmark
use std::time::Instant;

fn fib(n: i32) -> i32 {
    if n <= 1 { return n; }
    fib(n - 1) + fib(n - 2)
}

fn main() {
    println!("Rust Fibonacci Benchmark");
    println!("========================");

    // Warmup
    fib(30);

    // Benchmark
    let start = Instant::now();
    let result = fib(40);
    let duration = start.elapsed();

    println!("fib(40) = {}", result);
    println!("Time: {}ms", duration.as_millis());
    println!("Throughput: {:.2} ops/sec", 40000000.0 / duration.as_millis() as f64);
}
