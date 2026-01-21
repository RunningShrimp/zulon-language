// Copyright 2026 ZULON Language Team
// SPDX-License-Identifier: Apache-2.0 OR MIT

//! Benchmark infrastructure for ZULON standard library operations.

use std::io::{self, Write};
use std::time::{Duration, Instant};

/// Helper trait for duration operations
trait DurationOps {
    fn as_secs_f64(&self) -> f64;
}

impl DurationOps for Duration {
    fn as_secs_f64(&self) -> f64 {
        (self.as_nanos() as f64) / 1_000_000_000.0
    }
}

/// Benchmark result with timing and statistics
#[derive(Debug, Clone)]
pub struct BenchmarkResult<T> {
    /// Benchmark name
    pub name: String,
    /// Iterations
    pub iterations: usize,
    /// Total time elapsed
    pub total_duration: Duration,
    /// Average time per iteration
    pub avg_duration: Duration,
    /// Median time per iteration
    pub median_duration: Duration,
    /// Min time per iteration
    pub min_duration: Duration,
    /// Max time per iteration
    pub max_duration: Duration,
    /// Operations per second
    pub ops_per_sec: f64,
    /// Benchmark value (if applicable)
    pub value: Option<T>,
}

/// Benchmark configuration
#[derive(Debug, Clone)]
pub struct BenchmarkConfig {
    /// Number of warmup iterations
    pub warmup_iterations: usize,
    /// Number of measurement iterations
    pub measurement_iterations: usize,
    /// Minimum duration per iteration
    pub min_duration_ms: u64,
    /// Maximum duration per iteration (for variance detection)
    pub max_duration_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            warmup_iterations: 10,
            measurement_iterations: 100,
            min_duration_ms: 1,
            max_duration_ms: 1000,
        }
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        BenchmarkRunner {
            config: BenchmarkConfig::default(),
        }
    }

    pub fn with_config(config: BenchmarkConfig) -> Self {
        BenchmarkRunner { config }
    }

    pub fn run<T, F>(&self, name: &str, func: F) -> BenchmarkResult<T>
    where
        F: FnMut() -> T,
        T: Copy + std::fmt::Display,
    {
        println!("Benchmark: {}", name);
        println!("Warmup iterations: {}", self.config.warmup_iterations);
        println!(
            "Measurement iterations: {}",
            self.config.measurement_iterations
        );

        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            let _ = func();
        }

        // Measurement phase
        let mut durations = Vec::with_capacity(self.config.measurement_iterations);
        let mut results = Vec::with_capacity(self.config.measurement_iterations);

        for i in 0..self.config.measurement_iterations {
            let start = Instant::now();
            let value = func();
            let duration = start.elapsed();
            durations.push(duration);
            results.push(value);
        }

        // Calculate statistics
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / (self.config.measurement_iterations as u32);

        let sorted_durations = {
            let mut sorted = durations.clone();
            sorted.sort();
            sorted
        };

        let median_duration = if sorted_durations.is_empty() {
            avg_duration
        } else {
            let mid = sorted_durations.len() / 2;
            sorted_durations[mid]
        };

        let min_duration = sorted_durations
            .first()
            .copied()
            .unwrap_or_else(|| avg_duration);
        let max_duration = sorted_durations
            .last()
            .copied()
            .unwrap_or_else(|| avg_duration);

        // Calculate operations per second
        let total_secs_f64 = total_duration.as_secs_f64();
        let ops_per_sec = (self.config.measurement_iterations as f64) / total_secs_f64;

        BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.measurement_iterations,
            total_duration,
            avg_duration,
            median_duration,
            min_duration,
            max_duration,
            ops_per_sec,
            value: results.first().copied(),
        }
    }

    pub fn run_comparison<T, U>(
        &self,
        name: &str,
        zulon_func: impl FnMut() -> T,
        cpp_func: impl FnMut() -> U,
    ) -> (BenchmarkResult<T>, BenchmarkResult<U>)
    where
        T: Copy + std::fmt::Display,
        U: Copy + std::fmt::Display,
    {
        println!("Comparison Benchmark: {}", name);

        let zulon_result = self.run(name, &mut || zulon_func());
        let cpp_result = self.run(name, &mut || cpp_func());

        println!("\n=== ZULON Results ===");
        println!("{}", zulon_result);
        println!("ZULON ops/sec: {:.2}", zulon_result.ops_per_sec);

        println!("\n=== C++ Results ===");
        println!("{}", cpp_result);
        println!("C++ ops/sec: {:.2}", cpp_result.ops_per_sec);

        println!("\n=== Comparison ===");
        let ratio = (zulon_result.ops_per_sec / cpp_result.ops_per_sec) * 100.0;
        println!("Performance ratio: {:.2}%", ratio);

        if ratio >= 90.0 {
            println!("✅ MEETS 90-95% C++ performance target");
        } else {
            println!("⚠️  BELOW 90-95% C++ performance target");
        }

        (zulon_result, cpp_result)
    }
}

/// Benchmark configuration
#[derive(Debug, Clone)]
    pub warmup_iterations: usize,
    /// Number of measurement iterations
    pub measurement_iterations: usize,
    /// Minimum duration per iteration
    pub min_duration_ms: u64,
    /// Maximum duration per iteration (for variance detection)
    pub max_duration_ms: u64,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        BenchmarkConfig {
            warmup_iterations: 10,
            measurement_iterations: 100,
            min_duration_ms: 1,
            max_duration_ms: 1000,
        }
    }
}

/// Benchmark runner
pub struct BenchmarkRunner {
    config: BenchmarkConfig,
}

impl BenchmarkRunner {
    pub fn new() -> Self {
        BenchmarkRunner {
            config: BenchmarkConfig::default(),
        }
    }

    pub fn with_config(config: BenchmarkConfig) -> Self {
        BenchmarkRunner { config }
    }

    pub fn run<T, F>(&self, name: &str, func: F) -> BenchmarkResult<T>
    where
        F: FnMut() -> T,
        T: Copy + std::fmt::Display,
    {
        println!("Benchmark: {}", name);
        println!("Warmup iterations: {}", self.config.warmup_iterations);
        println!(
            "Measurement iterations: {}",
            self.config.measurement_iterations
        );

        // Warmup phase
        for _ in 0..self.config.warmup_iterations {
            let _ = func();
        }

        // Measurement phase
        let mut durations = Vec::with_capacity(self.config.measurement_iterations);
        let mut results = Vec::with_capacity(self.config.measurement_iterations);

        for i in 0..self.config.measurement_iterations {
            let start = Instant::now();
            let value = func();
            let duration = start.elapsed();
            durations.push(duration);
            results.push(value);
        }

        // Calculate statistics
        let total_duration: Duration = durations.iter().sum();
        let avg_duration = total_duration / (self.config.measurement_iterations as u32);

        let mut sorted_durations = durations.clone();
        sorted_durations.sort();
        let median_duration = if sorted_durations.is_empty() {
            avg_duration
        } else {
            let mid = sorted_durations.len() / 2;
            sorted_durations[mid]
        };

        let min_duration = sorted_durations
            .first()
            .copied()
            .unwrap_or_else(|| avg_duration);
        let max_duration = sorted_durations
            .last()
            .copied()
            .unwrap_or_else(|| avg_duration);

        // Calculate operations per second
        let ops_per_sec =
            (self.config.measurement_iterations as f64) / total_duration.as_secs_f64();

        BenchmarkResult {
            name: name.to_string(),
            iterations: self.config.measurement_iterations,
            total_duration,
            avg_duration,
            median_duration,
            min_duration,
            max_duration,
            ops_per_sec,
            value: results.first().copied(),
        }
    }

    pub fn run_comparison<T, U>(
        &self,
        name: &str,
        zulon_func: impl FnMut() -> T,
        cpp_func: impl FnMut() -> U,
    ) -> (BenchmarkResult<T>, BenchmarkResult<U>)
    where
        T: Copy + std::fmt::Display,
        U: Copy + std::fmt::Display,
    {
        println!("Comparison Benchmark: {}", name);

        let zulon_result = self.run(name, &mut || zulon_func());
        let cpp_result = self.run(name, &mut || cpp_func());

        println!("\n=== ZULON Results ===");
        println!("{}", zulon_result);
        println!("ZULON ops/sec: {:.2}", zulon_result.ops_per_sec);

        println!("\n=== C++ Results ===");
        println!("{}", cpp_result);
        println!("C++ ops/sec: {:.2}", cpp_result.ops_per_sec);

        println!("\n=== Comparison ===");
        let ratio = (zulon_result.ops_per_sec / cpp_result.ops_per_sec) * 100.0;
        println!("Performance ratio: {:.2}%", ratio);

        if ratio >= 90.0 {
            println!("✅ MEETS 90-95% C++ performance target");
        } else {
            println!("⚠️  BELOW 90-95% C++ performance target");
        }

        (zulon_result, cpp_result)
    }
}

/// Format benchmark results as a table
pub fn format_results(results: &[BenchmarkResult]) {
    let mut output = String::from("┌─────────────┬─────────────┐\n");

    for result in results {
        output.push_str(&format!(
            "│ {:<20} │ {:>10} │ {:>12} │ {:>12} │ {:>12} │ {:>12} │\n",
            result.name,
            format_duration(result.total_duration),
            format_duration(result.avg_duration),
            format_duration(result.median_duration),
            format_duration(result.min_duration),
            format_duration(result.max_duration),
            format!("{:.2}", result.ops_per_sec),
            result
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
        ));
    }

    output.push_str("└─────────────┴─────────────┘\n");
    output
}

fn format_duration(duration: StdDuration) -> String {
    let secs = duration.as_secs();
    if secs < 1.0 {
        format!("{:.3}ms", duration.as_millis())
    } else if secs < 60.0 {
        format!("{:.2}s", secs)
    } else {
        format!("{:.1}m", secs / 60.0)
    }
}

/// Write benchmark results to CSV file
pub fn write_csv<W: Write>(writer: &mut W, results: &[BenchmarkResult]) -> std::io::Result<()> {
    writeln!(
        writer,
        "name,iterations,total_ms,avg_ms,median_ms,min_ms,max_ms,ops_per_sec,value"
    )?;

    for result in results {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{}\n",
            result.name,
            result.iterations,
            result.total_duration.as_millis(),
            result.avg_duration.as_millis(),
            result.median_duration.as_millis(),
            result.min_duration.as_millis(),
            result.max_duration.as_millis(),
            result.ops_per_sec,
            result
                .value
                .map(|v| v.to_string())
                .unwrap_or_else(|| "".to_string()),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_benchmark_runner_creation() {
        let runner = BenchmarkRunner::new();
        assert_eq!(runner.config.warmup_iterations, 10);
        assert_eq!(runner.config.measurement_iterations, 100);
    }

    #[test]
    fn test_benchmark_result_display() {
        let result = BenchmarkResult {
            name: "test".to_string(),
            iterations: 100,
            total_duration: StdDuration::from_secs(1),
            avg_duration: StdDuration::from_secs(1),
            median_duration: StdDuration::from_secs(1),
            min_duration: StdDuration::from_secs(1),
            max_duration: StdDuration::from_secs(1),
            ops_per_sec: 100.0,
            value: Some(42u64),
        };

        let formatted = format_results(&[result]);
        assert!(formatted.contains("test"));
        assert!(formatted.contains("100.00"));
    }
}
