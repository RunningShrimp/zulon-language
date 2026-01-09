// bench_hashmap.rs - HashMap performance benchmark
use std::time::Instant;
use std::collections::HashMap as StdHashMap;

fn main() {
    println!("HashMap Performance Benchmark");
    println!("=============================");
    println!();

    // Test different sizes
    let sizes = [10, 100, 1000, 10000];

    for size in sizes {
        println!("Testing with {} entries:", size);
        println!("----------------------------");

        // Benchmark insertion
        let mut map = StdHashMap::new();
        let start = Instant::now();
        for i in 0..size {
            map.insert(i, i * 2);
        }
        let insert_duration = start.elapsed();

        // Benchmark lookup (successful)
        let start = Instant::now();
        for _ in 0..1000 {
            let key = size / 2; // Middle element
            let _ = map.get(&key);
        }
        let lookup_duration = start.elapsed();

        // Benchmark lookup (unsuccessful)
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = map.get(&(size + 1000)); // Non-existent key
        }
        let lookup_fail_duration = start.elapsed();

        // Benchmark iteration
        let start = Instant::now();
        let mut sum = 0;
        for (_, &val) in &map {
            sum += val;
        }
        let iter_duration = start.elapsed();

        println!("Insert:     {:8} ns ({:.2} us total)",
                 insert_duration.as_nanos() / size as u128,
                 insert_duration.as_micros() as f64);
        println!("Lookup:     {:8} ns/op (successful)",
                 lookup_duration.as_nanos() / 1000);
        println!("Lookup:     {:8} ns/op (failed)",
                 lookup_fail_duration.as_nanos() / 1000);
        println!("Iterate:    {:8} ns ({:.2} us total)",
                 iter_duration.as_nanos() / size as u128,
                 iter_duration.as_micros() as f64);
        println!("Sum check:  {}", sum);
        println!();
    }

    println!("Benchmark complete!");
}
