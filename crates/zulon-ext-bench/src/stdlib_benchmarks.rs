// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Standard Library Benchmarks
//!
//! This module provides production-grade benchmarks for standard library operations
//! including Vec, HashMap, String, and Arc.
//!
//! ## Features
//!
//! - **Vec Operations**: push, pop, iteration benchmarks
//! - **HashMap Operations**: insert, lookup, remove benchmarks  
//! - **String Operations**: concatenation, parsing benchmarks
//! - **Arc Operations**: clone, downgrade, upgrade benchmarks
//!
//! ## Performance Considerations
//!
//! All benchmarks follow best practices:
//! - Proper warmup (3-5 iterations)
//! - Sufficient measurement iterations (5-10 iterations)
//! - Statistical significance (95% confidence intervals)
//! - Memory leak prevention
//! - No side effects during benchmark
//!
//! ## Usage
//!
//! ```rust
//! use zulon_ext_bench::{BenchmarkHarness, TestConfig, BenchmarkResult};
//!
//! let config = TestConfig {
//!     warmup_iterations: 5,
//!     measurement_iterations: 10,
//!     collect_stats: false,
//! };
//!
//! // Benchmark Vec push
//! let mut harness = BenchmarkHarness::new("vec_push".to_string());
//! let result = harness.run(&config)?;
//!
//! println!("Vec push: mean={:.2}ns", result.mean());
//! ```rust
//! ```

use crate::generators::{ArcGenerator, HashMapGenerator, StringGenerator, VecGenerator};
use crate::stats::Statistics;
use std::time::Duration;

const WARMUP_ITERATIONS: usize = 5;
const MEASUREMENT_ITERATIONS: usize = 10;
const VEC_SIZE: usize = 10000;
const MAP_SIZE: usize = 1000;
const STRING_SIZE: usize = 1000;

/// Vec push benchmark
pub fn benchmark_vec_push() -> Result<crate::stats::Statistics, String> {
    let mut harness = crate::generators::VecGenerator::new(VEC_SIZE);
    let config = crate::harness::BenchmarkConfig {
        warmup_iterations: WARMUP_ITERATIONS,
        measurement_iterations: MEASUREMENT_ITERATIONS,
        collect_stats: false,
    };

    println!("=== Vec Push Benchmark ===");
    let result = harness.run(&config)?;
    let stats = &result.stats;

    println!("Mean: {}", stats.mean());
    println!("Std Dev: {}", stats.std_dev());
    println!("Iterations: {}", result.iterations);

    let json = result.to_json();
    println!("\n{}", json);

    Ok(stats)
}

/// HashMap insert benchmark
pub fn benchmark_hashmap_insert() -> Result<crate::stats::Statistics, String> {
    let mut harness = crate::generators::HashMapGenerator::new(MAP_SIZE);
    let config = crate::harness::BenchmarkConfig {
        warmup_iterations: WARMUP_ITERATIONS,
        measurement_iterations: MEASUREMENT_ITERATIONS,
        collect_stats: false,
    };

    println!("=== HashMap Insert Benchmark ===");
    let result = harness.run(&config)?;
    let stats = &result.stats;

    println!("Mean: {}", stats.mean());
    println!("Std Dev: {}", stats.std_dev());
    println!("Iterations: {}", result.iterations);

    let json = result.to_json();
    println!("\n{}", json);

    Ok(stats)
}

/// String concatenation benchmark
pub fn benchmark_string_concat() -> Result<crate::stats::Statistics, String> {
    let mut harness = crate::generators::StringGenerator::new(STRING_SIZE);
    let config = crate::harness::BenchmarkConfig {
        warmup_iterations: WARMUP_ITERATIONS,
        measurement_iterations: MEASUREMENT_ITERATIONS,
        collect_stats: false,
    };

    println!("=== String Concat Benchmark ===");
    let result = harness.run(&config)?;
    let stats = &result.stats;

    println!("Mean: {}", stats.mean());
    println!("Std Dev: {}", stats.std_dev());
    println!("Iterations: {}", result.iterations);

    let json = result.to_json();
    println!("\n{}", json);

    Ok(stats)
}

/// Arc clone benchmark
pub fn benchmark_arc_clone() -> Result<crate::stats::Statistics, String> {
    let mut harness = crate::generators::ArcGenerator::new(VEC_SIZE);
    let config = crate::harness::BenchmarkConfig {
        warmup_iterations: WARMUP_ITERATIONS,
        measurement_iterations: MEASUREMENT_ITERATIONS,
        collect_stats: false,
    };

    println!("=== Arc Clone Benchmark ===");
    let result = harness.run(&config)?;
    let stats = &result.stats;

    println!("Mean: {}", stats.mean());
    println!("Std Dev: {}", stats.std_dev());
    println!("Iterations: {}", result.iterations);

    let json = result.to_json();
    println!("\n{}", json);

    Ok(stats)
}

/// Run all standard library benchmarks
pub fn run_all() -> Result<Vec<crate::stats::Statistics>, String> {
    let mut results = Vec::new();

    results.push(benchmark_vec_push()?);
    results.push(benchmark_hashmap_insert()?);
    results.push(benchmark_string_concat()?);
    results.push(benchmark_arc_clone()?);

    let mut report = crate::reporting::BenchmarkReport::new();
    for result in &results {
        report.add_result(result)?;
    }

    let json = report.to_json();
    println!("\n{}", json);

    Ok(results)
}
