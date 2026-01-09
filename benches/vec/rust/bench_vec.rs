// bench_vec.rs - Vec performance benchmark
use std::time::Instant;

fn main() {
    println!("Vec Performance Benchmark");
    println!("=======================");
    println!();

    // Test different sizes
    let sizes: [usize; 4] = [10, 100, 1000, 10000];

    for size in sizes {
        println!("Testing with {} entries:", size);
        println!("----------------------------");

        // Benchmark push
        let mut vec = Vec::new();
        let start = Instant::now();
        for i in 0..size {
            vec.push(i as i32);
        }
        let push_duration = start.elapsed();

        // Benchmark insert at beginning (worst case)
        let mut vec: Vec<i32> = (0..size as i32).collect();
        let start = Instant::now();
        vec.insert(0, size as i32);
        let insert_begin_duration = start.elapsed();

        // Benchmark insert at middle
        let mut vec: Vec<i32> = (0..size as i32).collect();
        let start = Instant::now();
        vec.insert(size / 2, size as i32);
        let insert_middle_duration = start.elapsed();

        // Benchmark extend
        let mut vec: Vec<i32> = Vec::new();
        let start = Instant::now();
        let slice: &[i32] = &(0..size as i32).collect::<Vec<_>>();
        vec.extend(slice);
        let extend_duration = start.elapsed();

        // Benchmark iteration
        let vec: Vec<i32> = (0..size as i32).collect();
        let start = Instant::now();
        let mut sum = 0;
        for &val in &vec {
            sum += val;
        }
        let iter_duration = start.elapsed();

        // Benchmark reverse
        let mut vec: Vec<i32> = (0..size as i32).collect();
        let start = Instant::now();
        vec.reverse();
        let reverse_duration = start.elapsed();

        // Benchmark retain (filter even numbers)
        let mut vec: Vec<i32> = (0..size as i32).collect();
        let start = Instant::now();
        vec.retain(|&x| x % 2 == 0);
        let retain_duration = start.elapsed();

        println!("Push:        {:8} ns/op ({:.2} us total)",
                 push_duration.as_nanos() / size as u128,
                 push_duration.as_micros() as f64);
        println!("Insert beg:  {:8} ns",
                 insert_begin_duration.as_nanos());
        println!("Insert mid:  {:8} ns",
                 insert_middle_duration.as_nanos());
        println!("Extend:      {:8} ns/op ({:.2} us total)",
                 extend_duration.as_nanos() / size as u128,
                 extend_duration.as_micros() as f64);
        println!("Iterate:     {:8} ns/op ({:.2} us total)",
                 iter_duration.as_nanos() / size as u128,
                 iter_duration.as_micros() as f64);
        println!("Reverse:     {:8} ns ({:.2} us total)",
                 reverse_duration.as_nanos() / size as u128,
                 reverse_duration.as_micros() as f64);
        println!("Retain:      {:8} ns ({:.2} us total)",
                 retain_duration.as_nanos() / size as u128,
                 retain_duration.as_micros() as f64);
        println!("Sum check:   {}", sum);
        println!();
    }

    println!("Benchmark complete!");
}
